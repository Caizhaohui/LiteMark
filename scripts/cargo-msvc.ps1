param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [object[]]$CargoArgs
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$withMsvc = Join-Path $scriptDir "with-msvc.ps1"
$allArgs = @("cargo") + @($CargoArgs | ForEach-Object { "$_" })

& $withMsvc @allArgs
exit $LASTEXITCODE
