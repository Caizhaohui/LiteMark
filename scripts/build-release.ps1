param(
    [string]$TargetTriple = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir

Push-Location $repoRoot
try {
    & "$scriptDir\cargo-msvc.ps1" build --release --target $TargetTriple
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    $exePath = Join-Path $repoRoot "target\$TargetTriple\release\litemark.exe"
    if (-not (Test-Path $exePath)) {
        throw "Release executable not found: $exePath"
    }

    Write-Output "Built release executable: $exePath"
}
finally {
    Pop-Location
}
