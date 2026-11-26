# Building and integrating WebKit WinCairo

Nivora targets the official WebKit Windows **WinCairo** port. WinCairo is a native Windows port, not WebKitGTK and not WebView2. Its C API includes the Windows-specific `WKView` API in `Source/WebKit/UIProcess/API/C/win/WKView.h`.

## Upstream API references

The bridge is written against these upstream headers:

- `WKView.h` — `WKViewCreate`, `WKViewGetWindow`, `WKViewGetPage`, `WKViewSetParentWindow`, `WKViewWindowAncestryDidChange`, `WKViewSetIsInWindow`.
- `WKPage.h` — URL load, Back/Forward, Reload, close, navigation client registration, and active/provisional URL access.
- `WKPageConfigurationRef.h` — configuration creation.
- `WKPageNavigationClient.h` — versioned navigation callback layout.
- `WKFramePolicyListener.h` — explicit policy approval for ordinary navigation actions and responses.
- `WKString.h`, `WKURL.h`, `WKType.h` — URL/string creation and reference release.

The actual upstream implementation of the Windows view API is `Source/WebKit/UIProcess/API/C/win/WKView.cpp`. `WKViewCreate` calls `WebView::create(rect, configuration, parentWindow)` and the other exported functions forward to the corresponding `WebView` methods. The implementation is checked in upstream; Nivora does not invent its signatures.

The current upstream declarations in `WKView.h` are:

```cpp
WK_EXPORT WKViewRef WKViewCreate(RECT rect, WKPageConfigurationRef pageConfiguration, HWND parentWindow);
WK_EXPORT HWND WKViewGetWindow(WKViewRef view);
WK_EXPORT WKPageRef WKViewGetPage(WKViewRef view);
WK_EXPORT void WKViewSetParentWindow(WKViewRef view, HWND parentWindow);
WK_EXPORT void WKViewWindowAncestryDidChange(WKViewRef view);
WK_EXPORT void WKViewSetIsInWindow(WKViewRef view, bool isInWindow);
```

The bridge also uses the declarations `WKPageConfigurationCreate(void)`, `WKPageLoadURL(WKPageRef, WKURLRef)`, `WKPageGoBack(WKPageRef)`, `WKPageGoForward(WKPageRef)`, `WKPageCanGoBack(WKPageRef)`, `WKPageCanGoForward(WKPageRef)`, `WKPageReload(WKPageRef)`, `WKPageClose(WKPageRef)`, and `WKPageSetPageNavigationClient(WKPageRef, const WKPageNavigationClientBase*)`. The navigation client explicitly calls `WKFramePolicyListenerUse` for action and response decisions rather than leaving those policy callbacks unset.

These signatures were checked against the current upstream headers. The export table of the exact `WebKit2.dll` used for a Nivora build has **not** been verified in this environment; `scripts/verify-webkit-exports.ps1` performs that separate binary-level check.

## Toolchain

Follow the current [official WebKit Windows port guide](https://docs.webkit.org/Ports/WindowsPort.html). The guide's Windows build environment includes Visual Studio with Desktop development with C++, CMake, Ninja, Git, Perl, Python, Ruby, LLVM, and `gperf`. Use Python 3.11 for the WebKit helper scripts; do not replace another Python installation globally.

The helper script `scripts/setup-windows.ps1` installs available package-manager prerequisites. Visual Studio's C++ workload and Windows SDK must be installed through Visual Studio Installer if they are missing.

## Build upstream

```powershell
./scripts/build-webkit.ps1 -SourceDirectory C:\src\WebKit
```

The script runs the upstream Windows library update and release build steps:

```powershell
python Tools\Scripts\update-webkit-win-libs.py
perl Tools\Scripts\build-webkit --release --skip-library-update
```

The upstream build is substantial. Wait for the success message `WebKit is now built`. The upstream MiniBrowser is a useful independent smoke test at `WebKitBuild/Release/bin64/MiniBrowser.exe`.

## Locate SDK files

The exact output layout can vary with WebKit revision and build configuration. Locate, do not guess, these files:

- Public headers under an include directory containing `WebKit/WKBase.h` and the other headers listed above.
- `WebKit2.lib` import library.
- The matching `WebKit2.dll` and helper processes/runtime DLLs.

Set:

```powershell
$env:WEBKIT_INCLUDE_DIR = 'C:\path\to\include'
$env:WEBKIT_LIB_DIR = 'C:\path\to\directory\with\WebKit2.lib'
```

The build script validates every required header and the import library. A missing SDK is a hard error for Windows targets.

## Verify exports

A header declaration and source implementation do not prove that a specific built DLL exports a symbol. Run `scripts/verify-webkit-exports.ps1` against the exact `WebKit2.dll` that will ship with the app. The check uses `dumpbin /exports` and requires all `WKView` symbols used by the bridge plus page/navigation functions.

If the build's import library or DLL does not export one of these symbols, stop and reconcile the bridge with that exact WebKit revision. Do not add handwritten declarations or link against a different WebKit build.

## Runtime deployment

Keep `WebKit2.dll`, WebKit support/runtime DLLs, and WebKit helper processes from the same build together. Set `WEBKIT_RUNTIME_DIR` to that directory and launch with `scripts/run-nivora.ps1`; the script adds the matching runtime directory to `PATH` before starting Nivora. Ensure the process launcher can resolve the helper executables and DLLs. Do not copy WebKit binaries into the Git repository. Review the WebKit license and notices before redistributing any runtime package.
