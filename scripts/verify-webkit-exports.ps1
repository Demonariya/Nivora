param(
    [Parameter(Mandatory = $true)]
    [string]$WebKitDll
)
$ErrorActionPreference = 'Stop'
$dll = (Resolve-Path $WebKitDll).Path
$dumpbin = Get-Command dumpbin -ErrorAction Stop
$output = & $dumpbin.Source /nologo /exports $dll 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) { throw "dumpbin failed for $dll\`n$output" }

$required = @(
    'WKViewCreate',
    'WKViewGetWindow',
    'WKViewGetPage',
    'WKViewSetParentWindow',
    'WKViewWindowAncestryDidChange',
    'WKViewSetIsInWindow',
    'WKPageConfigurationCreate',
    'WKPageSetPageNavigationClient',
    'WKFramePolicyListenerUse',
    'WKPageLoadURL',
    'WKPageGoBack',
    'WKPageGoForward',
    'WKPageCanGoBack',
    'WKPageCanGoForward',
    'WKPageReload',
    'WKPageCopyActiveURL',
    'WKPageCopyProvisionalURL',
    'WKPageClose',
    'WKURLCreateWithUTF8CString',
    'WKURLCopyString',
    'WKStringGetMaximumUTF8CStringSize',
    'WKStringGetUTF8CString',
    'WKRelease'
)
$missing = @()
foreach ($symbol in $required) {
    if ($output -notmatch "(?m)\b$([regex]::Escape($symbol))\b") { $missing += $symbol }
}
if ($missing.Count -gt 0) {
    throw "The selected WebKit2.dll does not export all APIs used by Nivora. Missing: $($missing -join ', ')"
}
Write-Host "Verified $($required.Count) required exports in $dll" -ForegroundColor Green
