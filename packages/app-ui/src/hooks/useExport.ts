/**
 * M3 export controller: dialog state, progress subscription, HTML/PDF invoke,
 * cancel, and browser probe.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import type {
  ExportFormat,
  ExportCommandResult,
  PdfPageOptions,
} from "@litemark/shared-protocol";
import * as cmd from "../services/tauriCommands";
import type { ExportDialogResult } from "../components/ExportDialog";

export interface ExportState {
  open: boolean;
  format: ExportFormat;
  busy: boolean;
  progress: number | null;
  progressMessage: string | null;
  error: string | null;
  browserAvailable: boolean | null;
  browserName: string | null;
  lastResult: ExportCommandResult | null;
}

const initial: ExportState = {
  open: false,
  format: "html",
  busy: false,
  progress: null,
  progressMessage: null,
  error: null,
  browserAvailable: null,
  browserName: null,
  lastResult: null,
};

export function useExport() {
  const [state, setState] = useState<ExportState>(initial);
  const jobIdRef = useRef<string | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const markdownRef = useRef<string>("");
  const displayNameRef = useRef<string>("document");

  // Progress events
  useEffect(() => {
    const unlisten = cmd.onExportProgress((payload) => {
      if (jobIdRef.current && payload.jobId !== jobIdRef.current) return;
      setState((s) => ({
        ...s,
        progress: payload.progress,
        progressMessage: payload.message ?? payload.stage ?? s.progressMessage,
      }));
    });
    return () => {
      void unlisten.then((u) => u());
    };
  }, []);

  const openExport = useCallback(
    async (format: ExportFormat, sessionId: string, markdown: string, displayName: string) => {
      sessionIdRef.current = sessionId;
      markdownRef.current = markdown;
      displayNameRef.current = displayName;
      setState({
        ...initial,
        open: true,
        format,
      });
      if (format === "pdf") {
        try {
          const tools = await cmd.probeExportTools();
          setState((s) => ({
            ...s,
            browserAvailable: tools.browser.available,
            browserName: tools.browser.available
              ? `${tools.browser.name}${tools.browser.path ? ` (${tools.browser.path})` : ""}`
              : tools.browser.name,
          }));
        } catch {
          setState((s) => ({
            ...s,
            browserAvailable: null,
            browserName: null,
          }));
        }
      }
    },
    [],
  );

  const close = useCallback(() => {
    if (state.busy) return;
    setState(initial);
    jobIdRef.current = null;
  }, [state.busy]);

  const abort = useCallback(async () => {
    const id = jobIdRef.current;
    if (id) {
      try {
        await cmd.cancelExport(id);
      } catch {
        /* best-effort */
      }
    }
    setState((s) => ({
      ...s,
      busy: false,
      progress: null,
      progressMessage: null,
      error: "Export cancelled",
    }));
    jobIdRef.current = null;
  }, []);

  const confirm = useCallback(async (opts: ExportDialogResult) => {
    const sessionId = sessionIdRef.current;
    if (!sessionId) return;

    const baseName = (displayNameRef.current || "document").replace(/\.(md|markdown|mdx|mkd)$/i, "");
    const defaultName =
      opts.format === "html" ? `${baseName}.html` : `${baseName}.pdf`;

    let outputPath: string | null = null;
    try {
      outputPath =
        opts.format === "html"
          ? await cmd.showHtmlExportDialog(defaultName)
          : await cmd.showPdfExportDialog(defaultName);
    } catch (e) {
      const err = cmd.toCoreError(e);
      setState((s) => ({ ...s, error: err.message }));
      return;
    }
    if (!outputPath) return; // user cancelled dialog

    const jobId = crypto.randomUUID();
    jobIdRef.current = jobId;
    setState((s) => ({
      ...s,
      busy: true,
      progress: 0,
      progressMessage: "Starting…",
      error: null,
      lastResult: null,
    }));

    try {
      let result: ExportCommandResult;
      if (opts.format === "html") {
        result = await cmd.exportHtml({
          sessionId,
          outputPath,
          markdown: markdownRef.current,
          offline: opts.offline,
          jobId,
        });
      } else {
        const page: Partial<PdfPageOptions> = { ...opts.page };
        result = await cmd.exportPdf({
          sessionId,
          outputPath,
          markdown: markdownRef.current,
          page,
          jobId,
        });
      }
      setState((s) => ({
        ...s,
        busy: false,
        progress: 1,
        progressMessage: `Saved ${formatBytes(result.bytes)}`,
        lastResult: result,
        error: null,
      }));
      jobIdRef.current = null;
    } catch (e) {
      const err = cmd.toCoreError(e);
      let message = err.message;
      if (err.code === "BROWSER_NOT_FOUND") {
        message =
          "No Edge or Chrome found. Install Microsoft Edge and try again, or set a browser path.";
      } else if (err.code === "EXPORT_CANCELLED") {
        message = "Export cancelled";
      }
      setState((s) => ({
        ...s,
        busy: false,
        progress: null,
        progressMessage: null,
        error: message,
      }));
      jobIdRef.current = null;
    }
  }, []);

  return {
    state,
    displayName: displayNameRef.current,
    openExport,
    close,
    abort,
    confirm,
  };
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MiB`;
}

export type ExportController = ReturnType<typeof useExport>;
