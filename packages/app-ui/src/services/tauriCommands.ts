/**
 * Typed wrappers around Tauri commands. The webview never touches the
 * filesystem directly — every file operation goes through these Rust commands
 * (see docs/security.md). Types come from shared-protocol, which is the single
 * source of truth for the wire contract.
 *
 * M1: document lifecycle, dialogs, recent, recovery.
 * M2: render_markdown, release_render_session, open_external_url, assets.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppSettings,
  DocumentSession,
  ExportCommandResult,
  ExportHtmlCommandParams,
  ExportPdfCommandParams,
  OpenDialogOptions,
  OptionalToolsStatus,
  PandocExportParams,
  PandocExportResult,
  PandocStatus,
  ProbeExportToolsResult,
  RecoveryEntry,
  RecentEntry,
  RenderMarkdownParams,
  RenderMarkdownResult,
  ResolveAssetResult,
  SaveDialogOptions,
  SessionSummary,
  UpdateStatus,
} from "@litemark/shared-protocol";
import {
  DOCX_EXPORT_FILTERS,
  EPUB_EXPORT_FILTERS,
  HTML_EXPORT_FILTERS,
  LATEX_EXPORT_FILTERS,
  MARKDOWN_FILTERS,
  PDF_EXPORT_FILTERS,
} from "@litemark/shared-protocol";

/** A structured error echoed back from the Rust core (SidecarError shape). */
export interface CoreError {
  code: string;
  message: string;
  details?: unknown;
}

/** Extract a CoreError from an invoke rejection, or wrap an unknown value. */
export function toCoreError(value: unknown): CoreError {
  if (typeof value === "string") {
    // Tauri serializes the error string; it may be JSON or a plain message.
    try {
      const parsed = JSON.parse(value);
      if (parsed && typeof parsed.code === "string") {
        return parsed as CoreError;
      }
    } catch {
      return { code: "UNKNOWN", message: value };
    }
  }
  if (value && typeof value === "object" && "code" in value) {
    return value as CoreError;
  }
  return { code: "UNKNOWN", message: String(value) };
}

// --- Documents -------------------------------------------------------------

export function newDocument(): Promise<string> {
  return invoke<string>("new_document");
}

export function openFile(path: string): Promise<string> {
  return invoke<string>("open_file", { path });
}

export function saveDocument(sessionId: string): Promise<SaveResultLike> {
  return invoke<SaveResultLike>("save_document", { sessionId });
}

export function saveAsDocument(sessionId: string, path: string): Promise<SaveResultLike> {
  return invoke<SaveResultLike>("save_as_document", { sessionId, path });
}

export function setDocumentContent(sessionId: string, content: string): Promise<void> {
  return invoke<void>("set_document_content", { sessionId, content });
}

export function getDocument(sessionId: string): Promise<DocumentSession> {
  return invoke<DocumentSession>("get_document", { sessionId });
}

export function listDocuments(): Promise<SessionSummary[]> {
  return invoke<SessionSummary[]>("list_documents");
}

export function listDirtyDocuments(): Promise<string[]> {
  return invoke<string[]>("list_dirty_documents");
}

export function closeDocument(sessionId: string): Promise<boolean> {
  return invoke<boolean>("close_document", { sessionId });
}

export function setActiveDocument(sessionId: string | null): Promise<void> {
  return invoke<void>("set_active_document", { sessionId });
}

export function activeDocument(): Promise<string | null> {
  return invoke<string | null>("active_document");
}

export function checkExternalChange(sessionId: string): Promise<boolean> {
  return invoke<boolean>("check_external_change", { sessionId });
}

// --- File dialogs ----------------------------------------------------------

export function showOpenDialog(): Promise<string | null> {
  const options: OpenDialogOptions = {
    title: "Open Markdown",
    filters: MARKDOWN_FILTERS,
  };
  return invoke<string | null>("show_open_dialog", { options });
}

export function showSaveDialog(
  defaultFileName?: string,
  extra?: Partial<SaveDialogOptions>,
): Promise<string | null> {
  const options: SaveDialogOptions = {
    title: extra?.title ?? "Save Markdown",
    filters: extra?.filters ?? MARKDOWN_FILTERS,
    defaultFileName: defaultFileName ?? extra?.defaultFileName,
    defaultDirectory: extra?.defaultDirectory,
  };
  return invoke<string | null>("show_save_dialog", { options });
}

// --- Recent files ----------------------------------------------------------

export function getRecentFiles(): Promise<RecentEntry[]> {
  return invoke<RecentEntry[]>("get_recent_files");
}

export function setRecentPinned(path: string, pinned: boolean): Promise<void> {
  return invoke<void>("set_recent_pinned", { path, pinned });
}

export function clearRecentFiles(): Promise<void> {
  return invoke<void>("clear_recent_files");
}

// --- Recovery --------------------------------------------------------------

export function getPendingRecovery(): Promise<RecoveryEntry[]> {
  return invoke<RecoveryEntry[]>("get_pending_recovery");
}

export function restoreRecoverySnapshot(recoveryKey: string): Promise<string> {
  return invoke<string>("restore_recovery_snapshot", { recoveryKey });
}

export function discardRecoverySnapshot(recoveryKey: string): Promise<void> {
  return invoke<void>("discard_recovery_snapshot_cmd", { recoveryKey });
}

export function discardAllRecovery(): Promise<number> {
  return invoke<number>("discard_all_recovery");
}

// --- Events ----------------------------------------------------------------

/**
 * Subscribe to the `open-files` event emitted by the single-instance plugin
 * when a second instance forwards file-path arguments (e.g. double-clicking a
 * `.md` file while LiteMark is already running). Returns an unlisten handle.
 */
export function onOpenFiles(
  handler: (files: string[]) => void,
): Promise<UnlistenFn> {
  return listen<string[]>("open-files", (event) => {
    handler(event.payload ?? []);
  });
}

/**
 * Paths passed on cold start (file association / CLI). Consumed once; empty
 * afterwards. Pair with `onOpenFiles` for second-instance opens.
 * Also kicks sidecar warm in Rust when paths are present (P1-3).
 */
export function takePendingCliFiles(): Promise<string[]> {
  return invoke<string[]>("take_pending_cli_files");
}

/**
 * Ensure the Node/crossnote render sidecar is running (spawn + ping).
 * Fire-and-forget on app mount so the first preview is not cold (P1-1 / P1-3).
 */
export function warmSidecar(): Promise<boolean> {
  return invoke<boolean>("warm_sidecar");
}

/** Ping the sidecar (also spawns it if needed). Returns version info. */
export function pingSidecar(sentAt?: string): Promise<{
  version: string;
  crossnoteVersion: string;
  receivedAt?: string;
}> {
  return invoke("ping_sidecar", { sentAt: sentAt ?? null });
}

// --- M2 render / links / assets --------------------------------------------

/** Render in-memory Markdown via the sidecar (never writes the source file). */
export function renderMarkdown(
  params: RenderMarkdownParams,
): Promise<RenderMarkdownResult> {
  return invoke<RenderMarkdownResult>("render_markdown", { params });
}

/** Best-effort release of the crossnote session after a tab closes. */
export function releaseRenderSession(sessionId: string): Promise<void> {
  return invoke<void>("release_render_session", { sessionId });
}

/** Open http(s)/mailto/tel with the OS default handler. */
export function openExternalUrl(url: string): Promise<void> {
  return invoke<void>("open_external_url", { url });
}

/**
 * Resolve a relative asset under the document directory and return a
 * `lmlocal://` URL the preview may load.
 */
export function resolveDocumentAsset(
  sessionId: string,
  href: string,
): Promise<ResolveAssetResult> {
  return invoke<ResolveAssetResult>("resolve_document_asset", { sessionId, href });
}

// --- M3 export -------------------------------------------------------------

export function exportHtml(params: ExportHtmlCommandParams): Promise<ExportCommandResult> {
  return invoke<ExportCommandResult>("export_html", { params });
}

export function exportPdf(params: ExportPdfCommandParams): Promise<ExportCommandResult> {
  return invoke<ExportCommandResult>("export_pdf", { params });
}

export function cancelExport(jobId: string): Promise<boolean> {
  return invoke<boolean>("cancel_export", { jobId });
}

export function probeExportTools(): Promise<ProbeExportToolsResult> {
  return invoke<ProbeExportToolsResult>("probe_export_tools");
}

export function getLastExportDir(): Promise<string | null> {
  return invoke<string | null>("get_last_export_dir");
}

export function setLastExportDir(path: string): Promise<void> {
  return invoke<void>("set_last_export_dir", { path });
}

export function getThirdPartyNotices(): Promise<string> {
  return invoke<string>("get_third_party_notices");
}

/** Show a save dialog pre-filtered for HTML export. */
export async function showHtmlExportDialog(
  defaultFileName: string,
  preferredDirectory?: string | null,
): Promise<string | null> {
  const last = await getLastExportDir().catch(() => null);
  return showSaveDialog(defaultFileName, {
    title: "Export HTML",
    filters: HTML_EXPORT_FILTERS,
    defaultFileName,
    defaultDirectory: preferredDirectory || last || undefined,
  });
}

/** Show a save dialog pre-filtered for PDF export. */
export async function showPdfExportDialog(
  defaultFileName: string,
  preferredDirectory?: string | null,
): Promise<string | null> {
  const last = await getLastExportDir().catch(() => null);
  return showSaveDialog(defaultFileName, {
    title: "Export PDF",
    filters: PDF_EXPORT_FILTERS,
    defaultFileName,
    defaultDirectory: preferredDirectory || last || undefined,
  });
}

/**
 * Subscribe to export progress events forwarded from the sidecar.
 */
export function onExportProgress(
  handler: (payload: {
    jobId: string;
    stage: string;
    progress: number;
    message?: string | null;
  }) => void,
): Promise<UnlistenFn> {
  return listen("export-progress", (event) => {
    handler(event.payload as {
      jobId: string;
      stage: string;
      progress: number;
      message?: string | null;
    });
  });
}

// --- M5 settings / pandoc --------------------------------------------------

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function setSettings(settings: AppSettings): Promise<void> {
  return invoke<void>("set_settings", { settings });
}

export function trustWorkspace(path: string): Promise<AppSettings> {
  return invoke<AppSettings>("trust_workspace", { path });
}

export function untrustWorkspace(path: string): Promise<AppSettings> {
  return invoke<AppSettings>("untrust_workspace", { path });
}

export function isPathTrusted(path: string): Promise<boolean> {
  return invoke<boolean>("is_path_trusted", { path });
}

export function getCustomCss(): Promise<string | null> {
  return invoke<string | null>("get_custom_css");
}

export function probePandoc(): Promise<PandocStatus> {
  return invoke<PandocStatus>("probe_pandoc");
}

export function probeOptionalTools(): Promise<OptionalToolsStatus> {
  return invoke<OptionalToolsStatus>("probe_optional_tools");
}

export function exportWithPandoc(params: PandocExportParams): Promise<PandocExportResult> {
  return invoke<PandocExportResult>("export_with_pandoc", { params });
}

export async function showPandocExportDialog(
  format: "docx" | "epub" | "latex",
  defaultFileName: string,
  preferredDirectory?: string | null,
): Promise<string | null> {
  const last = await getLastExportDir().catch(() => null);
  const filters =
    format === "docx"
      ? DOCX_EXPORT_FILTERS
      : format === "epub"
        ? EPUB_EXPORT_FILTERS
        : LATEX_EXPORT_FILTERS;
  const title =
    format === "docx" ? "Export DOCX" : format === "epub" ? "Export EPUB" : "Export LaTeX";
  return showSaveDialog(defaultFileName, {
    title,
    filters,
    defaultFileName,
    defaultDirectory: preferredDirectory || last || undefined,
  });
}

// --- M6 --------------------------------------------------------------------

export function exportCrashLog(): Promise<string> {
  return invoke<string>("export_crash_log");
}

export function getUpdateStatus(): Promise<UpdateStatus> {
  return invoke<UpdateStatus>("get_update_status");
}

interface SaveResultLike {
  mtimeMs: number;
  contentHash: string;
}
