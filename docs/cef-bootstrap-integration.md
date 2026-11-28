# CEF Bootstrap Integration Plan (Windows)

**Status:** Rust cdylib entrypoint and bootstrap-to-Slint shell startup are implemented in source. Native build and Windows launch remain unverified. CEF browser initialization and browser HWND embedding are not implemented.

## Current process layout

```text
nivora-cef-host.exe          Official CEF bootstrap executable
  └── nivora-cef-host.dll    Native CEF bootstrap client
       ├── exported RunWinMain
       ├── CefExecuteProcess (child-process dispatch)
       └── imports nivora_app.dll
            └── nivora_rust_run_ui() C ABI
                 └── nivora_ui::run()
                      └── Slint MainWindow
```

The Rust application crate now produces both the existing `nivora.exe` binary and a `nivora_app.dll` cdylib. The C++ bootstrap client links against the cdylib import library and calls its stable C ABI export. Rust panics and UI startup errors are contained at the FFI boundary and translated to integer process exit codes.

## Feature separation

- The normal desktop executable retains the default `webkit` feature and existing WebKit WinCairo behavior.
- The bootstrap build uses `--no-default-features --features bootstrap`.
- This shell-only mode does not compile or link the WebKit native bridge and therefore does not require the WinCairo SDK.
- The shell-only status text explicitly says that CEF browser rendering is not connected yet.

This avoids forcing the CEF bootstrap build to provision two unrelated browser-engine SDKs while the CEF view integration is still pending.

## Current startup sequence

1. The official bootstrap starts and creates sandbox information.
2. Bootstrap loads `nivora-cef-host.dll` and calls its exported `RunWinMain`.
3. The C++ entrypoint passes the bootstrap-provided sandbox pointer to `CefExecuteProcess`.
4. If this is a CEF child process, CEF handles it and the returned exit code is propagated without starting Rust or Slint.
5. If this is the browser process, `CefExecuteProcess` returns the browser-process sentinel and the client calls `nivora_rust_run_ui`.
6. The Rust cdylib starts the Slint shell on the calling UI thread.
7. Slint exits; the Rust export returns an exit code to the bootstrap.

**Important boundary:** this milestone deliberately does not call `CefInitialize`. It validates the bootstrap process dispatch and Rust/Slint DLL handoff only. No CEF browser, renderer or GPU child is created by Nivora at this stage. The next CEF integration change must add browser-process initialization, message-loop scheduling and a browser view parented to the Slint HWND as one coordinated lifecycle.

## Build and staging

PowerShell environment:

```powershell
$env:NIVORA_CEF_ROOT = 'E:\SDKs\cef\156.0.14'
$env:NIVORA_CEF_BOOTSTRAP = 'E:\SDKs\cef\156.0.14-sandbox\Release\bootstrap.exe'
./scripts/build-cef-native.ps1
```

The build script:

1. Validates the pinned CEF 156.0.14 / Chromium 156.0.8078.48 Standard SDK.
2. Builds `nivora-app` as a release cdylib with the shell-only feature set.
3. Verifies `target/release/nivora_app.dll` and `target/release/nivora_app.dll.lib`.
4. Configures and builds the CEF bootstrap client DLL against the Rust import library.
5. Stages the matching official bootstrap as `nivora-cef-host.exe`.
6. Copies `nivora_app.dll` beside the bootstrap and client DLL.

The CEF Windows CI job is gated behind the pre-provisioned self-hosted runner and the `CEF_SDK_READY` repository variable. The runner must have the CEF SDK and matching bootstrap available through `NIVORA_CEF_ROOT` and `NIVORA_CEF_BOOTSTRAP`.

## Remaining integration work

- [ ] Compile Rust cdylib and native client DLL with MSVC on Windows x64.
- [ ] Launch bootstrap and verify the Rust cdylib is loaded and Slint window appears.
- [ ] Verify CEF child-process dispatch never starts Slint in child processes.
- [ ] Initialize CEF in browser-process mode with the bootstrap-provided sandbox pointer.
- [ ] Integrate CEF message pumping with the Slint event loop.
- [ ] Obtain the Slint HWND and create a CEF child browser view within it.
- [ ] Verify resize, focus, keyboard input, DPI, tab visibility and shutdown.
- [ ] Keep WebKit as the production fallback until CEF acceptance tests pass.

## Upstream references

- [CEF Windows sandbox setup](https://chromiumembedded.github.io/cef/sandbox_setup)
- [CEF 156 RunWinMain API](https://cef-builds.spotifycdn.com/docs/156.0/cef__sandbox__win_8h.html)
- [CEF 156 CefExecuteProcess / CefInitialize](https://cef-builds.spotifycdn.com/docs/156.0/cef__app_8h.html)
- [CEF general usage and message-loop integration](https://chromiumembedded.github.io/cef/general_usage)
