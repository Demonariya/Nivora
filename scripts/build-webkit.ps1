param(
    [string]$SourceDirectory = (Join-Path $PSScriptRoot '..\WebKit'),
    [string]$Python311 = 'py'
)
$ErrorActionPreference = 'Stop'

$source = [System.IO.Path]::GetFullPath($SourceDirectory)
if (-not (Test-Path (Join-Path $source '.git'))) {
    if (Test-Path $source) { throw "SourceDirectory exists but is not a WebKit Git checkout: $source" }
    git clone https://github.com/WebKit/WebKit.git $source
    if ($LASTEXITCODE -ne 0) { throw 'git clone failed.' }
}

Push-Location $source
try {
    git submodule update --init --recursive
    if ($LASTEXITCODE -ne 0) { throw 'WebKit submodule initialization failed.' }

    & $Python311 -3.11 -c "import sys; print(sys.version); assert sys.version_info[:2] == (3, 11)"
    if ($LASTEXITCODE -ne 0) { throw 'Python 3.11 is required for the upstream WebKit helper scripts. Set -Python311 to a Python launcher that provides -3.11.' }

    & $Python311 -3.11 Tools\Scripts\update-webkit-win-libs.py
    if ($LASTEXITCODE -ne 0) { throw 'update-webkit-win-libs.py failed.' }

    perl Tools\Scripts\build-webkit --release --skip-library-update
    if ($LASTEXITCODE -ne 0) { throw 'The upstream WebKit WinCairo release build failed.' }
} finally {
    Pop-Location
}

Write-Host 'WebKit build command completed. Locate the matching headers, WebKit2.lib, DLLs, and helper processes before configuring Nivora.' -ForegroundColor Green
