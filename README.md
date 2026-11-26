# Nivora

A lightweight, privacy-focused Windows browser powered by Rust and WebKit.

Nivora uses the official WebKit WinCairo port through its native WKView C API. The UI is Rust + Slint, with a narrow C++ ABI bridge. There is no WebView2, Chromium, Electron, or CEF fallback.

## Workspace

- nivora-app — executable and lifecycle
- nivora-ui — Slint interface and native HWND host
- nivora-engine — browser-engine facade and URL normalization
- nivora-webkit-sys — Rust FFI and C++ WinCairo bridge
- nivora-storage — SQLite history primitives
- nivora-tabs — in-memory tab model

## Build requirements

Windows 11 x64, Rust stable 1.88+, Visual Studio with Desktop development with C++ and Windows SDK, CMake, Ninja, Git, Perl, Python 3.11, Ruby, LLVM, gperf, and a locally built official WebKit WinCairo SDK.

Read docs/webkit-wincairo.md. Run scripts/setup-windows.ps1, then scripts/build-webkit.ps1. Set WEBKIT_INCLUDE_DIR to the include directory containing WebKit/WKBase.h and WEBKIT_LIB_DIR to the directory containing WebKit2.lib. Then run cargo fmt --all --check, cargo check --workspace, cargo test --workspace, and cargo build --release -p nivora-app.

The Windows build fails if required WebKit headers or the import library are missing. The non-Windows Rust path explicitly reports WinCairo as unavailable; it is not a substitute engine.

## Verification status

The workspace and bridge source are committed. Native MSVC compilation, actual WebKit2.dll export verification, HTTPS navigation, HWND z-order/focus/input, resize/DPI, and native teardown still require a Windows runner with the matching WebKit SDK. See docs/roadmap.md for the checklist.

WebKit binaries are not vendored. Nivora's original source is MIT-licensed; third-party licenses remain applicable.