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

  // CEF child processes are handled here. The browser-process path returns -1
  // and continues into the Rust/Slint application.
  const int process_exit_code =
      CefExecuteProcess(main_args, nullptr, sandbox_info);
  if (process_exit_code >= 0) {
    return process_exit_code;
  }

  // This integration milestone deliberately starts the Slint shell without
  // initializing a CEF browser yet. Browser initialization, message-pump
  // integration and HWND embedding are the next acceptance gates.
  return nivora_rust_run_ui();
}

}  // namespace

CEF_BOOTSTRAP_EXPORT int RunWinMain(HINSTANCE instance,
                                    LPWSTR,
                                    int,
                                    void* sandbox_info,
                                    cef_version_info_t*) {
  return RunNivora(instance, sandbox_info);
}
