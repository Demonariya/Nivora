# CEF Windows SDK Spike

**State:** SDK gate scaffold committed. No CEF native browser has been compiled or launched yet.

## Pinned candidate

The initial spike targets the official CEF Windows x64 Standard Distribution:

- CEF: `156.0.14+g4fa4ab5`
- Chromium: `156.0.8078.48`
- Architecture: Windows x64
- Distribution: Standard binary distribution (headers, import library, runtime DLLs, resources and sample projects)

The pin is based on the CEF Automated Builds page's Current Stable Build entry observed on 2026-11-28. The upstream download page lists a 350.17 MB Windows x64 archive for this build. Re-check the official build page before downloading or shipping; CEF/Chromium security fixes require a controlled update cadence.

## Download and local setup

1. Open the official CEF Automated Builds page: https://cef-builds.spotifycdn.com/index.html
2. Select **Windows 64-bit** and download the Standard Distribution for the exact pinned CEF/Chromium versions above.
3. Verify the archive checksum shown by the official download page before extraction.
4. Extract the archive to a stable local path, for example `E:\SDKs\cef\156.0.14\`.
5. Set `NIVORA_CEF_ROOT` to the extracted distribution root (the directory containing `include\`, `Release\`, and `Resources\`).
6. Run `cargo test -p nivora-cef-sys`, then `cargo check --workspace --locked`.

PowerShell example:

```powershell
$env:NIVORA_CEF_ROOT = 'E:\SDKs\cef\156.0.14'
cargo test -p nivora-cef-sys
cargo check --workspace --locked
```

The build script checks for the expected headers, import library, runtime DLL, ICU data, resources, and exact CEF/Chromium version macros. A different SDK version is rejected rather than silently accepted.

## What this scaffold does

- Adds an isolated `nivora-cef-sys` workspace crate.
- Pins CEF and Chromium version constants.
- Validates the expected Windows x64 SDK layout.
- Validates the exact version macros in `cef_version.h`.
- Keeps Linux CI and Windows type-check jobs usable when the large SDK is not installed.
- Exposes `sdk_available()` as a build-time SDK-presence signal.

## What it does not do

This crate does **not** yet:
- link `libcef.dll`;
- compile a C++ bridge;
- initialize CEF;
- start the CEF subprocess;
- create a Chromium browser;
- embed a browser HWND into Slint;
- implement extensions or MV3 service workers.

Those features must be implemented and tested in the next native integration increment. The current WebKit backend remains the only runtime backend.

## Next native acceptance gates

1. Add a minimal CEF C++ host using the exact downloaded binary distribution.
2. Compile the bridge with MSVC and verify required CEF symbols and runtime resources.
3. Initialize CEF and run a separate subprocess executable.
4. Create one child browser window hosted by the existing Win32/Slint window.
5. Navigate to a local test page and an HTTPS site.
6. Verify shutdown, message-loop integration, resize, focus, DPI and crash behavior.
7. Record the actual `chrome://extensions-support` API inventory from this exact build.
8. Only then begin the MV3 extension compatibility tests.

## Upstream references

- Official CEF automated builds: https://cef-builds.spotifycdn.com/index.html
- CEF repository and binary distribution guidance: https://github.com/chromiumembedded/cef
- CEF application layout and runtime resources: https://chromiumembedded.github.io/cef/general_usage
- CEF branch and build requirements: https://github.com/chromiumembedded/cef/blob/master/docs/branches_and_building.md
- CEF security update guidance: https://github.com/chromiumembedded/cef#security-updates
