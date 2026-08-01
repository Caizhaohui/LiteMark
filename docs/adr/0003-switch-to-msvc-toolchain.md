# ADR 0003 — Switch the Windows build to the MSVC toolchain

- **Status:** Accepted
- **Date:** 2026-07-29
- **Milestone:** M1
- **Supersedes (partly):** ADR 0001 §Decision D2 (the GNU toolchain choice)

## Context

ADR 0001 chose the `x86_64-pc-windows-gnu` Rust target for LiteMark because the
M0 environment had no MSVC C++ build tools, and rustup's self-contained MinGW
linker (`ld.exe` + import libraries) could link the Tauri binary without them.

When starting M1, the GNU toolchain no longer builds:

- `cargo build` / `cargo test` fail on every crate depending on `windows-sys`
  (Tauri pulls it in transitively) with:
  `dlltool.exe: CreateProcess` followed by `could not create import library`.
- The bundled `dlltool.exe` exists on disk but fails to spawn its child step.
  This is a known class of breakage for the self-contained MinGW; it normally
  requires a full external MinGW (MSYS2) install to repair — exactly the heavy
  install ADR 0001 deferred.

At the same time, the environment now **does** have a working MSVC linker: the
default toolchain is `stable-x86_64-pc-windows-msvc`, and a trivial
`rustc --target x86_64-pc-windows-msvc` program compiles and links. A full
`cargo test` of the existing M0 code (3 unit tests + doctests) passes on MSVC,
and `cargo build` links the Tauri app cleanly.

M1's acceptance criteria depend heavily on Rust unit tests (atomic save,
encoding round-trips, long/unicode/emoji paths, recovery). With the GNU
toolchain broken, those tests cannot run at all. MSVC unblocks them.

## Decision

**Build LiteMark with the `x86_64-pc-windows-msvc` target.**

Concretely:

- `rust-toolchain.toml` pins `targets = ["x86_64-pc-windows-msvc"]`.
- The gitignored, machine-specific `.cargo/config.toml` that embedded the
  per-user GNU linker path is removed. The MSVC target needs no such config —
  rustc discovers `link.exe` and the Windows SDK through the standard vcvars /
  SDK path.
- `scripts/setup-env.{sh,ps1}` no longer write a `.cargo/config.toml`. They now
  perform a one-time sanity check: compile a trivial program for the MSVC
  target and fail with a clear install hint if the linker / SDK is missing.
- `scripts/cargo-test.bat` drops the self-contained-PATH hack; it is now a
  plain `cargo test` wrapper kept only for workflow compatibility.
- `.cargo/config.toml.example` is rewritten to document the MSVC decision and
  the required prerequisite ("Visual Studio 2022 Build Tools — Desktop
  development with C++", or standalone MSVC compiler + Windows SDK).

This partially reverses ADR 0001 §D2. ADR 0001's other decisions (Tauri 2 +
Node sidecar, JSON Lines IPC) are unchanged.

## Prerequisite

- Visual Studio 2022 Build Tools with the **"Desktop development with C++"**
  workload, **or** a standalone MSVC compiler + Windows SDK install.

`scripts/setup-env.{sh,ps1}` verifies this and prints an actionable message if
the linker / SDK is absent.

## Consequences

- ✅ `cargo test` works (unblocks all M1 Rust unit tests).
- ✅ `cargo build` / `pnpm tauri build` link the app.
- ✅ MSVC is Tauri's officially-preferred Windows target.
- ⚠️ Developers must have MSVC C++ build tools installed (heavier than the GNU
  self-contained path, but standard for Windows Rust development).

## Alternatives considered

- **Repair the GNU toolchain** by installing a full external MinGW (MSYS2) to
  provide a working `dlltool`/`as`. Rejected: ADR 0001 already deferred this as
  a heavy install, and MSVC now works, so the original reason for GNU is gone.
- **Stay on GNU and skip Rust unit tests.** Rejected: it directly violates
  M1's acceptance criteria.
