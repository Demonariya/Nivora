# CEF Windows SDK and Native Host

**State:** SDK validation and a sandbox-enabled standalone CEF smoke-host source project are committed. The native host has not yet been compiled or launched on Windows.

## Pinned candidate

The initial spike targets the official CEF Windows x64 Standard Distribution:

- CEF: `156.0.14+g4fa4ab5`
- Chromium: `156.0.8078.48`
- Architecture: Windows x64
- Distribution: Standard binary distribution (headers, import library, runtime DLLs, resources, CMake configuration and upstream sample projects)

The pin matches the CEF Automated Builds page's Current Stable Build entry observed on 2026-11-28. The official page lists a 350.17 MB Windows x64 Standard archive. Re-check the official build page before downloading or shipping; CEF/Chromium security fixes require a controlled update cadence.

## Download and local setup

1. Open the [official CEF Automated Builds page](https://cef-builds.spotifycdn.com/index.html).
2. Select **Windows 64-bit** and download the Standard Distribution for the exact pinned versions above.
3. Verify the archive checksum shown by the official download page before extraction.
4. Extract the archive to a stable local path, for example `E:\SDKs\cef\156.0.14`.
5. Set `NIVORA_CEF_ROOT` to the extracted distribution root (the directory containing `include\`, `Release\`, `Resources\`, and `cmake\`).

PowerShell:

```powershell
$env:NIVORA_CEF_ROOT = 'E:\SDKs\cef\156.0.14'
cargo test -p nivora-cef-sys
./scripts/build-cef-native.ps1
```

The Rust SDK gate checks the expected headers, runtime DLL, import library, sandbox library, ICU data, resources, and exact CEF/Chromium version macros. A different SDK version is rejected rather than silently accepted.

## Native smoke host

The standalone native host lives under `crates/nivora-cef-sys/native` and is built with CMake + Visual Studio 2022:

- `src/main.cpp` creates a minimal CEF browser window.
- The host handles browser and child processes through the **same executable**.
- The Windows CEF sandbox is enabled; sandbox information is created in the executable and passed to both `CefExecuteProcess` and `CefInitialize`.
- The CEF message loop runs on the main thread.
- The initial URL defaults to `https://example.com`; override it with `--url=https://...`.
- CEF binaries and resources are copied beside the output executable by the upstream CEF CMake helpers.

Build:

```powershell
./scripts/build-cef-native.ps1
```

Run the resulting host:

```powershell
& .\target\cef-native\out\nivora-cef-host.exe --url=https://example.com
```

The exact output path can vary with the selected CMake generator/configuration; the script checks for the expected executable and prints its path.

## Current implementation boundary

Implemented in source:
- Isolated `nivora-cef-sys` workspace crate and pinned SDK validation.
- CMake project for a Windows x64 native CEF host.
- Minimal CEF app/client lifecycle, URL override, title updates and orderly message-loop shutdown.
- Windows sandbox-enabled process entrypoint using the same executable for CEF child processes.
- PowerShell SDK validation and native build script.

Not implemented or verified yet:
- No Windows/MSVC build result is available yet.
- The standalone CEF host is not yet embedded in the Slint window.
- Rust does not yet initialize CEF or own CEF browser instances.
- The existing WebKit runtime remains the only production backend.
- No extension, MV3, GPU, WebRTC or media compatibility result is claimed.

## Next acceptance gates

1. Build the native host with the exact pinned SDK on Windows x64.
2. Launch it and verify browser + renderer/GPU child process startup with sandbox enabled.
3. Navigate to local and HTTPS pages; verify title updates and clean shutdown.
4. Integrate CEF startup/subprocess handling with the Rust executable and Slint HWND.
5. Verify resize, focus, DPI, crash recovery and message-loop behavior.
6. Record the actual `chrome://extensions-support` API inventory from this exact build.
7. Only then begin the MV3 extension compatibility tests.

## Upstream references

- [Official CEF automated builds](https://cef-builds.spotifycdn.com/index.html)
- [CEF tutorial and Windows build steps](https://chromiumembedded.github.io/cef/tutorial)
- [CEF general usage, process model and message-loop integration](https://chromiumembedded.github.io/cef/general_usage)
- [CEF repository and binary distribution guidance](https://github.com/chromiumembedded/cef)
