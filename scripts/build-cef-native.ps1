[CmdletBinding()]
param(
    [string]$CefRoot = $env:NIVORA_CEF_ROOT,
    [string]$BootstrapPath = $env:NIVORA_CEF_BOOTSTRAP,
    [string]$BuildDirectory = (Join-Path $PSScriptRoot '..\target\cef-native')
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($CefRoot)) {
    throw 'Set NIVORA_CEF_ROOT to the extracted CEF Windows x64 Standard Distribution.'
}

$CefRoot = (Resolve-Path $CefRoot).Path
$Required = @(
    'CMakeLists.txt',
    'cmake\FindCEF.cmake',
    'include\cef_app.h',
    'include\cef_version.h',
    'include\cef_sandbox_win.h',
    'include\cef_version_info.h',
    'Release\libcef.dll',
    'Release\libcef.lib',
    'Resources\icudtl.dat'
)

foreach ($RelativePath in $Required) {
    $Candidate = Join-Path $CefRoot $RelativePath
    if (-not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
        throw "CEF standard distribution is incomplete: $RelativePath"
    }
}

if ([string]::IsNullOrWhiteSpace($BootstrapPath)) {
    $CandidateBootstrap = Join-Path $CefRoot 'Release\bootstrap.exe'
    if (Test-Path -LiteralPath $CandidateBootstrap -PathType Leaf) {
        $BootstrapPath = $CandidateBootstrap
    }
    else {
        throw @'
The CEF 138+ Windows sandbox bootstrap is distributed separately from the standard SDK in some build layouts.
Set NIVORA_CEF_BOOTSTRAP to the matching CEF 156 Windows x64 bootstrap.exe from the sandbox distribution.
The bootstrap must come from the same CEF/Chromium build as NIVORA_CEF_ROOT.
'@
    }
}

$BootstrapPath = (Resolve-Path $BootstrapPath).Path
if ((Split-Path -Leaf $BootstrapPath) -ne 'bootstrap.exe') {
    throw 'NIVORA_CEF_BOOTSTRAP must point to bootstrap.exe.'
}

$VersionHeader = Get-Content (Join-Path $CefRoot 'include\cef_version.h') -Raw
$ExpectedMacros = @{
    CEF_VERSION_MAJOR = '156'
    CEF_VERSION_MINOR = '0'
    CEF_VERSION_PATCH = '14'
    CHROME_VERSION_MAJOR = '156'
    CHROME_VERSION_MINOR = '0'
    CHROME_VERSION_BUILD = '8078'
    CHROME_VERSION_PATCH = '48'
}
foreach ($Macro in $ExpectedMacros.Keys) {
    $Pattern = "(?m)^\s*#define\s+$Macro\s+$($ExpectedMacros[$Macro])\s*$"
    if ($VersionHeader -notmatch $Pattern) {
        throw "CEF SDK version mismatch for $Macro. Expected $($ExpectedMacros[$Macro])."
    }
}

$BuildDirectory = [System.IO.Path]::GetFullPath($BuildDirectory)
$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

# Build the Rust application as a loadable Windows DLL without requiring the
# WinCairo SDK. The bootstrap path uses the Slint shell-only feature set.
& cargo build --manifest-path (Join-Path $RepositoryRoot 'Cargo.toml') -p nivora-app --release --no-default-features --features bootstrap
if ($LASTEXITCODE -ne 0) { throw 'Rust Nivora application DLL build failed.' }

$AppDll = Join-Path $RepositoryRoot 'target\release\nivora_app.dll'
$AppImportLibrary = Join-Path $RepositoryRoot 'target\release\nivora_app.dll.lib'
foreach ($Artifact in @($AppDll, $AppImportLibrary)) {
    if (-not (Test-Path -LiteralPath $Artifact -PathType Leaf)) {
        throw "Rust cdylib build did not produce required artifact: $Artifact"
    }
}

New-Item -ItemType Directory -Force -Path $BuildDirectory | Out-Null

$ConfigureArgs = @(
    '-S', (Join-Path $PSScriptRoot '..\crates\nivora-cef-sys\native'),
    '-B', $BuildDirectory,
    '-A', 'x64',
    "-DCEF_ROOT=$CefRoot",
    "-DNIVORA_APP_IMPORT_LIB=$AppImportLibrary"
)
& cmake @ConfigureArgs
if ($LASTEXITCODE -ne 0) { throw 'CEF CMake configure failed.' }

& cmake --build $BuildDirectory --config Release --target nivora-cef-host
if ($LASTEXITCODE -ne 0) { throw 'CEF native client build failed.' }

$ClientDll = Get-ChildItem -Path $BuildDirectory -Filter 'nivora-cef-host.dll' -File -Recurse |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $ClientDll) {
    throw "Build completed but nivora-cef-host.dll was not found under: $BuildDirectory"
}

$OutputDirectory = Split-Path -Parent $ClientDll
$BootstrapHost = Join-Path $OutputDirectory 'nivora-cef-host.exe'
Copy-Item -LiteralPath $BootstrapPath -Destination $BootstrapHost -Force
Copy-Item -LiteralPath $AppDll -Destination (Join-Path $OutputDirectory 'nivora_app.dll') -Force

Write-Host "Rust application DLL: $(Join-Path $OutputDirectory 'nivora_app.dll')"
Write-Host "CEF bootstrap client DLL: $ClientDll"
Write-Host "CEF bootstrap host: $BootstrapHost"
Write-Host 'Run the bootstrap host to launch the Nivora Slint shell.'
