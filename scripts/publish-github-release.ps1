param(
    [string]$Version = "v1.0.0",
    [string]$Repo = "Caizhaohui/LiteMark"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$artifact = Join-Path $repoRoot "dist\LiteMark-$Version-windows-x64-portable.zip"
$notes = Join-Path $repoRoot "RELEASE_NOTES_$Version.md"

if (-not (Test-Path $artifact)) {
    throw "Release artifact not found: $artifact"
}

if (-not (Test-Path $notes)) {
    throw "Release notes file not found: $notes"
}

Push-Location $repoRoot
try {
    gh release create $Version $artifact --repo $Repo --title "LiteMark $Version" --notes-file $notes
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    Pop-Location
}
