/**
 * In-memory Markdown rendering via crossnote.
 *
 * This is the heart of the "render unsaved content" requirement
 * (DEVELOPMENT_PLAN.md §5.4): we pass the editor's in-memory Markdown string
 * directly to `MarkdownEngine.parseMD`, which never reads or writes the user's
 * source file when given a non-empty `inputString`.
 */

import { getNotebook, resolveLogicalFilePath } from "./notebook-pool.js";
import { extractToc, type TocEntry } from "./toc.js";
import type { RenderOptions } from "@litemark/shared-protocol";

export interface RenderMarkdownTextOptions {
  markdown: string;
  /** Used only for relative-resource/import/link resolution; never read/written. */
  logicalFilePath?: string | null;
  /** Restricts where relative resources may resolve. */
  notebookPath?: string | null;
  options?: RenderOptions;
}

export interface RenderMarkdownTextResult {
  html: string;
  toc: TocEntry[];
}

/**
 * Render a Markdown STRING to HTML in memory. Never touches the source file.
 *
 * Implementation note: `parseMD` runs the full crossnote pipeline
 * (transformMarkdown -> renderMarkdown -> render-enhancers -> TOC), which is
 * what gives us Mermaid, KaTeX, code highlighting, etc.
 */
export async function renderMarkdownText(
  opts: RenderMarkdownTextOptions,
): Promise<RenderMarkdownTextResult> {
  const { notebook, notebookPath } = await getNotebook(opts.notebookPath, opts.options);
  const logicalFilePath = resolveLogicalFilePath(notebookPath, opts.logicalFilePath);
  const engine = notebook.getNoteMarkdownEngine(logicalFilePath);

  // isForPreview:true enables preview-mode rendering. useRelativeFilePath:false
  // keeps emitted resource references absolute/stable. hideFrontMatter:true
  // keeps YAML out of the rendered body (it remains available via yamlConfig).
  const output = await engine.parseMD(opts.markdown, {
    isForPreview: true,
    useRelativeFilePath: false,
    hideFrontMatter: true,
  });

  const toc = extractToc(output.tocHTML);
  return { html: output.html, toc };
}
