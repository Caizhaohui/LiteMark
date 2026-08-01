/**
 * Document session state for M1. Owns the list of open documents, the active
 * tab, dirty state, and the open/save/saveAs/close/reload flows.
 *
 * Design notes:
 * - `dirty` is never set here as a guess; it is refreshed from the Rust core
 *   (which derives it from the content hash). This matches §6.1.
 * - Edits are debounced to `set_document_content` so recovery snapshots are
 *   not written on every keystroke.
 * - External modification is detected by polling `check_external_change` when
 *   the window regains focus; the store surfaces a prompt rather than
 *   silently overwriting.
 */

import { useCallback, useEffect, useReducer, useRef } from "react";
import type { DocumentSession, SessionSummary } from "@litemark/shared-protocol";
import * as cmd from "../services/tauriCommands";
import type { CoreError } from "../services/tauriCommands";

/** The shape of an unsaved-close confirmation the UI must show. */
export interface PendingClose {
  /** The session the user is trying to close. */
  sessionId: string;
}

/** An external-modification notice the UI must show. */
export interface PendingExternalChange {
  sessionId: string;
  displayName: string;
}

/** A user-facing error notice. */
export interface AppNotice {
  id: number;
  level: "error" | "info";
  message: string;
  code?: string;
}

export interface DocumentStoreState {
  sessions: SessionSummary[];
  activeId: string | null;
  /** Full content of the active document (for the textarea). */
  activeContent: string;
  activeEncoding: string;
  activeLineEnding: string;
  activeReadOnly: boolean;
  pendingClose: PendingClose | null;
  pendingExternal: PendingExternalChange | null;
  notices: AppNotice[];
  busy: boolean;
}

type Action =
  | { type: "refresh"; sessions: SessionSummary[]; activeId: string | null; active?: DocumentSession }
  | { type: "setActive"; activeId: string | null; active?: DocumentSession }
  | { type: "beginEdit"; content: string }
  | { type: "busy"; busy: boolean }
  | { type: "pendingClose"; pending: PendingClose | null }
  | { type: "pendingExternal"; pending: PendingExternalChange | null }
  | { type: "notice"; notice: AppNotice }
  | { type: "dismissNotice"; id: number };

function reducer(state: DocumentStoreState, action: Action): DocumentStoreState {
  switch (action.type) {
    case "refresh": {
      const active = action.active;
      return {
        ...state,
        sessions: action.sessions,
        activeId: action.activeId,
        activeContent: active?.content ?? state.activeContent,
        activeEncoding: active?.encoding ?? state.activeEncoding,
        activeLineEnding: active?.lineEnding ?? state.activeLineEnding,
        activeReadOnly: active?.readOnly ?? state.activeReadOnly,
      };
    }
    case "setActive": {
      const active = action.active;
      return {
        ...state,
        activeId: action.activeId,
        pendingExternal: null,
        activeContent: active?.content ?? "",
        activeEncoding: active?.encoding ?? "utf-8",
        activeLineEnding: active?.lineEnding ?? "lf",
        activeReadOnly: active?.readOnly ?? false,
      };
    }
    case "beginEdit":
      return { ...state, activeContent: action.content };
    case "busy":
      return { ...state, busy: action.busy };
    case "pendingClose":
      return { ...state, pendingClose: action.pending };
    case "pendingExternal":
      return { ...state, pendingExternal: action.pending };
    case "notice":
      return { ...state, notices: [...state.notices, action.notice] };
    case "dismissNotice":
      return { ...state, notices: state.notices.filter((n) => n.id !== action.id) };
    default:
      return state;
  }
}

const initialState: DocumentStoreState = {
  sessions: [],
  activeId: null,
  activeContent: "",
  activeEncoding: "utf-8",
  activeLineEnding: "lf",
  activeReadOnly: false,
  pendingClose: null,
  pendingExternal: null,
  notices: [],
  busy: false,
};

let noticeCounter = 0;

export function useDocumentStore() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const editTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSentContent = useRef<string>("");
  const activeIdRef = useRef<string | null>(null);
  activeIdRef.current = state.activeId;

  /** Re-fetch the session list and (optionally) the active document's content. */
  const refresh = useCallback(async (opts?: { fetchActive?: boolean }) => {
    const [sessions, activeId] = await Promise.all([cmd.listDocuments(), cmd.activeDocument()]);
    let active: DocumentSession | undefined;
    if (opts?.fetchActive && activeId) {
      active = await cmd.getDocument(activeId);
    }
    dispatch({ type: "refresh", sessions, activeId, active });
    return { sessions, activeId };
  }, []);

  /** Push the current textarea content to Rust (debounced). */
  const flushEdit = useCallback(
    async (sessionId: string, content: string) => {
      try {
        await cmd.setDocumentContent(sessionId, content);
        lastSentContent.current = content;
        // Refresh summaries so the dirty dot updates.
        await refresh();
      } catch (e) {
        const err = cmd.toCoreError(e);
        dispatch({
          type: "notice",
          notice: { id: ++noticeCounter, level: "error", message: err.message, code: err.code },
        });
      }
    },
    [refresh],
  );

  /** Called on every textarea change. Debounces the write to Rust. */
  const onContentChange = useCallback(
    (content: string) => {
      dispatch({ type: "beginEdit", content });
      const id = activeIdRef.current;
      if (!id) return;
      if (editTimer.current) clearTimeout(editTimer.current);
      editTimer.current = setTimeout(() => {
        void flushEdit(id, content);
      }, 400);
    },
    [flushEdit],
  );

  /** Create a new empty document and switch to it. */
  const newDocument = useCallback(async () => {
    dispatch({ type: "busy", busy: true });
    try {
      const id = await cmd.newDocument();
      await refresh({ fetchActive: true });
      void cmd.setActiveDocument(id);
    } catch (e) {
      const err = cmd.toCoreError(e);
      dispatch({
        type: "notice",
        notice: { id: ++noticeCounter, level: "error", message: err.message, code: err.code },
      });
    } finally {
      dispatch({ type: "busy", busy: false });
    }
  }, [refresh]);

  /** Open an existing file path in a new tab. */
  const openPath = useCallback(
    async (path: string): Promise<string | null> => {
      dispatch({ type: "busy", busy: true });
      try {
        const id = await cmd.openFile(path);
        await refresh({ fetchActive: true });
        void cmd.setActiveDocument(id);
        return id;
      } catch (e) {
        const err = cmd.toCoreError(e);
        dispatch({
          type: "notice",
          notice: { id: ++noticeCounter, level: "error", message: err.message, code: err.code },
        });
        return null;
      } finally {
        dispatch({ type: "busy", busy: false });
      }
    },
    [refresh],
  );

  /** Show the native open dialog and open the chosen file. */
  const openViaDialog = useCallback(async (): Promise<string | null> => {
    dispatch({ type: "busy", busy: true });
    try {
      const path = await cmd.showOpenDialog();
      if (!path) return null;
      return await openPath(path);
    } finally {
      dispatch({ type: "busy", busy: false });
    }
  }, [openPath]);

  /** Save the active document to its current path. */
  const saveActive = useCallback(async (): Promise<boolean> => {
    const id = activeIdRef.current;
    if (!id) return false;
    dispatch({ type: "busy", busy: true });
    try {
      // Flush any pending edit before saving.
      if (editTimer.current) {
        clearTimeout(editTimer.current);
        editTimer.current = null;
      }
      if (lastSentContent.current !== state.activeContent) {
        await flushEdit(id, state.activeContent);
      }
      await cmd.saveDocument(id);
      await refresh({ fetchActive: true });
      return true;
    } catch (e) {
      const err = cmd.toCoreError(e);
      if (err.code === "PATH_NOT_AUTHORIZED") {
        // No file path yet → fall through to Save As.
        return await saveActiveAs();
      }
      dispatch({
        type: "notice",
        notice: { id: ++noticeCounter, level: "error", message: err.message, code: err.code },
      });
      return false;
    } finally {
      dispatch({ type: "busy", busy: false });
    }
  }, [flushEdit, refresh, state.activeContent]);

  /** Save the active document under a new path (Save As). */
  const saveActiveAs = useCallback(async (): Promise<boolean> => {
    const id = activeIdRef.current;
    if (!id) return false;
    dispatch({ type: "busy", busy: true });
    try {
      const name = state.sessions.find((s) => s.id === id)?.displayName ?? "Untitled.md";
      const path = await cmd.showSaveDialog(`${name}.md`);
      if (!path) return false;
      await cmd.saveAsDocument(id, path);
      await refresh({ fetchActive: true });
      return true;
    } catch (e) {
      const err = cmd.toCoreError(e);
      dispatch({
        type: "notice",
        notice: { id: ++noticeCounter, level: "error", message: err.message, code: err.code },
      });
      return false;
    } finally {
      dispatch({ type: "busy", busy: false });
    }
  }, [refresh, state.sessions]);

  /** Request to close a tab. If dirty, surface a confirmation prompt. */
  const requestClose = useCallback(
    async (sessionId: string) => {
      // Flush pending edits so dirty is accurate.
      const id = activeIdRef.current;
      if (id === sessionId && editTimer.current) {
        clearTimeout(editTimer.current);
        editTimer.current = null;
        if (lastSentContent.current !== state.activeContent) {
          await flushEdit(id, state.activeContent);
        }
      }
      const dirty = state.sessions.find((s) => s.id === sessionId)?.dirty ?? false;
      if (dirty) {
        dispatch({ type: "pendingClose", pending: { sessionId } });
      } else {
        await confirmClose(sessionId, "discard");
      }
    },
    [flushEdit, state.activeContent, state.sessions],
  );

  /** Resolve a pending-close confirmation. */
  const confirmClose = useCallback(
    async (sessionId: string, choice: "save" | "discard" | "cancel") => {
      dispatch({ type: "pendingClose", pending: null });
      if (choice === "cancel") return;
      if (choice === "save") {
        const ok = await saveActive();
        if (!ok) return; // save failed or cancelled — keep the tab
      }
      try {
        await cmd.closeDocument(sessionId);
        await refresh({ fetchActive: true });
      } catch (e) {
        const err = cmd.toCoreError(e);
        dispatch({
          type: "notice",
          notice: { id: ++noticeCounter, level: "error", message: err.message, code: err.code },
        });
      }
    },
    [refresh, saveActive],
  );

  /** Activate a tab. */
  const activate = useCallback(
    async (sessionId: string) => {
      // Flush the outgoing tab's pending edit first.
      const prev = activeIdRef.current;
      if (prev && prev !== sessionId && editTimer.current) {
        clearTimeout(editTimer.current);
        editTimer.current = null;
        if (lastSentContent.current !== state.activeContent) {
          await flushEdit(prev, state.activeContent);
        }
      }
      await cmd.setActiveDocument(sessionId);
      const active = await cmd.getDocument(sessionId);
      lastSentContent.current = active.content;
      dispatch({ type: "setActive", activeId: sessionId, active });
    },
    [flushEdit, state.activeContent],
  );

  /** Resolve an external-change prompt. */
  const resolveExternal = useCallback(
    async (choice: "reload" | "keep" | "compare") => {
      const pending = state.pendingExternal;
      dispatch({ type: "pendingExternal", pending: null });
      if (!pending || choice !== "reload") return;
      // Reopen the file from disk into the same session by closing + reopening.
      const session = state.sessions.find((s) => s.id === pending.sessionId);
      const path = session?.filePath ?? null;
      if (!path) return;
      await cmd.closeDocument(pending.sessionId);
      await openPath(path);
    },
    [openPath, state.pendingExternal, state.sessions],
  );

  /** Poll for external changes (called on window focus). */
  const pollExternalChange = useCallback(async () => {
    const id = activeIdRef.current;
    if (!id || state.pendingExternal) return;
    try {
      const changed = await cmd.checkExternalChange(id);
      if (changed) {
        const name = state.sessions.find((s) => s.id === id)?.displayName ?? "document";
        dispatch({ type: "pendingExternal", pending: { sessionId: id, displayName: name } });
      }
    } catch {
      // Non-fatal: ignore detection errors.
    }
  }, [state.pendingExternal, state.sessions]);

  /** Dismiss a notice. */
  const dismissNotice = useCallback((id: number) => {
    dispatch({ type: "dismissNotice", id });
  }, []);

  // Initial load.
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    state,
    refresh,
    onContentChange,
    newDocument,
    openPath,
    openViaDialog,
    saveActive,
    saveActiveAs,
    requestClose,
    confirmClose,
    activate,
    resolveExternal,
    pollExternalChange,
    dismissNotice,
  };
}

export type DocumentStore = ReturnType<typeof useDocumentStore>;

/** Shape of a SaveResult (kept here to avoid a circular import). */
export interface SaveResult {
  mtimeMs: number;
  contentHash: string;
}

export type { CoreError };
