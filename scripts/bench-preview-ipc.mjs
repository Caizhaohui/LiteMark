/**
 * End-to-end sidecar IPC bench for one Markdown file.
 * Usage: node scripts/bench-preview-ipc.mjs <file.md>
 */
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const mdPath = process.argv[2];
if (!mdPath) {
  console.error("Usage: node scripts/bench-preview-ipc.mjs <file.md>");
  process.exit(2);
}

const markdown = fs.readFileSync(mdPath, "utf8");
const bytes = Buffer.byteLength(markdown, "utf8");
const sidecarJs = path.join(root, "packages/render-sidecar/dist/index.js");
const cwd = path.join(root, "packages/render-sidecar");

const t0 = performance.now();
const child = spawn(process.execPath, [sidecarJs], {
  stdio: ["pipe", "pipe", "pipe"],
  cwd,
  env: { ...process.env },
  windowsHide: true,
});

let buf = "";
const queue = [];
let waiter = null;

function pushLine(line) {
  if (waiter) {
    const w = waiter;
    waiter = null;
    w(line);
  } else {
    queue.push(line);
  }
}

function nextLine() {
  if (queue.length) return Promise.resolve(queue.shift());
  return new Promise((resolve) => {
    waiter = resolve;
  });
}

child.stdout.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  buf += chunk;
  let idx;
  while ((idx = buf.indexOf("\n")) >= 0) {
    const line = buf.slice(0, idx);
    buf = buf.slice(idx + 1);
    if (line.trim()) pushLine(line);
  }
});

let stderr = "";
child.stderr.setEncoding("utf8");
child.stderr.on("data", (d) => {
  stderr += d;
});

const readyLine = await nextLine();
const readyMs = performance.now() - t0;
console.log("ready", readyMs.toFixed(1), "ms", readyLine.slice(0, 100));

function send(obj) {
  child.stdin.write(JSON.stringify(obj) + "\n", "utf8");
}

async function rpc(id, method, params) {
  const start = performance.now();
  send({ id, method, params });
  // Drain events until matching id response
  for (;;) {
    const line = await nextLine();
    let msg;
    try {
      msg = JSON.parse(line);
    } catch {
      continue;
    }
    if (msg && msg.event) continue; // progress/ready
    if (msg && msg.id === id) {
      return { ms: performance.now() - start, msg };
    }
  }
}

// createSession
const create = await rpc("c1", "createSession", {
  sessionId: "bench-session",
  logicalFilePath: mdPath,
});
console.log("createSession", create.ms.toFixed(1), "ms", create.msg.ok);

// render #1 cold-ish (highlighter languages first load inside process)
const r1 = await rpc("r1", "render", {
  sessionId: "bench-session",
  markdown,
  logicalFilePath: mdPath,
  revision: 1,
  options: { mathRenderer: "KaTeX", theme: "github-light.css" },
});
const res1 = r1.msg.result || {};
console.log(
  "render#1",
  r1.ms.toFixed(1),
  "ms",
  "renderMs=",
  res1.renderMs,
  "htmlBytes=",
  Buffer.byteLength(res1.html || "", "utf8"),
  "toc=",
  (res1.toc || []).length,
);

// render #2 warm
const r2 = await rpc("r2", "render", {
  sessionId: "bench-session",
  markdown,
  logicalFilePath: mdPath,
  revision: 2,
  options: { mathRenderer: "KaTeX", theme: "github-light.css" },
});
const res2 = r2.msg.result || {};
console.log(
  "render#2",
  r2.ms.toFixed(1),
  "ms",
  "renderMs=",
  res2.renderMs,
  "htmlBytes=",
  Buffer.byteLength(res2.html || "", "utf8"),
);

// render #3 warm
const r3 = await rpc("r3", "render", {
  sessionId: "bench-session",
  markdown,
  logicalFilePath: mdPath,
  revision: 3,
  options: { mathRenderer: "KaTeX", theme: "github-light.css" },
});
console.log("render#3", r3.ms.toFixed(1), "ms", "renderMs=", r3.msg.result?.renderMs);

// HTML shape
const html = res1.html || "";
const spanCount = (html.match(/<span\b/gi) || []).length;
const preCount = (html.match(/<pre\b/gi) || []).length;
const codeCount = (html.match(/<code\b/gi) || []).length;
const tableCount = (html.match(/<table\b/gi) || []).length;
console.log(
  JSON.stringify(
    {
      inputBytes: bytes,
      readyMs: +readyMs.toFixed(1),
      createSessionMs: +create.ms.toFixed(1),
      render1IpcMs: +r1.ms.toFixed(1),
      render1ParseMs: res1.renderMs,
      render2IpcMs: +r2.ms.toFixed(1),
      render2ParseMs: res2.renderMs,
      render3IpcMs: +r3.ms.toFixed(1),
      render3ParseMs: r3.msg.result?.renderMs,
      htmlBytes: Buffer.byteLength(html, "utf8"),
      spanCount,
      preCount,
      codeCount,
      tableCount,
      // Double-click path estimate if sidecar cold-started for this open:
      estFirstPreviewIfSidecarColdMs: +(
        readyMs +
        create.ms +
        r1.ms +
        250
      ).toFixed(1),
      // Sidecar already warm (app started earlier):
      estFirstPreviewIfSidecarWarmMs: +(create.ms + r1.ms + 250).toFixed(1),
      estSubsequentPreviewMs: +(r2.ms + 250).toFixed(1),
      note: "DOMPurify + innerHTML not included (~tens of ms typical for 56KB HTML)",
    },
    null,
    2,
  ),
);

send({ id: "bye", method: "shutdown", params: {} });
await new Promise((r) => setTimeout(r, 200));
child.kill();
process.exit(0);
