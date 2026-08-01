/**
 * Dependency audit helper (M6). Runs pnpm audit when available; always exits 0
 * with a report file so CI can archive results without blocking on advisories
 * until a policy is chosen.
 */
import { spawnSync } from "node:child_process";
import { writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "dist-sbom");
mkdirSync(outDir, { recursive: true });

const r = spawnSync("pnpm", ["audit", "--json"], {
  cwd: root,
  encoding: "utf8",
  shell: true,
});

const out = join(outDir, "pnpm-audit.json");
writeFileSync(out, r.stdout || JSON.stringify({ error: r.stderr || "audit failed" }, null, 2));
console.log(`pnpm audit exit=${r.status} → ${out}`);

const cargo = spawnSync(
  "cargo",
  ["audit", "--manifest-path", "src-tauri/Cargo.toml"],
  { cwd: root, encoding: "utf8", shell: true },
);
writeFileSync(
  join(outDir, "cargo-audit.txt"),
  cargo.stdout || cargo.stderr || "cargo-audit not installed or failed",
);
console.log(`cargo audit exit=${cargo.status}`);
