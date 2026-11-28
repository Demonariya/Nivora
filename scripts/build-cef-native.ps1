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

$Executable = Join-Path $BuildDirectory 'out\nivora-cef-host.exe'
if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
    throw "Build completed but expected executable was not found: $Executable"
}

Write-Host "CEF native host built: $Executable"
Write-Host "Run with executable path and --url=https://example.com"
