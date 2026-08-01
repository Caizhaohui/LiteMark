/**
 * LiteMark shared IPC protocol.
 *
 * This is the single source of truth for the JSON Lines protocol spoken over
 * stdin/stdout between the Rust application core and the Node render sidecar.
 *
 * Rules (see DEVELOPMENT_PLAN.md §5.3):
 * - Every message is exactly one JSON object per line.
 * - stdout is reserved for protocol messages; all logging MUST go to stderr.
 * - The method set is a static enum; there is no `exec`/`shell`/`runCommand`.
 */

// ----------------------------------------------------------------------------
// Request envelope
// ----------------------------------------------------------------------------

/**
 * All allowed sidecar methods. Kept as a const tuple so it can be iterated to
 * build a runtime whitelist and derived into a union type.
 *
 * The plan explicitly forbids generic `exec`/`shell`/`runCommand` entries.
 */
export const SIDECAR_METHODS = [
  "ping",
  "getCapabilities",
  "createSession",
  "closeSession",
  "render",
  "exportHtml",
  "exportPdf",
  "cancelJob",
  "probeExternalTools",
  "shutdown",
] as const;

export type SidecarMethod = (typeof SIDECAR_METHODS)[number];

/**
 * A canonical request id. Stringified to survive JSON round-trips regardless
 * of whether the caller prefers numbers or UUIDs.
 */
export type RequestId = string;

/**
 * Base shape every request shares. Concrete method params live in the
 * `params` union keyed by `method`.
 */
export interface SidecarRequestBase<M extends SidecarMethod, P> {
  id: RequestId;
  method: M;
  params: P;
}

// ----------------------------------------------------------------------------
// Method params
// ----------------------------------------------------------------------------

export interface PingParams {
  /** Optional caller timestamp; echoed back in the result for latency checks. */
  sentAt?: string;
}

export interface CapabilitiesParams {
  /** Nothing for now; reserved for future capability negotiation. */
}

export interface CreateSessionParams {
  /** A caller-provided opaque id. The sidecar treats it as an opaque key. */
  sessionId: string;
  /**
   * Logical path of the document on disk. Used ONLY for relative resource,
   * import and link resolution. The sidecar MUST NOT read or write this file.
   */
  logicalFilePath?: string | null;
  /**
   * Optional notebook/workspace root. Restricts where relative resources may
   * resolve. (See §5.4 / ADR 0005, future milestone.)
   */
  notebookPath?: string | null;
}

export interface CloseSessionParams {
  sessionId: string;
}

/** Safe crossnote defaults. Mirrors DEVELOPMENT_PLAN.md §8.1. */
export interface RenderOptions {
  theme?: string;
  codeBlockTheme?: string;
  mathRenderer?: "KaTeX" | "MathJax" | "None";
  /** ALWAYS defaults to false. Never accept `true` from untrusted input. */
  trusted?: boolean;
  /** ALWAYS defaults to false. */
  enableScriptExecution?: boolean;
}

export interface RenderParams {
  sessionId: string;
  /** Raw markdown text from the editor memory. Never written to disk. */
  markdown: string;
  /**
   * Logical path for resource resolution only. The sidecar does not touch the
   * file itself (see §5.4 "in-memory rendering").
   */
  logicalFilePath?: string | null;
  /** Monotonic document revision; stale results are discarded by the caller. */
  revision?: number;
  options?: RenderOptions;
}

export interface ExportHtmlParams {
  sessionId: string;
  markdown: string;
  logicalFilePath?: string | null;
  /**
   * Absolute output path, validated by the Rust core before forwarding.
   * The sidecar writes only to this path.
   */
  outputPath: string;
  /** When true, inline CSS/fonts/images so the file is fully offline. */
  offline?: boolean;
  /** Correlates cancelJob / exportProgress events. */
  jobId?: string;
  options?: RenderOptions;
}

export interface PdfPageOptions {
  pageSize: "A4" | "Letter" | "Legal";
  landscape: boolean;
  marginTopMm: number;
  marginRightMm: number;
  marginBottomMm: number;
  marginLeftMm: number;
  printBackground: boolean;
  displayHeaderFooter: boolean;
}

/** Default PDF page options (§9.2). */
export const DEFAULT_PDF_PAGE_OPTIONS: PdfPageOptions = {
  pageSize: "A4",
  landscape: false,
  marginTopMm: 10,
  marginRightMm: 10,
  marginBottomMm: 10,
  marginLeftMm: 10,
  printBackground: true,
  displayHeaderFooter: false,
};

export interface ExportPdfParams {
  sessionId: string;
  markdown: string;
  logicalFilePath?: string | null;
  /**
   * Absolute output path, validated by the Rust core before forwarding. The
   * sidecar rejects paths it has not been authorized to write.
   */
  outputPath: string;
  page?: Partial<PdfPageOptions>;
  /**
   * Optional explicit browser executable path. When omitted the sidecar probes
   * Edge then Chrome (see §9.2).
   */
  browserPath?: string | null;
  /** Correlates cancelJob / exportProgress events. */
  jobId?: string;
  options?: RenderOptions;
}

export interface CancelJobParams {
  jobId: string;
}

export interface ProbeExternalToolsParams {
  /** Nothing for now. */
}

// ----------------------------------------------------------------------------
// Concrete request types
// ----------------------------------------------------------------------------

export type SidecarRequest =
  | SidecarRequestBase<"ping", PingParams>
  | SidecarRequestBase<"getCapabilities", CapabilitiesParams>
  | SidecarRequestBase<"createSession", CreateSessionParams>
  | SidecarRequestBase<"closeSession", CloseSessionParams>
  | SidecarRequestBase<"render", RenderParams>
  | SidecarRequestBase<"exportHtml", ExportHtmlParams>
  | SidecarRequestBase<"exportPdf", ExportPdfParams>
  | SidecarRequestBase<"cancelJob", CancelJobParams>
  | SidecarRequestBase<"probeExternalTools", ProbeExternalToolsParams>
  | SidecarRequestBase<"shutdown", PingParams>;

// ----------------------------------------------------------------------------
// Results
// ----------------------------------------------------------------------------

export interface PingResult {
  version: string;
  crossnoteVersion: string;
  /** Echoed caller timestamp when provided. */
  receivedAt?: string;
}

export interface ExternalToolStatus {
  name: string;
  available: boolean;
  /** Discovered path or null. */
  path: string | null;
  version: string | null;
}

export interface CapabilitiesResult {
  /** The static set of methods this sidecar build implements. */
  methods: SidecarMethod[];
  crossnoteVersion: string;
  /** Safe defaults the sidecar will enforce regardless of caller wishes. */
  safeDefaults: {
    enableScriptExecution: false;
    enableHTML5Embed: false;
    protocolsWhiteList: string;
  };
  externalTools: ExternalToolStatus[];
}

export interface CreateSessionResult {
  sessionId: string;
  ok: true;
}

export interface CloseSessionResult {
  sessionId: string;
  ok: true;
}

/** A single table-of-contents entry. */
export interface TocEntry {
  level: number;
  text: string;
  /** Anchor id used in the rendered HTML. */
  id: string;
}

export interface Diagnostic {
  level: "info" | "warning" | "error";
  code: string;
  message: string;
  /** Optional 1-based source line when available. */
  line?: number | null;
}

export interface RenderResult {
  html: string;
  toc: TocEntry[];
  diagnostics: Diagnostic[];
  /** Wall-clock render time for observability. */
  renderMs: number;
}

export interface ExportHtmlResult {
  /** Path written by the sidecar. */
  outputPath: string;
  bytes: number;
}

export interface ExportPdfResult {
  outputPath: string;
  bytes: number;
}

export interface CancelJobResult {
  jobId: string;
  cancelled: boolean;
}

export interface ProbeExternalToolsResult {
  tools: ExternalToolStatus[];
}

/** Map a method to its success result type. */
export interface ResultByMethod {
  ping: PingResult;
  getCapabilities: CapabilitiesResult;
  createSession: CreateSessionResult;
  closeSession: CloseSessionResult;
  render: RenderResult;
  exportHtml: ExportHtmlResult;
  exportPdf: ExportPdfResult;
  cancelJob: CancelJobResult;
  probeExternalTools: ProbeExternalToolsResult;
  shutdown: PingResult;
}

// ----------------------------------------------------------------------------
// Response envelope
// ----------------------------------------------------------------------------

export interface SidecarSuccessResponse<M extends SidecarMethod> {
  id: RequestId;
  ok: true;
  result: ResultByMethod[M];
}

export interface SidecarErrorResponse {
  id: RequestId;
  ok: false;
  error: SidecarError;
}

export type SidecarResponse =
  | SidecarSuccessResponse<SidecarMethod>
  | SidecarErrorResponse;

// ----------------------------------------------------------------------------
// Events (asynchronous, server -> caller)
// ----------------------------------------------------------------------------

export type ExportProgressStage =
  | "preparing"
  | "rendering"
  | "writing"
  | "launching_browser"
  | "printing"
  | "finalizing";

export interface ExportProgressPayload {
  jobId: string;
  stage: ExportProgressStage;
  /** 0..1. */
  progress: number;
  /** Optional human-readable detail for the status bar / dialog. */
  message?: string;
}

export type SidecarEvent =
  | { event: "exportProgress"; payload: ExportProgressPayload }
  | { event: "ready"; payload: { version: string } }
  | { event: "log"; payload: { level: "info" | "warn" | "error"; message: string } };

// ----------------------------------------------------------------------------
// Error model (DEVELOPMENT_PLAN.md §14)
// ----------------------------------------------------------------------------

export const ERROR_CODES = [
  "FILE_NOT_FOUND",
  "FILE_PERMISSION_DENIED",
  "FILE_CHANGED_EXTERNALLY",
  "FILE_ENCODING_UNSUPPORTED",
  "SAVE_ATOMIC_REPLACE_FAILED",
  "SIDECAR_START_FAILED",
  "SIDECAR_CRASHED",
  "SIDECAR_TIMEOUT",
  "PROTOCOL_INVALID",
  "RENDER_FAILED",
  "RENDER_CANCELLED",
  "EXPORT_FAILED",
  "EXPORT_CANCELLED",
  "BROWSER_NOT_FOUND",
  "PANDOC_NOT_FOUND",
  "UNTRUSTED_OPERATION_BLOCKED",
  "PATH_NOT_AUTHORIZED",
  "ROUNDTRIP_DATA_LOSS_RISK",
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

export interface SidecarError {
  code: ErrorCode | string;
  message: string;
  /** Optional structured detail for diagnostics; never secrets. */
  details: unknown | null;
}

// ----------------------------------------------------------------------------
// Runtime helpers
// ----------------------------------------------------------------------------

/** Whitelist check used by the sidecar dispatcher. */
export function isKnownMethod(value: unknown): value is SidecarMethod {
  return typeof value === "string" && (SIDECAR_METHODS as readonly string[]).includes(value);
}

/** Minimal structural validation for an incoming JSON line. */
export function isSidecarRequest(value: unknown): value is SidecarRequest {
  if (typeof value !== "object" || value === null) return false;
  const r = value as Record<string, unknown>;
  return typeof r.id === "string" && isKnownMethod(r.method) && typeof r.params === "object";
}

/** Build a success envelope. */
export function ok<M extends SidecarMethod>(
  id: RequestId,
  result: ResultByMethod[M],
): SidecarSuccessResponse<M> {
  return { id, ok: true, result };
}

/** Build an error envelope. */
export function err(
  id: RequestId,
  error: SidecarError,
): SidecarErrorResponse {
  return { id, ok: false, error };
}

// ----------------------------------------------------------------------------
// M1 — Tauri command contracts (webview ⇄ Rust core)
// ----------------------------------------------------------------------------
//
// These types describe the commands exposed by the Rust core for document
// lifecycle, file dialogs, recent files, and recovery. Unlike the sidecar
// protocol above (Rust ⇄ Node), these run over Tauri's `invoke()` bridge. The
// Rust side serializes with camelCase keys (see DocumentSessionWire etc.), so
// these interfaces use camelCase to match the wire shape exactly.

/** The canonical M1 (and beyond) Tauri command names. */
export const TAURI_COMMANDS = [
  // M0
  "ping_sidecar",
  // M1 — documents
  "new_document",
  "open_file",
  "save_document",
  "save_as_document",
  "set_document_content",
  "get_document",
  "list_documents",
  "list_dirty_documents",
  "close_document",
  "set_active_document",
  "active_document",
  "check_external_change",
  "discard_recovery_snapshot",
  // M1 — native file dialogs
  "show_open_dialog",
  "show_save_dialog",
  "is_markdown_path",
  // M1 — recent files
  "get_recent_files",
  "set_recent_pinned",
  "clear_recent_files",
  // M1 — recovery
  "get_pending_recovery",
  "restore_recovery_snapshot",
  "discard_recovery_snapshot_cmd",
  "discard_all_recovery",
  // M2 — render / preview / links / local assets
  "render_markdown",
  "release_render_session",
  "open_external_url",
  "resolve_document_asset",
  // M3 — export
  "export_html",
  "export_pdf",
  "cancel_export",
  "probe_export_tools",
  "get_last_export_dir",
  "set_last_export_dir",
  "get_third_party_notices",
  // M5 — settings / pandoc / trust
  "get_settings",
  "set_settings",
  "trust_workspace",
  "untrust_workspace",
  "is_path_trusted",
  "get_custom_css",
  "probe_pandoc",
  "probe_optional_tools",
  "export_with_pandoc",
  // M6
  "export_crash_log",
  "get_update_status",
] as const;

export type TauriCommand = (typeof TAURI_COMMANDS)[number];

/** A single open document (§6.1). Mirrors the Rust `DocumentSessionWire`. */
export interface DocumentSession {
  id: string;
  filePath: string | null;
  displayName: string;
  content: string;
  savedContentHash: string;
  encoding: "utf-8" | "utf-8-bom";
  lineEnding: "lf" | "crlf";
  /** Derived from the content hash by Rust — the UI must not set this. */
  dirty: boolean;
  readOnly: boolean;
  mode: "source" | "hybrid" | "preview";
  revision: number;
  lastSavedRevision: number;
  externalMtimeMs: number | null;
  recoveryKey: string;
}

/** Tab metadata without content (for the tab bar). Mirrors SessionSummary. */
export interface SessionSummary {
  id: string;
  filePath: string | null;
  displayName: string;
  savedContentHash: string;
  encoding: string;
  lineEnding: string;
  dirty: boolean;
  readOnly: boolean;
  revision: number;
  externalMtimeMs: number | null;
  active: boolean;
}

/** Result of a save / save-as command. */
export interface SaveResult {
  mtimeMs: number;
  contentHash: string;
}

/** A filter row for a file dialog. */
export interface FileFilter {
  name: string;
  extensions: string[];
}

export interface OpenDialogOptions {
  title?: string;
  filters?: FileFilter[];
}

export interface SaveDialogOptions {
  title?: string;
  filters?: FileFilter[];
  defaultFileName?: string;
  /** Starting directory for the dialog (e.g. last export folder). */
  defaultDirectory?: string;
}

/** A recent-files entry. */
export interface RecentEntry {
  path: string;
  lastOpenedAt: string;
  pinned: boolean;
}

/** A pending crash-recovery snapshot. */
export interface RecoveryEntry {
  sessionId: string;
  originalPath: string | null;
  capturedAt: string;
  revision: number;
  content: string;
  recoveryKey: string;
}

/** Default markdown filter for the open/save dialogs. */
export const MARKDOWN_FILTERS: FileFilter[] = [
  { name: "Markdown", extensions: ["md", "markdown", "mdx", "mkd"] },
  { name: "All Files", extensions: ["*"] },
];

// ----------------------------------------------------------------------------
// M2 — render / preview contracts
// ----------------------------------------------------------------------------

/** Large-file thresholds (DEVELOPMENT_PLAN.md §8.3). Sizes in bytes. */
export const PREVIEW_THRESHOLDS = {
  /** Full real-time preview below this size. */
  fullPreviewBytes: 1 * 1024 * 1024,
  /** Reduced debounce / expensive diagrams off between full and this. */
  reducedPreviewBytes: 5 * 1024 * 1024,
  /** Default debounce for normal documents (ms). */
  debounceMs: 250,
  /** Debounce when 1–5 MiB (ms). */
  reducedDebounceMs: 750,
} as const;

/** Preview layout mode for the M2 UI (not the same as DocumentSession.mode). */
export type PreviewLayout = "source" | "split" | "preview";

/** Parameters for the `render_markdown` Tauri command. */
export interface RenderMarkdownParams {
  sessionId: string;
  /** Editor memory content; never written to the source file. */
  markdown: string;
  /**
   * Monotonic revision from the frontend. Echoed back so the UI can discard
   * stale responses (§8.2).
   */
  revision: number;
  options?: RenderOptions;
}

/** Result of `render_markdown`. */
export interface RenderMarkdownResult {
  html: string;
  toc: TocEntry[];
  diagnostics: Diagnostic[];
  renderMs: number;
  /** Echo of the request revision for stale-result filtering. */
  revision: number;
}

/** Result of `resolve_document_asset` — authorized absolute path only. */
export interface ResolveAssetResult {
  /** Absolute path the custom protocol may serve. */
  absolutePath: string;
  /** Custom-protocol URL for the webview (`lmlocal://…`). */
  url: string;
}

// ----------------------------------------------------------------------------
// M3 — export contracts (webview ⇄ Rust)
// ----------------------------------------------------------------------------

export type ExportFormat = "html" | "pdf";

/** HTML export options shown in the Export dialog. */
export interface HtmlExportUiOptions {
  /** Inline CSS/fonts/images for offline viewing (default true). */
  offline: boolean;
  theme?: string;
  codeBlockTheme?: string;
}

/** Parameters for the `export_html` Tauri command. */
export interface ExportHtmlCommandParams {
  sessionId: string;
  /** Absolute output path chosen by the user via the native save dialog. */
  outputPath: string;
  /** Optional override; when omitted Rust uses the session's in-memory content. */
  markdown?: string;
  offline?: boolean;
  options?: RenderOptions;
  jobId?: string;
}

/** Parameters for the `export_pdf` Tauri command. */
export interface ExportPdfCommandParams {
  sessionId: string;
  outputPath: string;
  markdown?: string;
  page?: Partial<PdfPageOptions>;
  browserPath?: string | null;
  options?: RenderOptions;
  jobId?: string;
}

/** Result of export_html / export_pdf Tauri commands. */
export interface ExportCommandResult {
  outputPath: string;
  bytes: number;
  jobId: string;
  format: ExportFormat;
}

/** Result of probe_export_tools. */
export interface ProbeExportToolsResult {
  browser: ExternalToolStatus;
}

export const HTML_EXPORT_FILTERS: FileFilter[] = [
  { name: "HTML", extensions: ["html", "htm"] },
  { name: "All Files", extensions: ["*"] },
];

export const PDF_EXPORT_FILTERS: FileFilter[] = [
  { name: "PDF", extensions: ["pdf"] },
  { name: "All Files", extensions: ["*"] },
];

// ----------------------------------------------------------------------------
// M5 — settings / pandoc
// ----------------------------------------------------------------------------

export interface AppSettings {
  trustedWorkspaces: string[];
  pandocPath: string | null;
  customCssPath: string | null;
  enableWikiLinks: boolean;
  experimentalCodeExecution: boolean;
  updateEndpoint: string | null;
}

export interface PandocStatus {
  available: boolean;
  path: string | null;
  version: string | null;
}

export interface ToolPresence {
  name: string;
  available: boolean;
  path: string | null;
}

export interface OptionalToolsStatus {
  pandoc: PandocStatus;
  graphviz: ToolPresence;
  plantuml: ToolPresence;
}

export type PandocFormat = "docx" | "epub" | "latex";

export interface PandocExportParams {
  sessionId: string;
  outputPath: string;
  format: PandocFormat;
  markdown?: string;
}

export interface PandocExportResult {
  outputPath: string;
  bytes: number;
  format: string;
}

export interface UpdateStatus {
  enabled: boolean;
  endpoint: string | null;
  message: string;
}

export const DOCX_EXPORT_FILTERS: FileFilter[] = [
  { name: "Word Document", extensions: ["docx"] },
  { name: "All Files", extensions: ["*"] },
];

export const EPUB_EXPORT_FILTERS: FileFilter[] = [
  { name: "EPUB", extensions: ["epub"] },
  { name: "All Files", extensions: ["*"] },
];

export const LATEX_EXPORT_FILTERS: FileFilter[] = [
  { name: "LaTeX", extensions: ["tex"] },
  { name: "All Files", extensions: ["*"] },
];


