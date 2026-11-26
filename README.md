# Nivora

[![Made with Slint](https://raw.githubusercontent.com/slint-ui/slint/v1.19.1/logo/MadeWithSlint-logo-whitebg.png)](https://slint.dev/)

A lightweight, privacy-focused Windows browser powered by Rust and WebKit.

Nivora uses the official WebKit WinCairo port through its native WKView C API. The UI is Rust + Slint, with a narrow C++ ABI bridge. There is no WebView2, Chromium, Electron, or CEF fallback.

## Workspace

- nivora-app — executable and lifecycle
- nivora-ui — Slint interface and native HWND host
- nivora-engine — browser-engine facade and URL normalization
- nivora-webkit-sys — Rust FFI and C++ WinCairo bridge
- nivora-storage — SQLite history primitives
- nivora-tabs — in-memory tab model

## Current interface

The native Slint shell now uses a shared Dark/Light token palette, compact top tab treatment, a focused address/search field, minimal Back/Forward/Reload controls, optional vertical placement for the current tab, and a compact Quick Settings popover. Controls include hover, pressed, disabled, and focus feedback, short transitions, and accessible labels/actions.

The tab-placement switch changes the presentation of the current tab only. Multi-tab WebKit ownership, tab drag-and-drop, automatic Windows appearance detection, bookmark/history/download panels, suggestions, favicons, TLS/security indicators, and real percentage-based loading progress are not yet implemented. These are intentionally not represented by non-functional controls.

See [the UI design system](docs/ui-design-system.md) for the palette, component inventory, interaction behavior, and native child-window constraints.

## Build requirements

Windows 11 x64, Rust stable 1.88+, Visual Studio with Desktop development with C++ and Windows SDK, CMake, Ninja, Git, Perl, Python 3.11, Ruby, LLVM, gperf, and a locally built official WebKit WinCairo SDK.

Read docs/webkit-wincairo.md. Run scripts/setup-windows.ps1, then scripts/build-webkit.ps1. Set WEBKIT_INCLUDE_DIR to the include directory containing WebKit/WKBase.h, WEBKIT_LIB_DIR to the directory containing WebKit2.lib, and WEBKIT_RUNTIME_DIR to the directory containing the matching WebKit2.dll for native CI export verification. Then run cargo fmt --all --check, cargo check --workspace, cargo test --workspace, and cargo build --release -p nivora-app. Set WEBKIT_RUNTIME_DIR to the matching runtime folder and launch with scripts/run-nivora.ps1 so WebKit2.dll and its helper processes are discoverable.

The Windows build fails if required WebKit headers or the import library are missing. The non-Windows Rust path explicitly reports WinCairo as unavailable; it is not a substitute engine. CI also performs a Windows-target Rust type-check; that explicit check-only mode does not compile or link the C++ bridge.

## Validation gate

CI installs Slint's Linux native build dependencies, verifies Rust formatting, checks the host workspace, runs unit tests, and type-checks Windows Rust code. Native MSVC compilation, actual WebKit2.dll export verification, HTTPS navigation, HWND z-order/focus/input, resize/DPI, and native teardown still require a Windows runner with the matching WebKit SDK. See docs/roadmap.md for the acceptance checklist.

WebKit binaries are not vendored. Nivora's original source is MIT-licensed; third-party licenses remain applicable. See [third-party license notes](docs/third-party-licenses.md) before redistribution.


## Multi-tab implementation

The browser shell now has a real tab model backed by independent WebKit WinCairo views. Each opened tab lazily creates its own WKView/WKPage, retains its navigation state while hidden, and is shown again when selected. Closing a tab destroys its native view before releasing its navigation callback context. This is source-level implementation; native WinCairo runtime acceptance still requires a Windows runner provisioned with the matching WebKit SDK and runtime.


### Page metadata and loading feedback

The WinCairo bridge subscribes to WebKit page-load state changes. Main-page title updates are stored per tab and reflected in the tab strip, including updates received while a tab is in the background. The toolbar loading indicator uses WebKit's estimated page-load progress rather than a simulated timer. Favicon retrieval is not included yet because it requires a separately verified icon/resource API path for the selected WinCairo SDK.


### Keyboard shortcuts

Nivora registers a UI-thread-scoped Win32 message hook so browser shortcuts remain available while a native WebKit child window owns keyboard focus:

| Shortcut | Action |
| --- | --- |
| Ctrl+T | Open a new tab |
| Ctrl+W | Close the active tab |
| Ctrl+Tab | Switch to the next tab |
| Ctrl+Shift+Tab | Switch to the previous tab |

The hook consumes only these browser shortcuts and is removed before its callback context is released. Native WinCairo runtime behavior still requires validation on a Windows machine with the matching SDK.
