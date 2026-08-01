/**
 * PDF export via puppeteer-core driving the system Edge/Chrome.
 *
 * crossnote's own `MarkdownEngine.chromeExport` reads/writes next to the source
 * — forbidden by the LiteMark security model. We compose public helpers and
 * drive puppeteer-core ourselves. Supports cancel + progress (M3).
 */

import puppeteer, { type Browser } from "puppeteer-core";
import { spawn, type ChildProcess } from "node:child_process";
import { getNotebook, resolveLogicalFilePath, SAFE_DEFAULTS } from "./notebook-pool.js";
import { findBrowserPath } from "../security/browser-probe.js";
import { emitExportProgress } from "../events.js";
import { beginJob, endJob, onJobCleanup, throwIfCancelled } from "../jobs.js";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as http from "node:http";
import { pathToFileURL } from "node:url";
import type { PdfPageOptions, ExportPdfParams } from "@litemark/shared-protocol";

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

export interface ExportPdfOptions {
  markdown: string;
  logicalFilePath?: string | null;
  notebookPath?: string | null;
  outputPath: string;
  page?: Partial<PdfPageOptions>;
  browserPath?: string | null;
  jobId?: string;
}

export interface ExportPdfResult {
  outputPath: string;
  bytes: number;
}

const MM_PER_INCH = 25.4;
function mmToInch(mm: number): number {
  return mm / MM_PER_INCH;
}

const DEBUG_PORT_READY_TIMEOUT_MS = 10_000;

async function launchBrowserOnPort(
  executablePath: string,
  userDataDir: string,
  extraArgs: string[],
): Promise<{ browserURL: string; child: ChildProcess }> {
  const port = await pickFreePort();
  const args = [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--no-sandbox",
    "--disable-background-networking",
    "--disable-sync",
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${userDataDir}`,
    ...extraArgs,
    "about:blank",
  ];

  const child = spawn(executablePath, args, {
    stdio: "ignore",
    detached: false,
    windowsHide: true,
  });

  const browserURL = `http://127.0.0.1:${port}`;
  const ready = await waitForPort(browserURL, DEBUG_PORT_READY_TIMEOUT_MS);
  if (!ready) {
    child.kill();
    throw {
      code: "EXPORT_FAILED",
      message: `Browser debug port ${browserURL} did not become ready within ${DEBUG_PORT_READY_TIMEOUT_MS}ms`,
      details: { browserPath: executablePath },
    };
  }
  return { browserURL, child };
}

function pickFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = http.createServer();
    srv.listen(0, "127.0.0.1", () => {
      const addr = srv.address();
      const port = typeof addr === "object" && addr ? addr.port : 0;
      srv.close(() => resolve(port));
    });
    srv.on("error", reject);
  });
}

function waitForPort(browserURL: string, timeoutMs: number): Promise<boolean> {
  const start = Date.now();
  return new Promise((resolve) => {
    const attempt = () => {
      const req = http.get(`${browserURL}/json/version`, (res) => {
        res.resume();
        res.on("end", () => resolve(true));
      });
      req.on("error", () => {
        if (Date.now() - start > timeoutMs) resolve(false);
        else setTimeout(attempt, 200);
      });
      req.setTimeout(300, () => req.destroy(new Error("timeout")));
    };
    attempt();
  });
}

function killChild(child: ChildProcess | undefined): void {
  if (!child || child.killed) return;
  try {
    child.kill();
  } catch {
    /* best-effort */
  }
  // On Windows, ensure the process tree is gone.
  if (process.platform === "win32" && child.pid) {
    try {
      spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"], {
        stdio: "ignore",
        windowsHide: true,
      });
    } catch {
      /* best-effort */
    }
  }
}

export async function exportPdf(opts: ExportPdfOptions): Promise<ExportPdfResult> {
  const jobId = opts.jobId ?? `pdf-${Date.now()}`;
  beginJob(jobId);

  let browser: Browser | undefined;
  let child: ChildProcess | undefined;
  let tmpDir: string | undefined;

  onJobCleanup(jobId, async () => {
    if (browser) {
      await browser.close().catch(() => undefined);
      browser = undefined;
    }
    killChild(child);
    child = undefined;
    if (tmpDir) {
      await fs.rm(tmpDir, { recursive: true, force: true }).catch(() => undefined);
      tmpDir = undefined;
    }
  });

  try {
    emitExportProgress(jobId, "preparing", 0.05, "Looking for browser");
    throwIfCancelled(jobId);

    const browserPath = await findBrowserPath(opts.browserPath ?? null);
    if (!browserPath) {
      throw {
        code: "BROWSER_NOT_FOUND",
        message:
          "No Edge or Chrome installation found. Install Microsoft Edge, or set a browser path.",
        details: { probed: ["msedge", "chrome"] },
      };
    }

    emitExportProgress(jobId, "rendering", 0.2, "Rendering Markdown");
    throwIfCancelled(jobId);

    const { notebook, notebookPath } = await getNotebook(opts.notebookPath, undefined);
    const logicalFilePath = resolveLogicalFilePath(notebookPath, opts.logicalFilePath);
    const engine = notebook.getNoteMarkdownEngine(logicalFilePath);

    const parsed = await engine.parseMD(opts.markdown, {
      isForPreview: false,
      useRelativeFilePath: false,
      hideFrontMatter: false,
    });
    throwIfCancelled(jobId);

    const documentHtml = await engine.generateHTMLTemplateForExport(
      parsed.html,
      parsed.yamlConfig,
      {
        isForPrint: true,
        isForPrince: false,
        offline: true,
        embedLocalImages: true,
        embedSVG: true,
      },
    );
    throwIfCancelled(jobId);

    tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "litemark-pdf-"));
    const htmlPath = path.join(tmpDir, "document.html");
    await fs.writeFile(htmlPath, documentHtml, "utf8");
    const profileDir = path.join(tmpDir, "profile");

    emitExportProgress(jobId, "launching_browser", 0.45, "Launching browser");
    throwIfCancelled(jobId);

    const puppeteerArgs = (SAFE_DEFAULTS.puppeteerArgs as string[] | undefined) ?? [];
    const launched = await launchBrowserOnPort(browserPath, profileDir, puppeteerArgs);
    child = launched.child;
    throwIfCancelled(jobId);

    browser = await puppeteer.connect({ browserURL: launched.browserURL });
    throwIfCancelled(jobId);

    emitExportProgress(jobId, "printing", 0.7, "Printing PDF");
    const page = await browser.newPage();
    await page.goto(pathToFileURL(htmlPath).toString(), {
      waitUntil: "networkidle0",
      timeout: 60_000,
    });
    throwIfCancelled(jobId);

    const pageOpts = { ...DEFAULT_PDF_PAGE_OPTIONS, ...opts.page };
    await page.pdf({
      path: opts.outputPath,
      format: pageOpts.pageSize,
      landscape: pageOpts.landscape,
      printBackground: pageOpts.printBackground,
      displayHeaderFooter: pageOpts.displayHeaderFooter,
      margin: {
        top: mmToInch(pageOpts.marginTopMm) + "in",
        right: mmToInch(pageOpts.marginRightMm) + "in",
        bottom: mmToInch(pageOpts.marginBottomMm) + "in",
        left: mmToInch(pageOpts.marginLeftMm) + "in",
      },
    });
    throwIfCancelled(jobId);

    const stat = await fs.stat(opts.outputPath);
    emitExportProgress(jobId, "finalizing", 1, "Done");
    return { outputPath: opts.outputPath, bytes: stat.size };
  } finally {
    await endJob(jobId);
  }
}

export type { ExportPdfParams };
