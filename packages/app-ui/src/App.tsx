/**
 * LiteMark application shell (M0–M6).
 *
 * M1 lifecycle · M2 Monaco+preview · M3 HTML/PDF export ·
 * M4 hybrid Milkdown · M5 pandoc/settings/trust · M6 diagnostics.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PreviewLayout } from "@litemark/shared-protocol";
import { hybridRoundtrip } from "@litemark/markdown-core";
import { TabBar } from "./components/TabBar";
import { EditorPane } from "./components/EditorPane";
import { PreviewPane } from "./components/PreviewPane";
import { StatusBar } from "./components/StatusBar";
import { ConfirmClose } from "./components/ConfirmClose";
import { ExternalChangePrompt } from "./components/ExternalChangePrompt";
import { RecoveryPrompt } from "./components/RecoveryPrompt";
import { DropZone } from "./components/DropZone";
import { ExportDialog } from "./components/ExportDialog";
import { LicensesDialog } from "./components/LicensesDialog";
import { ModeSwitchWarning } from "./components/ModeSwitchWarning";
import { SettingsDialog } from "./components/settings/SettingsDialog";
import type { EditorMode } from "./components/EditorModeBar";
import { HybridEditor, HybridToolbar, applyToolbarMarkdown, type HybridToolbarCommand } from "./editors/HybridEditor";
import { useDocumentStore } from "./hooks/useDocumentStore";
import { useRecovery } from "./hooks/useRecovery";
import { usePreview } from "./hooks/usePreview";
import { useExport } from "./hooks/useExport";
import * as cmd from "./services/tauriCommands";

import "katex/dist/katex.min.css";

const MARKDOWN_EXT = /\.(md|markdown|mdx|mkd)$/i;

export function App(): JSX.Element {
  const store = useDocumentStore();
  const recovery = useRecovery();
  const exporter = useExport();
  const [dragArmed, setDragArmed] = useState(false);
  const [layout, setLayout] = useState<PreviewLayout>("split");
  const [editorMode, setEditorMode] = useState<EditorMode>("source");
  const [modeRisks, setModeRisks] = useState<string[] | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [editorScroll, setEditorScroll] = useState<number | null>(null);
  const [previewScroll, setPreviewScroll] = useState<number | null>(null);
  const [tocJump, setTocJump] = useState<string | null>(null);
  const [licensesText, setLicensesText] = useState<string | null>(null);
  const { state } = store;
  const activeSession = state.sessions.find((s) => s.id === state.activeId) ?? null;

  // Hybrid mode: hide live preview side by default in "source" layout semantics;
  // layout still controls panes. Preview enabled when not layout=source-only.
  const previewEnabled = layout !== "source" && !!activeSession;
  const preview = usePreview({
    sessionId: state.activeId,
    content: state.activeContent,
    enabled: previewEnabled,
  });

  const openPathRef = useRef(store.openPath);
  openPathRef.current = store.openPath;

  useEffect(() => {
    const unlistenPromise = cmd.onOpenFiles((files) => {
      for (const f of files) {
        if (MARKDOWN_EXT.test(f)) void openPathRef.current(f);
      }
    });
    return () => {
      void unlistenPromise.then((un) => un());
    };
  }, []);

  useEffect(() => {
    const webview = getCurrentWebview();
    const unlisten = webview.onDragDropEvent((event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        setDragArmed(true);
      } else if (event.payload.type === "drop") {
        setDragArmed(false);
        const paths = event.payload.paths.filter((p) => MARKDOWN_EXT.test(p));
        for (const p of paths) void openPathRef.current(p);
      } else if (event.payload.type === "leave") {
        setDragArmed(false);
      }
    });
    return () => {
      void unlisten.then((u) => u());
    };
  }, []);

  const pollRef = useRef(store.pollExternalChange);
  pollRef.current = store.pollExternalChange;
  useEffect(() => {
    const win = getCurrentWindow();
    const unlisten = win.onFocusChanged(({ payload: focused }) => {
      if (focused) void pollRef.current();
    });
    return () => {
      void unlisten.then((u) => u());
    };
  }, []);

  // Reset editor mode when switching tabs.
  useEffect(() => {
    setEditorMode("source");
    setModeRisks(null);
  }, [state.activeId]);

  const saveRef = useRef(store.saveActive);
  saveRef.current = store.saveActive;
  const saveAsRef = useRef(store.saveActiveAs);
  saveAsRef.current = store.saveActiveAs;
  const newRef = useRef(store.newDocument);
  newRef.current = store.newDocument;
  const openRef = useRef(store.openViaDialog);
  openRef.current = store.openViaDialog;
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.ctrlKey || e.metaKey;
      if (!mod) return;
      if (e.key === "s" && e.shiftKey) {
        e.preventDefault();
        void saveAsRef.current();
      } else if (e.key === "s") {
        e.preventDefault();
        void saveRef.current();
      } else if (e.key === "n") {
        e.preventDefault();
        void newRef.current();
      } else if (e.key === "o") {
        e.preventDefault();
        void openRef.current();
      } else if (e.key === "1") {
        e.preventDefault();
        setLayout("source");
      } else if (e.key === "2") {
        e.preventDefault();
        setLayout("split");
      } else if (e.key === "3") {
        e.preventDefault();
        setLayout("preview");
      } else if (e.key === ",") {
        e.preventDefault();
        setSettingsOpen(true);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const handleRestore = useCallback(
    async (recoveryKey: string) => {
      const id = await recovery.restoreOne(recoveryKey);
      if (id) {
        await store.refresh({ fetchActive: true });
        void cmd.setActiveDocument(id);
      }
    },
    [recovery, store],
  );

  const handleRecoveryDiscardAll = useCallback(async () => {
    await recovery.discardAll();
    await store.refresh();
  }, [recovery, store]);

  const onEditorScroll = useCallback((ratio: number) => {
    setPreviewScroll(ratio);
  }, []);
  const onPreviewScroll = useCallback((ratio: number) => {
    setEditorScroll(ratio);
  }, []);

  const requestEditorMode = useCallback(
    (mode: EditorMode) => {
      if (mode === editorMode) return;
      if (mode === "hybrid") {
        // source → hybrid: roundtrip guard (must not change disk).
        const result = hybridRoundtrip(state.activeContent);
        if (!result.ok) {
          setModeRisks(result.risks);
          return;
        }
        setEditorMode("hybrid");
        return;
      }
      // hybrid → source: always allowed; content already in store.
      setEditorMode("source");
    },
    [editorMode, state.activeContent],
  );

  const onHybridToolbar = useCallback(
    (c: HybridToolbarCommand) => {
      const next = applyToolbarMarkdown(state.activeContent, c);
      store.onContentChange(next);
    },
    [state.activeContent, store],
  );

  const startExportHtml = useCallback(() => {
    if (!state.activeId) return;
    void exporter.openExport(
      "html",
      state.activeId,
      state.activeContent,
      activeSession?.displayName ?? "document",
    );
  }, [activeSession?.displayName, exporter, state.activeContent, state.activeId]);

  const startExportPdf = useCallback(() => {
    if (!state.activeId) return;
    void exporter.openExport(
      "pdf",
      state.activeId,
      state.activeContent,
      activeSession?.displayName ?? "document",
    );
  }, [activeSession?.displayName, exporter, state.activeContent, state.activeId]);

  const startExportPandoc = useCallback(
    async (format: "docx" | "epub" | "latex") => {
      if (!state.activeId) return;
      const base = (activeSession?.displayName ?? "document").replace(
        /\.(md|markdown|mdx|mkd)$/i,
        "",
      );
      const ext = format === "latex" ? "tex" : format;
      try {
        const path = await cmd.showPandocExportDialog(format, `${base}.${ext}`);
        if (!path) return;
        const result = await cmd.exportWithPandoc({
          sessionId: state.activeId,
          outputPath: path,
          format,
          markdown: state.activeContent,
        });
        window.alert(`Exported ${format.toUpperCase()} (${result.bytes} bytes)\n${result.outputPath}`);
      } catch (e) {
        const err = cmd.toCoreError(e);
        window.alert(err.message);
      }
    },
    [activeSession?.displayName, state.activeContent, state.activeId],
  );

  const showLicenses = useCallback(async () => {
    try {
      const text = await cmd.getThirdPartyNotices();
      setLicensesText(text);
    } catch (e) {
      const err = cmd.toCoreError(e);
      setLicensesText(`Could not load notices: ${err.message}`);
    }
  }, []);

  const pendingCloseName = state.pendingClose
    ? state.sessions.find((s) => s.id === state.pendingClose!.sessionId)?.displayName ?? "document"
    : "";

  let previewLabel: string | null = null;
  if (previewEnabled) {
    if (preview.status === "pending") previewLabel = "Preview: rendering…";
    else if (preview.status === "ready" && preview.renderMs != null) {
      previewLabel = `Preview: ${preview.renderMs} ms`;
    } else if (preview.status === "error") previewLabel = "Preview: error";
    else if (preview.status === "degraded") previewLabel = "Preview: paused";
  }

  const showEditor = layout === "source" || layout === "split";
  const showPreview = layout === "preview" || layout === "split";

  return (
    <div className="appshell">
      <a className="skip-link" href="#main-editor">
        Skip to editor
      </a>
      <TabBar
        sessions={state.sessions}
        busy={state.busy || exporter.state.busy}
        layout={layout}
        editorMode={editorMode}
        hasActive={!!activeSession}
        onLayoutChange={setLayout}
        onEditorModeChange={requestEditorMode}
        onActivate={(id) => void store.activate(id)}
        onClose={(id) => void store.requestClose(id)}
        onNew={() => void store.newDocument()}
        onOpen={() => void store.openViaDialog()}
        onSave={() => void store.saveActive()}
        onExportHtml={startExportHtml}
        onExportPdf={startExportPdf}
        onExportPandoc={(f) => void startExportPandoc(f)}
        onSettings={() => setSettingsOpen(true)}
        onLicenses={() => void showLicenses()}
      />

      {showEditor && editorMode === "hybrid" && (
        <HybridToolbar disabled={!activeSession || state.activeReadOnly} onCommand={onHybridToolbar} />
      )}

      <main
        id="main-editor"
        className={`appshell__main ${layout === "split" ? "appshell__main--split" : ""}`}
        tabIndex={-1}
      >
        {showEditor && editorMode === "source" && (
          <EditorPane
            value={state.activeContent}
            readOnly={state.activeReadOnly}
            disabled={!activeSession}
            onChange={(v) => store.onContentChange(v)}
            scrollRatio={editorScroll}
            onScrollRatio={onEditorScroll}
          />
        )}
        {showEditor && editorMode === "hybrid" && (
          <HybridEditor
            value={state.activeContent}
            readOnly={state.activeReadOnly}
            disabled={!activeSession}
            documentKey={state.activeId ?? "none"}
            onChange={(v) => store.onContentChange(v)}
          />
        )}
        {layout === "split" && <div className="appshell__divider" aria-hidden />}
        {showPreview && (
          <PreviewPane
            html={preview.html}
            toc={preview.toc}
            diagnostics={preview.diagnostics}
            status={preview.status}
            error={preview.error}
            renderMs={preview.renderMs}
            sessionId={state.activeId}
            filePath={activeSession?.filePath ?? null}
            degraded={preview.degraded}
            onRequestPreview={preview.requestManualPreview}
            scrollRatio={previewScroll}
            onScrollRatio={onPreviewScroll}
            scrollToId={tocJump}
            onTocNavigate={(id) => setTocJump(id)}
          />
        )}
      </main>

      <StatusBar
        encoding={state.activeEncoding}
        lineEnding={state.activeLineEnding}
        readOnly={state.activeReadOnly}
        dirty={activeSession?.dirty ?? false}
        charCount={state.activeContent.length}
        busy={state.busy}
        previewLabel={
          previewLabel
            ? `${previewLabel} · ${editorMode === "hybrid" ? "Hybrid" : "Source"}`
            : editorMode === "hybrid"
              ? "Hybrid"
              : null
        }
        reduced={preview.reduced}
        degraded={preview.degraded}
      />

      {state.notices.length > 0 && (
        <div className="notices" role="status">
          {state.notices.map((n) => (
            <div key={n.id} className={`notice notice--${n.level}`}>
              <span className="notice__msg">{n.message}</span>
              <button
                type="button"
                className="notice__close"
                aria-label="Dismiss"
                onClick={() => store.dismissNotice(n.id)}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}

      {state.pendingClose && (
        <ConfirmClose
          displayName={pendingCloseName}
          onChoose={(choice) => void store.confirmClose(state.pendingClose!.sessionId, choice)}
        />
      )}
      {state.pendingExternal && (
        <ExternalChangePrompt
          displayName={state.pendingExternal.displayName}
          onChoose={(choice) => void store.resolveExternal(choice)}
        />
      )}
      {recovery.pending.length > 0 && (
        <RecoveryPrompt
          entries={recovery.pending}
          onRestore={(key) => void handleRestore(key)}
          onDiscardOne={(key) => void recovery.discardOne(key)}
          onDiscardAll={() => void handleRecoveryDiscardAll()}
        />
      )}
      <DropZone armed={dragArmed} onDropPaths={() => {}} />

      {exporter.state.open && (
        <ExportDialog
          format={exporter.state.format}
          displayName={activeSession?.displayName ?? "document"}
          browserAvailable={exporter.state.browserAvailable}
          browserName={exporter.state.browserName}
          busy={exporter.state.busy}
          progress={exporter.state.progress}
          progressMessage={exporter.state.progressMessage}
          error={exporter.state.error}
          onConfirm={(opts) => void exporter.confirm(opts)}
          onCancel={exporter.close}
          onAbort={() => void exporter.abort()}
        />
      )}
      {licensesText != null && (
        <LicensesDialog text={licensesText} onClose={() => setLicensesText(null)} />
      )}
      {modeRisks && (
        <ModeSwitchWarning risks={modeRisks} onStaySource={() => setModeRisks(null)} />
      )}
      {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} />}
    </div>
  );
}
