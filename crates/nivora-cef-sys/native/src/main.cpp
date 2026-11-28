// Minimal standalone CEF smoke host for Nivora.
// This executable uses the same image for browser and renderer processes,
// and keeps the Windows CEF sandbox enabled.

#include <windows.h>

#include "include/cef_app.h"
#include "include/cef_browser.h"
#include "include/cef_client.h"
#include "include/cef_command_line.h"
#include "include/cef_display_handler.h"
#include "include/cef_life_span_handler.h"
#include "include/cef_sandbox_win.h"

namespace {

class NivoraBrowserClient final : public CefClient,
                                  public CefLifeSpanHandler,
                                  public CefDisplayHandler {
 public:
  CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override { return this; }
  CefRefPtr<CefDisplayHandler> GetDisplayHandler() override { return this; }

  void OnAfterCreated(CefRefPtr<CefBrowser> browser) override {
    CEF_REQUIRE_UI_THREAD();
    browser_ = browser;
  }

  void OnBeforeClose(CefRefPtr<CefBrowser> browser) override {
    CEF_REQUIRE_UI_THREAD();
    if (browser_ && browser_->IsSame(browser)) {
      browser_ = nullptr;
    }
    CefQuitMessageLoop();
  }

  void OnTitleChange(CefRefPtr<CefBrowser> browser,
                     const CefString& title) override {
    CEF_REQUIRE_UI_THREAD();
    const auto window = browser->GetHost()->GetWindowHandle();
    if (window) {
      const auto wide_title = title.ToWString();
      SetWindowTextW(window, wide_title.c_str());
    }
  }

 private:
  CefRefPtr<CefBrowser> browser_;

  IMPLEMENT_REFCOUNTING(NivoraBrowserClient);
  DISALLOW_COPY_AND_ASSIGN(NivoraBrowserClient);
};

class NivoraApp final : public CefApp, public CefBrowserProcessHandler {
 public:
  CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override {
    return this;
  }

  void OnContextInitialized() override {
    CEF_REQUIRE_UI_THREAD();

    auto command_line = CefCommandLine::GetGlobalCommandLine();
    CefString initial_url = command_line->GetSwitchValue("url");
    if (initial_url.empty()) {
      initial_url = "https://example.com";
    }

    CefWindowInfo window_info;
    window_info.SetAsPopup(nullptr, "Nivora CEF Native Host");

    CefBrowserSettings browser_settings;
    browser_settings.default_encoding = "UTF-8";

    client_ = new NivoraBrowserClient();
    if (!CefBrowserHost::CreateBrowser(
            window_info, client_, initial_url, browser_settings, nullptr,
            nullptr)) {
      OutputDebugStringW(
          L"Nivora CEF host: CefBrowserHost::CreateBrowser failed.\n");
      CefQuitMessageLoop();
    }
  }

 private:
  CefRefPtr<NivoraBrowserClient> client_;

  IMPLEMENT_REFCOUNTING(NivoraApp);
  DISALLOW_COPY_AND_ASSIGN(NivoraApp);
};

int RunCef(HINSTANCE instance) {
  CefMainArgs main_args(instance);

  // Sandbox information is created inside this executable, as required by CEF.
  CefScopedSandboxInfo sandbox;
  void* sandbox_info = sandbox.sandbox_info();

  // This same executable is relaunched for renderer, GPU and utility processes.
  const int process_exit_code =
      CefExecuteProcess(main_args, nullptr, sandbox_info);
  if (process_exit_code >= 0) {
    return process_exit_code;
  }

  CefSettings settings;
  settings.no_sandbox = false;
  settings.multi_threaded_message_loop = false;
  settings.windowless_rendering_enabled = false;

  CefRefPtr<NivoraApp> app = new NivoraApp();
  if (!CefInitialize(main_args, settings, app, sandbox_info)) {
    return CefGetExitCode();
  }

  CefRunMessageLoop();
  CefShutdown();
  return 0;
}

}  // namespace

int APIENTRY wWinMain(HINSTANCE instance, HINSTANCE, LPWSTR, int) {
  return RunCef(instance);
}
