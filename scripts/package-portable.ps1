param(
    [string]$Version = "1.0.0",
    [string]$TargetTriple = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$distRoot = Join-Path $repoRoot "dist"
$packageName = "LiteMark-v$Version-windows-x64-portable"
$packageDir = Join-Path $distRoot $packageName
$zipPath = Join-Path $distRoot "$packageName.zip"
$exePath = Join-Path $repoRoot "target\$TargetTriple\release\litemark.exe"

Push-Location $repoRoot
try {
    & "$scriptDir\build-release.ps1" -TargetTriple $TargetTriple
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    if (Test-Path $packageDir) {
        Remove-Item -Recurse -Force $packageDir
    }
    if (Test-Path $zipPath) {
        Remove-Item -Force $zipPath
    }

    New-Item -ItemType Directory -Path $packageDir -Force | Out-Null

    Copy-Item $exePath (Join-Path $packageDir "litemark.exe")
    Copy-Item (Join-Path $repoRoot "README.md") (Join-Path $packageDir "README.md")
    Copy-Item (Join-Path $repoRoot ".litemark.toml.example") (Join-Path $packageDir ".litemark.toml.example")

    Compress-Archive -Path "$packageDir\*" -DestinationPath $zipPath

    Write-Output "Portable package directory: $packageDir"
    Write-Output "Portable package zip: $zipPath"
}
finally {
    Pop-Location
}
