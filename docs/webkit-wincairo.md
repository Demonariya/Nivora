# Building and integrating WebKit WinCairo

Nivora targets the official WebKit Windows WinCairo port. It is not WebKitGTK and not WebView2. The Windows-specific WKView API is declared in Source/WebKit/UIProcess/API/C/win/WKView.h.

## Upstream API references

The bridge includes these upstream headers:

- WKView.h — WKViewCreate, WKViewGetWindow, WKViewGetPage, WKViewSetParentWindow, WKViewWindowAncestryDidChange, WKViewSetIsInWindow.
- WKPage.h — URL load, Back/Forward, Reload, close, navigation client registration, active/provisional URL access.
- WKPageConfigurationRef.h — page configuration creation.
- WKPageNavigationClient.h — versioned navigation callback structure.
- WKString.h, WKURL.h, WKType.h — URL/string creation and reference release.

The actual implementation is Source/WebKit/UIProcess/API/C/win/WKView.cpp. WKViewCreate forwards to WebView::create(rect, configuration, parentWindow); the other public functions forward to WebView methods. The bridge uses the public headers rather than handwritten ABI declarations.

## Toolchain

Follow the official WebKit Windows port guide: https://docs.webkit.org/Ports/WindowsPort.html

The Windows build environment requires Visual Studio with Desktop development with C++, a Windows SDK, CMake, Ninja, Git, Perl, Python, Ruby, LLVM, and gperf. Use Python 3.11 for WebKit helper scripts; do not replace another Python installation globally.

Run scripts/setup-windows.ps1 for available package-manager prerequisites. Install the Visual Studio C++ workload and Windows SDK through Visual Studio Installer if missing.

## Build upstream

Run scripts/build-webkit.ps1 -SourceDirectory C:\src\WebKit.

The upstream steps are:
- python Tools\Scripts\update-webkit-win-libs.py
- perl Tools\Scripts\build-webkit --release --skip-library-update

The build is substantial. Wait for the upstream success message: WebKit is now built. The upstream MiniBrowser at WebKitBuild/Release/bin64/MiniBrowser.exe is a useful independent smoke test.

## Locate SDK files

The exact output layout can vary by WebKit revision and build configuration. Locate, do not guess:
- An include directory containing WebKit/WKBase.h and the other public headers.
- WebKit2.lib import library.
- The matching WebKit2.dll and helper processes/runtime DLLs.

Set WEBKIT_INCLUDE_DIR to the include directory and WEBKIT_LIB_DIR to the directory containing WebKit2.lib. The Nivora Windows build script validates every required header and the import library. Missing SDK files are a hard error.

## Verify exports

Header declarations and source implementations do not prove that a particular built DLL exports a symbol. Run scripts/verify-webkit-exports.ps1 against the exact WebKit2.dll that will be used at runtime. It uses Visual Studio dumpbin /exports and checks every WKView/page/string/url/release symbol used by the bridge.

If the import library or DLL does not export a required symbol, stop and reconcile the bridge with that exact WebKit revision. Do not invent declarations or link against a different WebKit build.

## Runtime deployment

Keep WebKit2.dll, support/runtime DLLs, and helper processes from one build together. Ensure the process launcher can resolve helper executables and DLLs. Do not commit WebKit binaries. Review upstream WebKit licensing and notices before redistributing runtime files.
