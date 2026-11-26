$ErrorActionPreference = 'Stop'

$packages = @(
    'Git.Git',
    'Kitware.CMake',
    'Ninja-build.Ninja',
    'Python.Python.3.11',
    'RubyInstallerTeam.Ruby.3.2',
    'LLVM.LLVM',
    'ApacheFriends.Xampp.8.2',
    'GnuWin32.Gperf'
)

foreach ($package in $packages) {
    Write-Host "Installing/checking $package ..." -ForegroundColor Cyan
    winget install --exact --id $package --accept-package-agreements --accept-source-agreements
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "winget returned $LASTEXITCODE for $package. Verify it manually before building WebKit."
    }
}

Write-Host "\`nVisual Studio must include the Desktop development with C++ workload and a Windows SDK." -ForegroundColor Yellow
Write-Host 'Install or modify Visual Studio through Visual Studio Installer if that workload is absent.'
Write-Host 'Open a new Developer PowerShell after installations, then run scripts/check-webkit-env.ps1.'
