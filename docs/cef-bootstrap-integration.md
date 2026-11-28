# CEF Bootstrap Integration Plan (Windows)

**Status:** Architecture decision recorded; Rust/Slint integration is not implemented.

## Why the process model changes

The pinned CEF 156 distribution follows the CEF 138+ Windows sandbox model. The official bootstrap.exe is the executable image. It creates sandbox information and loads a client DLL, then calls the DLL's exported RunWinMain(HINSTANCE, LPWSTR, int, void*, cef_version_info_t*) entrypoint. The client must forward the supplied sandbox pointer to both CefExecuteProcess and CefInitialize.

Consequences:

- Do not link the removed legacy cef_sandbox.lib into a third-party Rust executable.
- Do not create sandbox information from inside the client DLL.
- Do not set browser_subprocess_path for this sandboxed bootstrap mode.
- The matching bootstrap executable and client DLL must be deployed together.
- The browser and child-process startup path must be tested with the exact pinned CEF distribution.

## Intended integrated process layout

```text
nivora-cef-host.exe          Official CEF bootstrap executable
  └── nivora.dll             Rust application DLL loaded by bootstrap
       ├── exported RunWinMain entrypoint
       ├── Rust startup / Slint UI
       └── CEF ABI boundary (native C++ implementation)
            ├── CefExecuteProcess
            ├── CefInitialize
            ├── browser/client handlers
            ├── browser HWND lifecycle
            └── CefDoMessageLoopWork scheduling
```

The currently committed nivora-cef-host.dll is only a native smoke-test client. It creates a standalone CEF popup window and proves neither Rust integration nor Slint embedding. It is not the final application DLL.

The Rust application should become a library artifact that can be loaded by the bootstrap (for example, a cdylib with a bootstrap-compatible exported entrypoint), while the current executable may remain a separate development launcher. The exact Cargo target layout should be implemented only with a Windows/MSVC build test in place.

## Startup sequence

1. Bootstrap starts and creates sandbox information.
2. Bootstrap loads the Nivora client DLL and calls RunWinMain.
3. The Nivora entrypoint calls the CEF process-dispatch function before creating Slint.
4. If this is a renderer/GPU/utility child process, CEF handles it and the entrypoint returns its exit code without starting Slint.
5. If this is the browser process, initialize CEF with the supplied sandbox pointer.
6. Create the Slint top-level window and obtain its Win32 HWND.
7. Create CEF browser views as child windows of the Slint host window.
8. Pump CEF work through the same UI thread as Slint.
9. Close browser instances, complete CEF shutdown, then tear down Slint and return from the bootstrap DLL entrypoint.

The precise shutdown order must be validated against CEF's callback requirements. No CEF API may be called after CefShutdown.

## Message-loop decision

Do not call CefRunMessageLoop on the same thread that runs Slint's ComponentHandle::run(); both functions own a blocking event loop.

The preferred integration candidate is:

- multi_threaded_message_loop = false
- external_message_pump = true
- Handle OnScheduleMessagePumpWork in the browser-process handler.
- Schedule CefDoMessageLoopWork on the Slint/UI thread at the requested delay.
- Coalesce redundant pump requests and cancel pending work during shutdown.

A fixed high-frequency timer may be used only as a temporary diagnostic fallback. The production pump must avoid busy-waiting, missed scheduled work and re-entrant CEF calls.

## Native window ownership

- Slint owns the top-level HWND.
- CEF owns each browser child HWND and its renderer lifecycle.
- Rust owns the opaque CEF view handle and destroys it on the required UI thread.
- The CEF bridge must expose a C ABI; CEF C++ class layouts and CefRefPtr must never cross into Rust.
- Resize, visibility, focus, DPI, z-order, tab switching and close ordering need Windows UI tests.
- CEF browser callbacks must not retain pointers to Rust objects beyond their documented lifetime. Use explicit registration/unregistration and stable callback contexts.

## Acceptance gates

- [ ] Confirm CEF 156 bootstrap/client loading with the exact SDK.
- [ ] Add a Rust DLL entrypoint callable by the bootstrap.
- [ ] Verify child-process dispatch returns before Slint starts.
- [ ] Verify browser-process initialization uses the bootstrap-provided sandbox pointer.
- [ ] Integrate CEF external message pumping with Slint.
- [ ] Embed one CEF browser HWND in the Slint host and verify resize/focus/input/DPI.
- [ ] Verify multiple tabs, background visibility and teardown.
- [ ] Verify renderer crash and browser shutdown behavior.
- [ ] Run Windows/MSVC build and runtime CI before enabling the backend.
- [ ] Keep WebKit as the fallback until all acceptance gates pass.

## Upstream references

- [CEF Windows sandbox setup](https://chromiumembedded.github.io/cef/sandbox_setup)
- [CEF 156 RunWinMain API](https://cef-builds.spotifycdn.com/docs/156.0/cef__sandbox__win_8h.html)
- [CEF 156 CefExecuteProcess / CefInitialize](https://cef-builds.spotifycdn.com/docs/156.0/cef__app_8h.html)
- [CEF general usage and message-loop integration](https://chromiumembedded.github.io/cef/general_usage)
