/**
 * Offline preview pipeline benchmark for a Markdown file.
 * Measures: cold Notebook.init, cold/warm parseMD, HTML size, fence stats,
 * optional DOMPurify if available from app-ui deps.
 *
 * Usage: node scripts/bench-preview-file.mjs <path-to.md>
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const mdPath = process.argv[2];

if (!mdPath) {
  console.error("Usage: node scripts/bench-preview-file.mjs <file.md>");
  process.exit(2);
}

const markdown = fs.readFileSync(mdPath, "utf8");
const bytes = Buffer.byteLength(markdown, "utf8");

function fenceStats(t) {
  const langs = {};
  const re = /^```([^\r\n`]*)/gm;
  let m;
  while ((m = re.exec(t))) {
    const lang = (m[1] || "").trim().toLowerCase() || "(none)";
    langs[lang] = (langs[lang] || 0) + 1;
  }
  return {
    fenceMarkers: Object.values(langs).reduce((a, b) => a + b, 0),
    langs,
    mermaid: (t.match(/^```\s*mermaid/gim) || []).length,
    headings: (t.match(/^#{1,6}\s/gm) || []).length,
    lines: t.split(/\r?\n/).length,
  };
}

function now() {
  return performance.now();
}

async function loadCrossnote() {
  // Prefer workspace node_modules resolution via render-sidecar package.
  const pkg = path.join(root, "packages/render-sidecar/node_modules/crossnote/package.json");
  const alt = path.join(root, "node_modules/crossnote/package.json");
  const resolved = fs.existsSync(pkg) ? pkg : alt;
  if (!fs.existsSync(resolved)) {
    throw new Error("crossnote not found; run pnpm install from repo root");
  }
  const entry = pathToFileURL(path.join(path.dirname(resolved), "out/src/index.js")).href;
  // crossnote package exports may differ; try require via createRequire
  try {
    return await import("crossnote");
  } catch {
    return await import(entry);
  }
}

async function tryDomPurify(html) {
  try {
    // jsdom + dompurify path used by app if available
    const { JSDOM } = await import("jsdom");
    const createDOMPurify = (await import("dompurify")).default;
    const window = new JSDOM("").window;
    const purify = createDOMPurify(window);
    const t0 = now();
    const clean = purify.sanitize(html, {
      USE_PROFILES: { html: true, svg: true, svgFilters: true },
    });
    return { ms: now() - t0, outBytes: Buffer.byteLength(clean, "utf8") };
  } catch (e) {
    return { skipped: true, reason: String(e.message || e) };
  }
}

const stats = fenceStats(markdown);
console.log("=== Input ===");
console.log(
  JSON.stringify(
    {
      file: mdPath,
      bytes,
      chars: markdown.length,
      ...stats,
    },
    null,
    2,
  ),
);

const tImport0 = now();
const cn = await loadCrossnote();
const importMs = now() - tImport0;
console.log(`\n=== Module load ===\ncrossnote import: ${importMs.toFixed(1)} ms`);

const {
  Notebook,
  getDefaultNotebookConfig,
  utility,
} = cn;

const SAFE = {
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
  puppeteerArgs: ["--no-sandbox", "--disable-gpu"],
};

const notebookPath = utility.getCrossnoteBuildDirectory();
const logical = path.join(notebookPath, "__bench__.md");

// Cold Notebook.init
const tInit0 = now();
const notebook = await Notebook.init({
  notebookPath,
  config: { ...getDefaultNotebookConfig(), ...SAFE },
});
const initMs = now() - tInit0;
console.log(`Notebook.init (cold): ${initMs.toFixed(1)} ms`);

const engine = notebook.getNoteMarkdownEngine(logical);

async function parseOnce(label) {
  const t0 = now();
  const output = await engine.parseMD(markdown, {
    isForPreview: true,
    useRelativeFilePath: false,
    hideFrontMatter: true,
  });
  const ms = now() - t0;
  const htmlBytes = Buffer.byteLength(output.html || "", "utf8");
  const tocBytes = Buffer.byteLength(output.tocHTML || "", "utf8");
  console.log(
    `${label}: parseMD=${ms.toFixed(1)} ms | html=${htmlBytes} B | tocHTML=${tocBytes} B`,
  );
  return { ms, html: output.html || "", tocHTML: output.tocHTML || "", htmlBytes, tocBytes };
}

// Cold first render
const cold = await parseOnce("parseMD #1 (cold)");
// Warm renders
const warm1 = await parseOnce("parseMD #2 (warm)");
const warm2 = await parseOnce("parseMD #3 (warm)");
const warm3 = await parseOnce("parseMD #4 (warm)");

// Simulate IPC JSON cost
const tJson0 = now();
const req = JSON.stringify({
  id: 1,
  method: "render",
  params: { sessionId: "s", markdown, revision: 1 },
});
const reqMs = now() - tJson0;
const tJson1 = now();
const res = JSON.stringify({
  id: 1,
  ok: true,
  result: { html: cold.html, toc: [], diagnostics: [], renderMs: cold.ms },
});
const resMs = now() - tJson1;
console.log(
  `\n=== IPC payload (JSON.stringify only) ===\nrequest: ${reqMs.toFixed(1)} ms, ${Buffer.byteLength(req)} B\nresponse: ${resMs.toFixed(1)} ms, ${Buffer.byteLength(res)} B`,
);

// DOMPurify
console.log("\n=== DOMPurify (if jsdom available) ===");
const purify = await tryDomPurify(cold.html);
console.log(JSON.stringify(purify, null, 2));

// innerHTML cost estimate not available in node without jsdom full; skip heavy

const warms = [warm1.ms, warm2.ms, warm3.ms];
const warmAvg = warms.reduce((a, b) => a + b, 0) / warms.length;

console.log("\n=== Summary (this process) ===");
console.log(
  JSON.stringify(
    {
      importMs: +importMs.toFixed(1),
      notebookInitColdMs: +initMs.toFixed(1),
      parseMdColdMs: +cold.ms.toFixed(1),
      parseMdWarmAvgMs: +warmAvg.toFixed(1),
      parseMdWarmSamples: warms.map((x) => +x.toFixed(1)),
      htmlBytes: cold.htmlBytes,
      htmlOverMarkdown: +(cold.htmlBytes / bytes).toFixed(2),
      jsonRequestBytes: Buffer.byteLength(req),
      jsonResponseBytes: Buffer.byteLength(res),
      estimatedFirstPreviewMs: +(
        /* debounce 250 not included: open can skip */ initMs + cold.ms + (purify.ms || 0) + reqMs + resMs
      ).toFixed(1),
      estimatedWarmPreviewMs: +(warmAvg + (purify.ms || 0) + reqMs + resMs).toFixed(1),
      notes: [
        "Does not include Tauri IPC process hop or webview innerHTML.",
        "Cold first open in app may reuse already-running sidecar (skip import+init).",
        "If sidecar was cold-started for this file open, add import+init.",
      ],
    },
    null,
    2,
  ),
);
