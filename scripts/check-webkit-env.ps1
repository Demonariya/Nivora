$ErrorActionPreference = 'Stop'
$commands = @('git', 'cmake', 'ninja', 'perl', 'ruby', 'gperf', 'clang-cl', 'cl', 'link', 'dumpbin')
foreach ($name in $commands) {
    $found = Get-Command $name -ErrorAction SilentlyContinue
    if ($found) { Write-Host "OK  $name -> $($found.Source)" -ForegroundColor Green }
    else { Write-Warning "MISSING $name" }
}

try {
    py -3.11 -c "import sys; print('Python', sys.version)"
    if ($LASTEXITCODE -ne 0) { Write-Warning 'Python 3.11 launcher unavailable.' }
} catch { Write-Warning 'Python 3.11 launcher unavailable.' }

if ($env:WEBKIT_INCLUDE_DIR) {
    Write-Host "WEBKIT_INCLUDE_DIR=$env:WEBKIT_INCLUDE_DIR"
    foreach ($header in @('WKBase.h','WKView.h','WKPage.h','WKPageConfigurationRef.h','WKPageNavigationClient.h')) {
        $path = Join-Path $env:WEBKIT_INCLUDE_DIR "WebKit/$header"
        if (-not (Test-Path $path)) { throw "Missing WebKit header: $path" }
    }
} else { Write-Warning 'WEBKIT_INCLUDE_DIR is not set.' }

if ($env:WEBKIT_LIB_DIR) {
    $lib = Join-Path $env:WEBKIT_LIB_DIR 'WebKit2.lib'
    if (-not (Test-Path $lib)) { throw "Missing import library: $lib" }
} else { Write-Warning 'WEBKIT_LIB_DIR is not set.' }
