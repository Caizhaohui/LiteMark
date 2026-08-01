/**
 * HTML export — produce a fully self-contained HTML document.
 *
 * We deliberately do NOT call `MarkdownEngine.htmlExport`, because that method
 * reads from `engine.filePath` and writes the result next to the source file.
 * Instead we compose the public helpers: parseMD -> generateHTMLTemplateForExport.
 * The output path is authorized by the Rust core.
 */

import { getNotebook, resolveLogicalFilePath } from "./notebook-pool.js";
import { emitExportProgress } from "../events.js";
import { beginJob, endJob, throwIfCancelled } from "../jobs.js";
import * as fs from "node:fs/promises";
import type { RenderOptions } from "@litemark/shared-protocol";

export interface ExportHtmlOptions {
  markdown: string;
  logicalFilePath?: string | null;
  notebookPath?: string | null;
  /** When true, inline CSS/fonts so the file is fully offline. */
  offline?: boolean;
  /** Caller-authorized absolute output path (validated by Rust core). */
  outputPath: string;
  jobId?: string;
  options?: RenderOptions;
}

export interface ExportHtmlResult {
  outputPath: string;
  bytes: number;
}

export async function exportHtml(opts: ExportHtmlOptions): Promise<ExportHtmlResult> {
  const jobId = opts.jobId ?? `html-${Date.now()}`;
  beginJob(jobId);
  try {
    emitExportProgress(jobId, "preparing", 0.05, "Preparing HTML export");
    throwIfCancelled(jobId);

    const { notebook, notebookPath } = await getNotebook(opts.notebookPath, opts.options);
    const logicalFilePath = resolveLogicalFilePath(notebookPath, opts.logicalFilePath);
    const engine = notebook.getNoteMarkdownEngine(logicalFilePath);

    emitExportProgress(jobId, "rendering", 0.25, "Rendering Markdown");
    throwIfCancelled(jobId);

    const offline = opts.offline ?? true;
    const parsed = await engine.parseMD(opts.markdown, {
      isForPreview: false,
      useRelativeFilePath: false,
      hideFrontMatter: false,
    });
    throwIfCancelled(jobId);

    emitExportProgress(jobId, "writing", 0.7, "Writing HTML");
    const documentHtml = await engine.generateHTMLTemplateForExport(
      parsed.html,
      parsed.yamlConfig,
      {
        isForPrint: false,
        isForPrince: false,
        offline,
        embedLocalImages: offline,
        embedSVG: true,
        isForBrowser: true,
      },
    );
    throwIfCancelled(jobId);

    await fs.writeFile(opts.outputPath, documentHtml, "utf8");
    const stat = await fs.stat(opts.outputPath);

    emitExportProgress(jobId, "finalizing", 1, "Done");
    return { outputPath: opts.outputPath, bytes: stat.size };
  } finally {
    await endJob(jobId);
  }
}
