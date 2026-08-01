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
import { useT } from "../i18n/I18nProvider";
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

/** Strip markdown extension; keep a safe export stem. */
function exportBaseName(displayName: string, filePath: string | null): string {
  // Prefer path basename when available (matches double-clicked file on disk).
  let name = displayName || "document";
  if (filePath) {
    const slash = Math.max(filePath.lastIndexOf("\\"), filePath.lastIndexOf("/"));
    const base = slash >= 0 ? filePath.slice(slash + 1) : filePath;
    if (base.trim()) name = base;
  }
  const stripped = name.replace(/\.(md|markdown|mdx|mkd|mkdn|mdown)$/i, "");
  return stripped.trim() || "document";
}

function parentDir(filePath: string | null): string | null {
  if (!filePath) return null;
  const slash = Math.max(filePath.lastIndexOf("\\"), filePath.lastIndexOf("/"));
  if (slash <= 0) return null;
  return filePath.slice(0, slash);
}

export function useExport() {
  const t = useT();
  const [state, setState] = useState<ExportState>(initial);
  const jobIdRef = useRef<string | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const markdownRef = useRef<string>("");
  const displayNameRef = useRef<string>("document");
  const filePathRef = useRef<string | null>(null);

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
    async (
      format: ExportFormat,
      sessionId: string,
      markdown: string,
      displayName: string,
      filePath?: string | null,
    ) => {
      sessionIdRef.current = sessionId;
      markdownRef.current = markdown;
      displayNameRef.current = displayName;
      filePathRef.current = filePath ?? null;
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
      error: t("export.cancelled"),
    }));
    jobIdRef.current = null;
  }, [t]);

  const confirm = useCallback(
    async (opts: ExportDialogResult) => {
      const sessionId = sessionIdRef.current;
      if (!sessionId) return;

      // Default name = source stem + .pdf/.html (user can still change path).
      const baseName = exportBaseName(displayNameRef.current, filePathRef.current);
      const defaultName =
        opts.format === "html" ? `${baseName}.html` : `${baseName}.pdf`;
      // Prefer the document's folder; fall back to last export dir inside dialog helpers.
      const preferredDir = parentDir(filePathRef.current);

      let outputPath: string | null = null;
      try {
        outputPath =
          opts.format === "html"
            ? await cmd.showHtmlExportDialog(defaultName, preferredDir)
            : await cmd.showPdfExportDialog(defaultName, preferredDir);
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
        progressMessage: t("export.starting"),
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
          progressMessage: t("export.savedBytes", { size: formatBytes(result.bytes) }),
          lastResult: result,
          error: null,
        }));
        jobIdRef.current = null;
      } catch (e) {
        const err = cmd.toCoreError(e);
        let message = err.message;
        if (err.code === "BROWSER_NOT_FOUND") {
          message = t("export.browserNotFound");
        } else if (err.code === "EXPORT_CANCELLED") {
          message = t("export.cancelled");
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
    },
    [t],
  );

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
