# ADR 0001 — Tauri 2 + Node sidecar architecture (GNU toolchain)

- **Status:** Accepted
- **Date:** 2026-07-15
- **Milestone:** M0

## Context

LiteMark needs a Windows desktop shell (windows, files, system integration,
security boundary) plus the rich Markdown rendering/export capabilities of
[`crossnote`](https://github.com/shd101wyy/crossnote). crossnote is a Node.js
library — it uses Node's `fs`, optional subprocesses, `puppeteer-core`, `sharp`,
and Pandoc/Java/LaTeX. It cannot run inside a WebView directly.

DEVELOPMENT_PLAN.md §2.3–2.4 decided on Tauri 2 for the shell and a separate
Node sidecar for crossnote. This ADR records the toolchain decision forced by
the M0 build environment.

### Environment constraints observed at M0

- **No MSVC C++ build tools** are installed (no Visual Studio Build Tools, no
  `cl.exe`, no real MSVC `link.exe`). Only the Windows SDK is present.
- **rustup ships a self-contained MinGW** for the `x86_64-pc-windows-gnu`
  target: a link-only `ld.exe`, import libraries, and `dlltool.exe`, under
  `…/rustlib/x86_64-pc-windows-gnu/{bin/self-contained,lib/self-contained}`.
  There is **no `as.exe`** (GNU assembler) and no full external MinGW install.
- Only **Node.js 24.14.0** is installed (the plan pinned Node 22; see §Decision
  D3). `corepack` is available, so pnpm 11.13.0 is enabled through it.
- WebView2 Runtime v143 is present.

## Decision

**D1 — Tauri 2 (Rust) shell + Node sidecar**, communicating over stdin/stdout
JSON Lines (one object per line; logs to stderr). The webview never talks to
the sidecar directly — it calls Rust commands, which enforce timeouts,
correlation, and structured errors. This matches §5.1–5.3.

**D2 — Build with the `x86_64-pc-windows-gnu` Rust target.** The self-contained
MinGW linker (`ld.exe` + `-C link-self-contained=yes` + `-Lnative` to the
bundled import libraries) successfully links the Tauri app binary and its
external dependencies (serde, tokio, tauri, …). A bootstrap script
(`scripts/setup-env.{ps1,sh}`) resolves the per-user sysroot via
`rustc +stable-x86_64-pc-windows-gnu --print sysroot` and writes the
developer-specific, gitignored `.cargo/config.toml`. The committed
`.cargo/config.toml.example` documents the structure. This avoids requiring
every developer to install MSVC Build Tools.

**D3 — Node 24, not Node 22.** `.nvmrc` and `package.json#engines` are bumped
to `>=24.0.0` because only Node 24 is installed in this environment. Node 24
is a strict superset of Node 22 for our toolchain. This is a deviation from the
plan's Node 22 LTS recommendation; recorded here for traceability.

## Alternatives considered

- **MSVC target (`x86_64-pc-windows-msvc`)** — Tauri's officially preferred
  Windows target. Rejected at M0 because the environment has no MSVC compiler.
  Revisit when MSVC Build Tools are installed; the switch is config-only.
- **Full external MinGW (MSYS2)** — would provide `as.exe` and a complete
  binutils, unblocking `cargo test` (see Consequences). Rejected for M0 to
  avoid a heavy install; deferred.
- **Bundling Chromium** — explicitly forbidden by §2.3/§9.2; we drive the
  system Edge/Chrome.

## Consequences

- ✅ `cargo build` / `cargo check` / `cargo clippy -D warnings` / `cargo fmt`
  all pass. `pnpm tauri build` produces an NSIS bundle (subject to the same
  link path).
- ✅ The full sidecar→crossnote→HTML and →PDF spikes succeed (see ADR 0002).
- ⚠️ **`cargo test` does not complete in this environment.** The `windows-*`
  crates (transitive deps of Tauri) use `#[link(kind = "raw-dylib")]`, which
  makes rustc invoke `dlltool` to synthesize import libraries; `dlltool` then
  needs `as.exe` (the GNU assembler), which the self-contained MinGW does not
  ship. Workaround applied for M0: the Rust unit tests (`src/error.rs`) were
  verified in an isolated crate that does not pull in Tauri's windows-* deps —
  all 3 pass. Full `cargo test` requires either MSVC Build Tools or a full
  MinGW/MSYS2 install; tracked as an M1 prerequisite.
- ⚠️ Node 24 vs plan's Node 22 — flagged in the M0 report; CI pins Node 24.

## Status

Accepted for M0. Re-evaluate the MSVC-vs-GNU choice (and Node version) at M1
once the toolchain can be standardized.
