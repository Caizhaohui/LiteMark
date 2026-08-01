/**
 * M2 live-preview controller.
 *
 * - Debounces editor content (250 ms normal / 750 ms for 1–5 MiB docs).
 * - P1-2: first preview after open / session switch uses 0 debounce so
 *   double-click open is not delayed an extra 250 ms.
 * - Sends `render_markdown` with a monotonic revision; discards stale results.
 * - Large-file degradation: > 5 MiB defaults to source-only until the user
 *   requests a manual preview (§8.3).
 * - Never writes the source file (in-memory markdown only).
 */

import { useCallback, useEffect, useRef, useState } from "react";
import {
  PREVIEW_THRESHOLDS,
  type Diagnostic,
  type TocEntry,
} from "@litemark/shared-protocol";
import * as cmd from "../services/tauriCommands";
import { sanitizePreviewHtml } from "../services/sanitize";

export type PreviewStatus = "idle" | "pending" | "ready" | "error" | "degraded";

export interface PreviewState {
  html: string;
  toc: TocEntry[];
  diagnostics: Diagnostic[];
  renderMs: number | null;
  status: PreviewStatus;
  error: string | null;
  /** Byte size of the last rendered content. */
  contentBytes: number;
  /** True when content exceeds the full real-time threshold. */
  reduced: boolean;
  /** True when content exceeds 5 MiB and auto-preview is off. */
  degraded: boolean;
}

const empty: PreviewState = {
  html: "",
  toc: [],
  diagnostics: [],
  renderMs: null,
  status: "idle",
  error: null,
  contentBytes: 0,
  reduced: false,
  degraded: false,
};

function byteLength(text: string): number {
  return new TextEncoder().encode(text).length;
}

export interface UsePreviewOptions {
  sessionId: string | null;
  content: string;
  /** When false, skip auto-render (e.g. source-only layout). */
  enabled: boolean;
  /** Bump to force a manual render even in degraded mode. */
  manualToken?: number;
}

export function usePreview({
  sessionId,
  content,
  enabled,
  manualToken = 0,
}: UsePreviewOptions): PreviewState & { requestManualPreview: () => void } {
  const [state, setState] = useState<PreviewState>(empty);
  const [manualBump, setManualBump] = useState(0);
  const revisionRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSessionRef = useRef<string | null>(null);
  /** P1-2: skip debounce for the first scheduled preview of each session. */
  const firstPreviewForSessionRef = useRef(true);

  const requestManualPreview = useCallback(() => {
    setManualBump((n) => n + 1);
  }, []);

  useEffect(() => {
    // Session changed: clear preview and treat next content as a "first open".
    if (sessionId !== lastSessionRef.current) {
      lastSessionRef.current = sessionId;
      revisionRef.current = 0;
      firstPreviewForSessionRef.current = true;
      setState(empty);
    }
  }, [sessionId]);

  useEffect(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }

    if (!sessionId || !enabled) {
      return;
    }

    const bytes = byteLength(content);
    const reduced = bytes >= PREVIEW_THRESHOLDS.fullPreviewBytes;
    const degraded = bytes > PREVIEW_THRESHOLDS.reducedPreviewBytes;

    // > 5 MiB: only render when the user explicitly requests it.
    if (degraded && manualBump === 0 && manualToken === 0) {
      setState((prev) => ({
        ...prev,
        contentBytes: bytes,
        reduced: true,
        degraded: true,
        status: "degraded",
        error: null,
      }));
      return;
    }

    // P1-2: open / tab switch / manual request → immediate; typing → debounce.
    const isFirstForSession = firstPreviewForSessionRef.current;
    const immediate =
      isFirstForSession || manualBump > 0 || manualToken > 0;
    if (isFirstForSession) {
      firstPreviewForSessionRef.current = false;
    }

    const delay = immediate
      ? PREVIEW_THRESHOLDS.openDebounceMs
      : reduced
        ? PREVIEW_THRESHOLDS.reducedDebounceMs
        : PREVIEW_THRESHOLDS.debounceMs;

    setState((prev) => ({
      ...prev,
      contentBytes: bytes,
      reduced,
      degraded,
      status: "pending",
      error: null,
    }));

    const revision = ++revisionRef.current;
    const markdownSnapshot = content;

    const run = () => {
      void (async () => {
        try {
          const options = reduced
            ? { mathRenderer: "KaTeX" as const, theme: "github-light.css" }
            : { mathRenderer: "KaTeX" as const, theme: "github-light.css" };

          const result = await cmd.renderMarkdown({
            sessionId,
            markdown: markdownSnapshot,
            revision,
            options,
          });

          // Stale result — a newer edit already superseded this render.
          if (result.revision !== revisionRef.current) {
            return;
          }

          const clean = sanitizePreviewHtml(result.html);
          setState({
            html: clean,
            toc: result.toc ?? [],
            diagnostics: result.diagnostics ?? [],
            renderMs: result.renderMs,
            status: "ready",
            error: null,
            contentBytes: bytes,
            reduced,
            degraded,
          });
        } catch (e) {
          if (revision !== revisionRef.current) return;
          const err = cmd.toCoreError(e);
          setState((prev) => ({
            ...prev,
            status: "error",
            error: err.message,
            contentBytes: bytes,
            reduced,
            degraded,
          }));
        }
      })();
    };

    if (delay <= 0) {
      run();
    } else {
      timerRef.current = setTimeout(run, delay);
    }

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [sessionId, content, enabled, manualBump, manualToken]);

  return { ...state, requestManualPreview };
}
