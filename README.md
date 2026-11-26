# Nivora

A lightweight, privacy-focused Windows browser powered by Rust and WebKit.

Nivora is a native Windows desktop browser using the official WebKit WinCairo port. The UI is built with Rust and Slint. A small C++ ABI bridge calls the upstream WebKit C API and owns the native WKView.

## Current status

The six-crate Rust workspace, Slint shell, SQLite storage primitives, tab model, Rust FFI wrapper, and C++ WinCairo bridge are in source control. The native integration has not yet been compiled against a real Windows WebKit SDK or run on Windows. Do not consider the first-browser milestone accepted until the Windows checklist passes.

## Workspace crates

- nivora-app — executable and lifecycle
- nivora-ui — Slint interface and native HWND host
- nivora-engine — browser-engine facade and URL normalization
- nivora-webkit-sys — Rust FFI and C++ WinCairo bridge
- nivora-storage — SQLite history primitives
- nivora-tabs — in-memory tab model

## Requirements

Windows 11 x64, Rust stable 1.88+, Visual Studio with Desktop development with C++ and Windows SDK, CMake, Ninja, Git, Perl, Python 3.11, Ruby, LLVM, gperf, and a locally built official WebKit WinCairo SDK.

Nivora does not use WebView2, Chromium, Electron, or CEF.

## Build sequence

1. Read docs/webkit-wincairo.md.
2. Run scripts/setup-windows.ps1.
3. Build upstream WebKit with scripts/build-webkit.ps1.
4. Set WEBKIT_INCLUDE_DIR to the include directory containing WebKit/WKBase.h.
5. Set WEBKIT_LIB_DIR to the directory containing WebKit2.lib.
6. Run cargo fmt --all --check, cargo check --workspace, cargo test --workspace, then cargo build --release -p nivora-app.
7. Run scripts/verify-webkit-exports.ps1 against the exact WebKit2.dll used at runtime.

The Windows build fails if the WebKit headers or import library are missing. The non-Windows Rust path reports WinCairo as unavailable; it is not a browser-engine fallback.

## Verification still required

- Rust format, check, and tests on a Rust-enabled host.
- Native MSVC/CMake compilation against a real WinCairo SDK.
- dumpbin export verification against the exact WebKit2.dll.
- HTTPS navigation and Back/Forward/Reload on Windows.
- HWND z-order, focus, keyboard/mouse input, resize, DPI, and destruction tests.

See docs/architecture.md, docs/webkit-wincairo.md, and docs/roadmap.md. WebKit binaries are not vendored. Nivora's original source is MIT-licensed; third-party licenses remain applicable.
