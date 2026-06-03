param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CommandArgs
)

$ErrorActionPreference = "Stop"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$env:Path = if ($userPath) { "$userPath;$machinePath" } else { $machinePath }

$candidates = @(
    "C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\BuildTools\Common7\Tools\VsDevCmd.bat"
)

$envScript = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $envScript) {
    Write-Error "MSVC environment script not found. Expected one of: $($candidates -join ', ')"
}

$cmdLine = if ($envScript -like "*VsDevCmd.bat") {
    'call "' + $envScript + '" -arch=x64 -host_arch=x64 >nul && set'
} else {
    'call ' + $envScript + ' >nul && set'
}

$envDump = & cmd.exe /d /s /c $cmdLine
foreach ($line in $envDump) {
    if ($line -match "^(.*?)=(.*)$") {
        [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
    Write-Error "MSVC environment loaded, but link.exe is still unavailable."
}

if (-not $CommandArgs -or $CommandArgs.Count -eq 0) {
    Write-Output "MSVC environment loaded for this process."
    Write-Output "Usage: .\scripts\with-msvc.ps1 cargo check"
    exit 0
}

$command = $CommandArgs[0]
$arguments = if ($CommandArgs.Count -gt 1) { $CommandArgs[1..($CommandArgs.Count - 1)] } else { @() }

& $command @arguments
exit $LASTEXITCODE
