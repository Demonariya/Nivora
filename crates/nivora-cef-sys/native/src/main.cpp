// CEF bootstrap client DLL. The official bootstrap owns the process image
// and sandbox setup; this DLL dispatches CEF child processes before entering Rust.

#include <windows.h>

#include "include/cef_app.h"
#include "include/cef_sandbox_win.h"
#include "include/cef_version_info.h"

extern "C" __declspec(dllimport) int nivora_rust_run_ui();

namespace {

int RunNivora(HINSTANCE instance, void* sandbox_info) {
  CefMainArgs main_args(instance);

  // Dispatch renderer/GPU/utility processes before initializing the browser
  // process. Child processes must never construct the Rust/Slint application.
  const int process_exit_code =
      CefExecuteProcess(main_args, nullptr, sandbox_info);
  if (process_exit_code >= 0) {
    return process_exit_code;
  }

  CefSettings settings;

  // Slint owns the application's main/UI thread. On Windows CEF supports a
  // dedicated browser-process UI thread, so it can run without blocking the
  // Slint event loop or requiring CefDoMessageLoopWork() timers.
  settings.multi_threaded_message_loop = true;

  // Keep the bootstrap-provided sandbox information. Do not set no_sandbox.
  if (!CefInitialize(main_args, settings, nullptr, sandbox_info)) {
    // CEF documents that no further CEF APIs may be called after a failed
    // initialization (except the optional CefGetExitCode()).
    OutputDebugStringW(
        L"Nivora: CefInitialize failed; aborting browser-process startup.\n");
    return 3;
  }

  // CefInitialize and CefShutdown both execute on the bootstrap/browser
  // process main thread. CEF's own UI/message loop runs on its dedicated
  // thread while Rust runs Slint on this thread.
  const int app_exit_code = nivora_rust_run_ui();

  // The Slint UI has returned and this milestone creates no CEF browsers yet,
  // so shutdown has no outstanding browser HWNDs to destroy.
  CefShutdown();
  return app_exit_code;
}

}  // namespace

CEF_BOOTSTRAP_EXPORT int RunWinMain(HINSTANCE instance,
                                    LPWSTR,
                                    int,
                                    void* sandbox_info,
                                    cef_version_info_t*) {
  return RunNivora(instance, sandbox_info);
}
