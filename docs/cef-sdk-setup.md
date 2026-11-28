# CEF Windows SDK and Native Client

**State:** SDK validation and a bootstrap-based native CEF client DLL source project are committed. The client DLL has not yet been compiled or launched on Windows.

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
5. Set `NIVORA_CEF_ROOT` to the extracted distribution root (the directory containing `include\\`, `Release\\`, `Resources\\`, and `cmake\\`).

PowerShell:

```powershell
$env:NIVORA_CEF_ROOT = 'E:\SDKs\cef\156.0.14'
cargo test -p nivora-cef-sys
./scripts/build-cef-native.ps1
```

The Rust SDK gate checks the expected headers, runtime DLL, import library, official bootstrap executable, ICU data, resources, and exact CEF/Chromium version macros. A different SDK version is rejected rather than silently accepted.

## Windows sandbox architecture (CEF 138+)

CEF 138 and newer no longer distribute `cef_sandbox.lib` for ordinary third-party client linking. The supported Windows sandbox integration uses the official `bootstrap.exe`:

- The Nivora CEF client is built as `nivora-cef-host.dll`.
- The official `Release\\bootstrap.exe` is copied beside it and renamed to `nivora-cef-host.exe`.
- The bootstrap creates sandbox information in its own executable and passes it to the exported `RunWinMain` entry point.
- The client DLL forwards that pointer to both `CefExecuteProcess` and `CefInitialize`.
- The bootstrap executable remains the process image for the browser and sandboxed child processes.
- Do not create sandbox information inside the client DLL, link the removed legacy sandbox library, or configure a separate `browser_subprocess_path` for this sandboxed mode.

This is materially different from the pre-CEF-138 integration model. The CEF SDK version pin and CMake project must stay aligned with this bootstrap contract.

## Native smoke client

The native source lives under `crates/nivora-cef-sys/native` and is built with CMake + Visual Studio 2022:

- `src/main.cpp` exports the bootstrap-compatible `RunWinMain` entry point.
- The client calls `CefExecuteProcess` before browser-process initialization.
- The browser process opens a minimal CEF window; the default URL is `https://example.com`.
- `--url=https://...` overrides the initial URL.
- The CEF message loop runs on the main thread and shutdown is orderly.
- CEF runtime binaries and resources are staged beside the client using upstream CEF CMake helpers.

Build:

```powershell
./scripts/build-cef-native.ps1
```

Run the staged bootstrap executable:

```powershell
& .\target\cef-native\out\nivora-cef-host.exe --url=https://example.com
```

The exact output path can vary with the CMake generator. The build script finds the DLL, stages the matching bootstrap executable beside it, and prints both paths.

## Current implementation boundary

Implemented in source:

- Isolated `nivora-cef-sys` workspace crate and pinned SDK validation.
- CMake project that builds a bootstrap-compatible Windows x64 client DLL.
- Exported `RunWinMain` implementation using the sandbox pointer supplied by the official bootstrap.
- PowerShell SDK validation, native build, and bootstrap staging.

Not implemented or verified yet:

- No Windows/MSVC build result is available yet.
- The client DLL is still a standalone smoke test, not embedded in the Slint window.
- Rust does not yet dispatch CEF subprocess startup or own CEF browser instances.
- The existing WebKit runtime remains the only production backend.
- No extension, MV3, GPU, WebRTC or media compatibility result is claimed.

## Next acceptance gates

1. Build the native client DLL with the exact pinned SDK on Windows x64.
2. Confirm the SDK contains the expected official bootstrap binary and run the staged bootstrap/client pair.
3. Verify browser + renderer/GPU child process startup with the sandbox enabled.
4. Navigate to local and HTTPS pages; verify title updates and clean shutdown.
5. Design and implement Rust executable startup dispatch through the bootstrap architecture.
6. Integrate CEF browser HWND ownership, resize, focus, DPI and message-loop scheduling with Slint.
7. Record the actual `chrome://extensions-support` API inventory from this exact build.
8. Only then begin MV3 extension compatibility tests.

## Upstream references

- [Official CEF automated builds](https://cef-builds.spotifycdn.com/index.html)
- [CEF Windows sandbox setup (bootstrap architecture)](https://chromiumembedded.github.io/cef/sandbox_setup)
- [CEF 156 sandbox header and RunWinMain signature](https://cef-builds.spotifycdn.com/docs/156.0/cef__sandbox__win_8h.html)
- [CEF tutorial and Windows build steps](https://chromiumembedded.github.io/cef/tutorial)
- [CEF general usage, process model and message-loop integration](https://chromiumembedded.github.io/cef/general_usage)
