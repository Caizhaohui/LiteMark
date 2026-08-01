/**
 * Prepare Tauri bundle resources for release (M3).
 *
 * Copies:
 *  - render-sidecar dist → src-tauri/resources/sidecar/
 *  - LICENSE + THIRD_PARTY_NOTICES → src-tauri/resources/
 *
 * Optionally copies a portable Node runtime into resources/node/ when
 * LITEMARK_BUNDLE_NODE=1 and LITEMARK_NODE points at a node.exe (so end users
 * do not need a system Node install). Never downloads Node automatically.
 */

import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const resources = join(root, "src-tauri", "resources");
const sidecarSrc = join(root, "packages", "render-sidecar", "dist");
const sidecarDst = join(resources, "sidecar");

function ensureDir(p) {
  mkdirSync(p, { recursive: true });
}

function copyFile(src, dst) {
  ensureDir(dirname(dst));
  cpSync(src, dst);
  console.log(`[prepare-release] copied ${src} → ${dst}`);
}

if (!existsSync(sidecarSrc)) {
  console.error(
    "[prepare-release] sidecar dist missing. Run: pnpm --filter @litemark/render-sidecar build",
  );
  process.exit(1);
}

// Reset sidecar resources.
rmSync(sidecarDst, { recursive: true, force: true });
ensureDir(sidecarDst);
cpSync(sidecarSrc, sidecarDst, { recursive: true });
console.log(`[prepare-release] sidecar → ${sidecarDst}`);

// Copy node_modules needed at runtime? crossnote is a dependency of the
// sidecar package — for production the recommended path is to run the built
// dist with a Node that can resolve workspace node_modules, OR ship a
// self-contained bundle. For M3 we also write a small package.json so that
// `node index.js` can resolve from the monorepo node_modules when installed
// beside the app in a full deploy tree. For the NSIS package, operators should
// set LITEMARK_BUNDLE_NODE=1 and provide node + a production node_modules.
const runtimePkg = {
  name: "litemark-render-sidecar-runtime",
  type: "module",
  private: true,
};
writeFileSync(join(sidecarDst, "package.json"), JSON.stringify(runtimePkg, null, 2));

// Licenses
const license = join(root, "LICENSE");
const notices = join(root, "THIRD_PARTY_NOTICES.md");
if (existsSync(license)) copyFile(license, join(resources, "LICENSE"));
if (existsSync(notices)) copyFile(notices, join(resources, "THIRD_PARTY_NOTICES.md"));

// Optional portable Node
if (process.env.LITEMARK_BUNDLE_NODE === "1") {
  const nodeSrc = process.env.LITEMARK_NODE || "node.exe";
  const nodeDstDir = join(resources, "node");
  if (existsSync(nodeSrc)) {
    ensureDir(nodeDstDir);
    const dst = join(nodeDstDir, "node.exe");
    cpSync(nodeSrc, dst);
    console.log(`[prepare-release] bundled Node → ${dst}`);
  } else {
    console.warn(
      `[prepare-release] LITEMARK_BUNDLE_NODE=1 but node not found at ${nodeSrc}; skipping`,
    );
  }
}

// README for operators
writeFileSync(
  join(resources, "README-SIDECAR.txt"),
  readFileSync(join(root, "scripts", "SIDECAR-BUNDLE.txt"), "utf8"),
);

console.log("[prepare-release] done");
