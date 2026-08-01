/**
 * crossnote adapter — the safe boundary between LiteMark and crossnote.
 *
 * Responsibilities (DEVELOPMENT_PLAN.md §5.4, §8.1, §2.4):
 *  - Cache one `Notebook` per notebookPath (it holds the shared markdown-it).
 *  - Enforce safe defaults regardless of caller input: scripts off, no `file://`
 *    protocol, no HTML5 embed, print background on.
 *  - Render Markdown STRINGS to HTML in memory — never write to the user's
 *    source file. `logicalFilePath` is used only for relative-resource /
 *    import / link resolution.
 *  - Build fully self-contained HTML for export via the public
 *    `generateHTMLTemplateForExport` helper (bypassing `htmlExport`, which writes
 *    next to the source file).
 *
 * The crossnote public API (verified against crossnote@0.9.31):
 *   Notebook.init({ notebookPath, config, fs }) -> Notebook
 *   notebook.getNoteMarkdownEngine(filePath)    -> MarkdownEngine
 *   engine.parseMD(inputString, opts)          -> { html, markdown, tocHTML, yamlConfig, JSAndCssFiles }
 *   engine.generateHTMLTemplateForExport(html, yamlConfig, opts) -> string
 */

import {
  Notebook,
  getDefaultNotebookConfig,
  utility,
  type NotebookConfig,
} from "crossnote";
import type { RenderOptions } from "@litemark/shared-protocol";

/**
 * The crossnote version this sidecar was built and tested against. Mirrored
 * into ping/capabilities results. Bumping crossnote requires re-validating the
 * render + PDF spikes (see docs/adr/0002-crossnote-in-memory-rendering.md).
 */
export const CROSSNOTE_VERSION = "0.9.31";

/**
 * Safe defaults merged into every Notebook config (DEVELOPMENT_PLAN.md §8.1).
 *
 * `enableScriptExecution` MUST stay false for untrusted Markdown — when false,
 * crossnote zeroes JS/CSS `@import` and disables code-chunk execution. We also
 * drop `file://` and `atom://` from crossnote's default protocol whitelist so
 * rendered HTML cannot reference local files directly (local images are served
 * through a Rust-controlled protocol instead).
 */
export const SAFE_DEFAULTS: Partial<NotebookConfig> = {
  markdownParser: "markdown-it",
  mathRenderingOption: "KaTeX",
  previewTheme: "github-light.css",
  codeBlockTheme: "auto.css",
  enableScriptExecution: false,
  enableHTML5Embed: false,
  printBackground: true,
  protocolsWhiteList: "http://, https://, mailto:, tel:",
  enableWikiLinkSyntax: false,
  enableCriticMarkupSyntax: false,
  enableExtendedTableSyntax: false,
  // Headless sidecar safety args for Puppeteer (used by PDF export).
  puppeteerArgs: ["--no-sandbox", "--disable-gpu"],
} as Partial<NotebookConfig>;

/**
 * Merge caller-supplied render options into the safe defaults, but NEVER allow
 * callers to flip the security-sensitive flags on. `trusted` /
 * `enableScriptExecution` are ignored from untrusted input by design.
 */
export function resolveConfig(callerOptions?: RenderOptions): Partial<NotebookConfig> {
  const config: Record<string, unknown> = { ...SAFE_DEFAULTS };
  if (callerOptions) {
    // Only honor cosmetic, non-security options from the caller.
    if (callerOptions.theme) config.previewTheme = callerOptions.theme;
    if (callerOptions.codeBlockTheme) config.codeBlockTheme = callerOptions.codeBlockTheme;
    if (callerOptions.mathRenderer) {
      config.mathRenderingOption =
        callerOptions.mathRenderer === "None" ? "None" : callerOptions.mathRenderer;
    }
  }
  // Hard-enforce: scripts always off in the sidecar.
  config.enableScriptExecution = false;
  config.enableHTML5Embed = false;
  return config as Partial<NotebookConfig>;
}

/**
 * A canonical notebook path used when the caller provides none. Pointing at a
 * real, empty-ish directory avoids crossnote touching the user's document dir,
 * and avoids loading any `.crossnote/parser.js` from an untrusted location
 * (DEVELOPMENT_PLAN.md §12.5).
 */
const FALLBACK_NOTEBOOK_PATH = utility.getCrossnoteBuildDirectory();

interface CacheEntry {
  notebook: Notebook;
  notebookPath: string;
  configKey: string;
}

const notebookCache = new Map<string, CacheEntry>();

function configKey(options: RenderOptions | undefined): string {
  return JSON.stringify(options ?? null);
}

/**
 * Get (or create) a Notebook for the given notebookPath + resolved options.
 * Cached per (notebookPath, configKey) so repeated renders reuse the shared
 * markdown-it instance. Separated for unit-testability.
 */
export async function getNotebook(
  notebookPath: string | null | undefined,
  options: RenderOptions | undefined,
): Promise<{ notebook: Notebook; notebookPath: string }> {
  const resolvedNotebookPath =
    notebookPath && notebookPath.trim() !== "" ? notebookPath : FALLBACK_NOTEBOOK_PATH;
  const key = `${resolvedNotebookPath}::${configKey(options)}`;
  const cached = notebookCache.get(key);
  if (cached) {
    return { notebook: cached.notebook, notebookPath: cached.notebookPath };
  }
  const config = { ...getDefaultNotebookConfig(), ...resolveConfig(options) };
  const notebook = await Notebook.init({
    notebookPath: resolvedNotebookPath,
    config,
  });
  notebookCache.set(key, {
    notebook,
    notebookPath: resolvedNotebookPath,
    configKey: key,
  });
  return { notebook, notebookPath: resolvedNotebookPath };
}

/** Drop all cached notebooks. Used on shutdown and in tests. */
export function clearNotebookCache(): void {
  notebookCache.clear();
}

/**
 * Choose a logical file path for the MarkdownEngine. Must be absolute for
 * relative-resource resolution. When the caller supplies one, use it; otherwise
 * derive a stable synthetic path under the notebook root. The file does NOT
 * need to exist on disk because parseMD is given the markdown string directly.
 */
export function resolveLogicalFilePath(
  notebookPath: string,
  logicalFilePath: string | null | undefined,
): string {
  if (logicalFilePath && logicalFilePath.trim() !== "") {
    return logicalFilePath;
  }
  // Synthetic in-memory document name.
  const base = notebookPath.replace(/[\\/]+$/, "");
  return `${base}/__litemark_unsaved__.md`;
}
