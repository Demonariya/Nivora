param(
    [string]$RuntimeDirectory = $env:WEBKIT_RUNTIME_DIR,
    [string]$Executable = (Join-Path $PSScriptRoot '..\target\release\nivora.exe')
)
$ErrorActionPreference = 'Stop'

if (-not $RuntimeDirectory) {
    throw 'Set WEBKIT_RUNTIME_DIR or pass -RuntimeDirectory to the directory containing the matching WebKit2.dll and WebKit helper processes.'
}
$runtime = (Resolve-Path $RuntimeDirectory).Path
$webkitDll = Join-Path $runtime 'WebKit2.dll'
if (-not (Test-Path $webkitDll)) {
    throw "WebKit2.dll was not found in WEBKIT_RUNTIME_DIR: $runtime"
}
$exe = (Resolve-Path $Executable).Path

# WebKit2.dll and its helper processes must come from the same WinCairo build.
$env:PATH = "$runtime;$env:PATH"
& $exe
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
