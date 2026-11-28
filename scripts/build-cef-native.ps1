[CmdletBinding()]
param(
    [string]$CefRoot = $env:NIVORA_CEF_ROOT,
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
    'Release\libcef.dll',
    'Release\libcef.lib',
    'Release\cef_sandbox.lib',
    'Resources\icudtl.dat'
)

foreach ($RelativePath in $Required) {
    $Candidate = Join-Path $CefRoot $RelativePath
    if (-not (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
        throw "CEF distribution is incomplete: $RelativePath"
    }
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
New-Item -ItemType Directory -Force -Path $BuildDirectory | Out-Null

$ConfigureArgs = @(
    '-S', (Join-Path $PSScriptRoot '..\crates\nivora-cef-sys\native'),
    '-B', $BuildDirectory,
    '-A', 'x64',
    "-DCEF_ROOT=$CefRoot",
    '-DUSE_SANDBOX=ON'
)
& cmake @ConfigureArgs
if ($LASTEXITCODE -ne 0) { throw 'CEF CMake configure failed.' }

& cmake --build $BuildDirectory --config Release --target nivora-cef-host
if ($LASTEXITCODE -ne 0) { throw 'CEF native host build failed.' }

$Executable = Get-ChildItem -Path $BuildDirectory -Filter 'nivora-cef-host.exe' -File -Recurse |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $Executable) {
    throw "Build completed but nivora-cef-host.exe was not found under: $BuildDirectory"
}

Write-Host "CEF native host built: $Executable"
Write-Host "Run with executable path and --url=https://example.com"
