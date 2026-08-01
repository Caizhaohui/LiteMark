#!/usr/bin/env bash
# scripts/setup-env.sh
#
# POSIX/bash version of scripts/setup-env.ps1 — one-time environment sanity
# check for LiteMark on Windows (Git Bash / MSYS2).
#
# History: LiteMark originally built with the x86_64-pc-windows-gnu target and
# this script wrote a gitignored .cargo/config.toml pointing at rustup's
# bundled MinGW linker (see ADR 0001). That self-contained GNU toolchain later
# broke (dlltool "CreateProcess" failure), and the MSVC linker became
# available, so ADR 0003 switched the project to x86_64-pc-windows-msvc. The
# MSVC target needs no machine-specific cargo config, so this script no longer
# writes one — it only verifies the toolchain can compile.
#
# Usage (from repo root):
#   bash scripts/setup-env.sh
#
# Exits non-zero if the MSVC toolchain cannot compile a trivial program.
set -euo pipefail

command -v rustc >/dev/null 2>&1 || { echo "setup-env: rustc not found on PATH. Install Rust via https://rustup.rs" >&2; exit 1; }

echo "setup-env: rustc = $(rustc --version)"
echo "setup-env: target = x86_64-pc-windows-msvc (see ADR 0003)"

# Verify the MSVC linker + Windows SDK are discoverable by compiling a trivial
# program for the MSVC target. If this fails, install "Visual Studio 2022 Build
# Tools" with the "Desktop development with C++" workload (or the standalone
# MSVC compiler + Windows SDK).
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
printf 'fn main() {}\n' > "$tmpdir/probe.rs"

if ! rustc --edition 2021 --target x86_64-pc-windows-msvc -o "$tmpdir/probe.exe" "$tmpdir/probe.rs" >/dev/null 2>&1; then
  cat >&2 <<'EOF'
setup-env: FAILED to compile for x86_64-pc-windows-msvc.
  The MSVC linker / Windows SDK was not found. Install:
    - Visual Studio 2022 Build Tools with the "Desktop development with C++" workload,
      OR the standalone MSVC compiler + Windows SDK.
  Then rerun this script.
EOF
  exit 1
fi

echo "setup-env: MSVC toolchain OK."
echo "setup-env: no .cargo/config.toml needed for the MSVC target."
echo "setup-env: done."
