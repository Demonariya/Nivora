# CEF Windows SDK and Native Bootstrap Client

**State:** The Rust application cdylib and native bootstrap-to-Slint handoff are implemented in source. The native DLLs have not yet been compiled or launched on Windows.

## Pinned candidate

The initial spike targets the official CEF Windows x64 Standard Distribution:

- CEF: `156.0.14+g4fa4ab5`
- Chromium: `156.0.8078.48`
- Architecture: Windows x64
- Distribution: Standard binary distribution (headers, import library, runtime DLLs, resources and CMake configuration)

The pin matches the CEF Automated Builds page's Current Stable Build entry observed on 2026-11-28. Re-check the official build page before downloading or shipping; CEF/Chromium security fixes require a controlled update cadence.

## Download and local setup

1. Open the [official CEF Automated Builds page](https://cef-builds.spotifycdn.com/index.html).
2. Select **Windows 64-bit** and download the Standard Distribution for the exact pinned versions above.
3. Verify the archive checksum shown by the official download page before extraction.
4. Extract the archive to a stable local path, for example `E:\\SDKs\\cef\\156.0.14`.
5. Obtain the matching Windows x64 **sandbox bootstrap.exe** for the same CEF/Chromium build. Depending on the official build layout, this may be supplied as the separate Sandbox Distribution; it is not assumed to be present in the Standard Distribution. Do not mix bootstrap and SDK versions.
6. Extract the bootstrap and set `NIVORA_CEF_BOOTSTRAP` to its full path.
7. Set `NIVORA_CEF_ROOT` to the standard distribution root.

PowerShell:

```powershell
$env:NIVORA_CEF_ROOT = 'E:\SDKs\cef\156.0.14'
$env:NIVORA_CEF_BOOTSTRAP = 'E:\SDKs\cef\156.0.14-sandbox\Release\bootstrap.exe'
cargo test -p nivora-cef-sys
./scripts/build-cef-native.ps1
```

The build script validates the pinned SDK, builds the Rust cdylib in shell-only mode, verifies the DLL/import-library artifacts, builds the CEF bootstrap client DLL, then stages the official bootstrap and Rust DLL together.

## Windows sandbox architecture (CEF 138+)

CEF 138 and newer use the official bootstrap model for sandboxed Windows clients:

- The Nivora native client is built as `nivora-cef-host.dll`.
- The matching official `bootstrap.exe` is copied beside it and renamed to `nivora-cef-host.exe`.
- The bootstrap creates sandbox information in its own executable and passes it to the exported `RunWinMain` entry point.
- The client forwards that pointer to `CefExecuteProcess` before starting the Rust browser-process UI.
- Do not create sandbox information inside the client DLL, link the removed legacy sandbox library, or configure a separate `browser_subprocess_path` for this sandboxed mode.

The Rust application crate builds both the normal `nivora.exe` and `nivora_app.dll` (`cdylib`). The native client links against `nivora_app.dll.lib` and calls the exported `nivora_rust_run_ui` function after CEF child-process dispatch.

## Feature separation

- Normal desktop builds use the default `webkit` feature and retain the existing WinCairo implementation.
- The bootstrap build uses `cargo build -p nivora-app --release --no-default-features --features bootstrap`.
- Shell-only bootstrap builds do not compile or link `nivora-webkit-sys`; they do not require the separate WinCairo SDK.
- The shell-only Slint window clearly indicates that CEF browser rendering is not connected yet.

## Current implementation boundary

Implemented in source:

- Pinned CEF SDK validation and native CMake project.
- Rust app `cdylib` and stable C ABI entrypoint.
- Optional WebKit feature so the bootstrap shell can be built independently of WinCairo.
- CEF bootstrap `RunWinMain` and `CefExecuteProcess` child-process dispatch.
- C++ client linkage against the Rust import library.
- PowerShell build and staging of bootstrap executable, CEF client DLL and Rust application DLL.

Not implemented or verified yet:

- No Windows/MSVC build or launch result is available yet.
- Browser-process CEF initialization (`CefInitialize`) is deliberately not called in this handoff milestone.
- No CEF browser/renderer/GPU child process is created by Nivora yet.
- CEF message-loop integration and browser HWND embedding into Slint are not implemented.
- The existing WebKit runtime remains the only production rendering backend.
- No extension, MV3, GPU, WebRTC or media compatibility result is claimed.

## Next acceptance gates

1. Provision the exact pinned Standard SDK and matching bootstrap binary on Windows x64.
2. Build the Rust cdylib and native client DLL with MSVC.
3. Launch the bootstrap and verify that it loads `nivora_app.dll` and displays the Slint shell.
4. Verify child-process dispatch does not start a second Slint instance.
5. Add browser-process `CefInitialize` with the bootstrap-provided sandbox pointer.
6. Integrate CEF's message pump with Slint and embed a CEF child HWND in the Slint window.
7. Verify navigation, title updates, resize, focus, DPI and clean shutdown.
8. Only then begin extension compatibility tests.

## Upstream references

- [Official CEF automated builds](https://cef-builds.spotifycdn.com/index.html)
- [CEF Windows sandbox setup](https://chromiumembedded.github.io/cef/sandbox_setup)
- [CEF 156 sandbox header and RunWinMain signature](https://cef-builds.spotifycdn.com/docs/156.0/cef__sandbox__win_8h.html)
- [CEF tutorial and Windows build steps](https://chromiumembedded.github.io/cef/tutorial)
- [CEF general usage, process model and message-loop integration](https://chromiumembedded.github.io/cef/general_usage)
