# scripts/setup-env.ps1
#
# One-time environment sanity check for LiteMark on Windows.
#
# History: LiteMark originally built with the *GNU* target
# (x86_64-pc-windows-gnu) so that no MSVC C++ build tools were required (see
# ADR 0001), and this script wrote a gitignored .cargo/config.toml pointing at
# rustup's bundled MinGW linker. That self-contained GNU toolchain later broke
# (dlltool "CreateProcess" failure), and the MSVC linker became available, so
# ADR 0003 switched the project to x86_64-pc-windows-msvc. The MSVC target
# needs no machine-specific cargo config, so this script no longer writes one
# — it only verifies the toolchain can compile.
#
# Usage (from repo root):
#   pwsh -File scripts/setup-env.ps1
#   # or, in PowerShell:
#   ./scripts/setup-env.ps1
#
# Exits non-zero if the MSVC toolchain cannot compile a trivial program.

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    throw "rustc not found on PATH. Install Rust via https://rustup.rs first."
}

Write-Host "setup-env: rustc = $(& rustc --version)"
Write-Host "setup-env: target = x86_64-pc-windows-msvc (see ADR 0003)"

# Verify the MSVC linker + Windows SDK are discoverable by compiling a trivial
# program for the MSVC target. If this fails, install "Visual Studio 2022 Build
# Tools" with the "Desktop development with C++" workload (or the standalone
# MSVC compiler + Windows SDK).
$tmpDir = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath() + "litemark-setup-$(Get-Random)") -Force
try {
    Set-Content -Path (Join-Path $tmpDir "probe.rs") -Value "fn main() {}`n" -Encoding utf8
    $probeExe = Join-Path $tmpDir "probe.exe"
    & rustc --edition 2021 --target x86_64-pc-windows-msvc -o $probeExe (Join-Path $tmpDir "probe.rs") 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw @"
setup-env: FAILED to compile for x86_64-pc-windows-msvc.
  The MSVC linker / Windows SDK was not found. Install:
    - Visual Studio 2022 Build Tools with the "Desktop development with C++" workload,
      OR the standalone MSVC compiler + Windows SDK.
  Then rerun this script.
"@
    }
}
finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}

Write-Host "setup-env: MSVC toolchain OK."
Write-Host "setup-env: no .cargo/config.toml needed for the MSVC target."
Write-Host "setup-env: done."
