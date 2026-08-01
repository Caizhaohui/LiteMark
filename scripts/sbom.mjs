/**
 * Emit a lightweight SBOM summary (M6) from lockfiles.
 * Full SPDX/CycloneDX can replace this later; this is a human-auditable dump.
 */
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "dist-sbom");
mkdirSync(outDir, { recursive: true });

const lines = [];
lines.push("# LiteMark SBOM summary");
lines.push(`generated: ${new Date().toISOString()}`);
lines.push("");

try {
  const lock = readFileSync(join(root, "pnpm-lock.yaml"), "utf8");
  const pkgs = new Set();
  for (const m of lock.matchAll(/^\s{2}(\S+)@/gm)) {
    pkgs.add(m[1]);
  }
  lines.push(`## npm (pnpm) — ${pkgs.size} package keys`);
  [...pkgs].sort().slice(0, 500).forEach((p) => lines.push(`- ${p}`));
  if (pkgs.size > 500) lines.push(`- … and ${pkgs.size - 500} more`);
} catch (e) {
  lines.push(`## npm: failed to read pnpm-lock.yaml (${e})`);
}

lines.push("");
try {
  const cargo = readFileSync(join(root, "src-tauri", "Cargo.lock"), "utf8");
  const names = [...cargo.matchAll(/^name = "([^"]+)"/gm)].map((m) => m[1]);
  lines.push(`## crates — ${names.length} packages`);
  names.slice(0, 500).forEach((n) => lines.push(`- ${n}`));
} catch (e) {
  lines.push(`## crates: failed (${e})`);
}

const out = join(outDir, "sbom-summary.md");
writeFileSync(out, lines.join("\n") + "\n");
console.log(`Wrote ${out}`);
