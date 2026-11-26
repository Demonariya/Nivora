#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>

#include <WebKit/WKBase.h>
#include <WebKit/WKPage.h>
#include <WebKit/WKPageConfigurationRef.h>
#include <WebKit/WKFramePolicyListener.h>
#include <WebKit/WKPageNavigationClient.h>
#include <WebKit/WKPageStateClient.h>
#include <WebKit/WKString.h>
#include <WebKit/WKType.h>
#include <WebKit/WKURL.h>
#include <WebKit/WKView.h>

#include <new>
#include <string>
#include <vector>

using NivoraEventCallback = void(__cdecl*)(void* context, int event_kind, const char* value, double progress);
using NivoraShortcutCallback = void(__cdecl*)(void* context, int action);

static HHOOK g_keyboard_hook { nullptr };
static NivoraShortcutCallback g_shortcut_callback { nullptr };
static void* g_shortcut_context { nullptr };

static LRESULT CALLBACK keyboard_message_hook(int code, WPARAM wparam, LPARAM lparam)
{
    if (code == HC_ACTION && wparam == PM_REMOVE && g_shortcut_callback) {
        auto* message = reinterpret_cast<MSG*>(lparam);
        if (message && (message->message == WM_KEYDOWN || message->message == WM_SYSKEYDOWN)
            && (message->lParam & (1LL << 30)) == 0
            && (GetKeyState(VK_CONTROL) & 0x8000)) {
            int action = 0;
            switch (message->wParam) {
            case 'T': action = 1; break;
            case 'W': action = 2; break;
            case VK_TAB: action = (GetKeyState(VK_SHIFT) & 0x8000) ? 4 : 3; break;
            default: break;
            }
            if (action) {
                g_shortcut_callback(g_shortcut_context, action);
                // Consume the shortcut before it reaches WebKit or a page.
                message->message = WM_NULL;
                message->wParam = 0;
                message->lParam = 0;
            }
        }
    }
    return CallNextHookEx(g_keyboard_hook, code, wparam, lparam);
}

enum : int {
    NIVORA_NAVIGATION_STARTED = 1,
    NIVORA_NAVIGATION_COMMITTED = 2,
    NIVORA_NAVIGATION_FINISHED = 3,
    NIVORA_NAVIGATION_FAILED = 4,
    NIVORA_PAGE_TITLE_CHANGED = 5,
    NIVORA_LOAD_PROGRESS_CHANGED = 6,
};

struct NivoraWebKitView {
    WKViewRef view { nullptr };
    WKPageRef page { nullptr };
    HWND child_window { nullptr };
    NivoraEventCallback callback { nullptr };
    void* callback_context { nullptr };
    WKPageNavigationClientV0 navigation_client { };
    WKPageStateClientV0 state_client { };
};

static std::string copy_page_url(WKPageRef page)
{
    WKURLRef url = WKPageCopyActiveURL(page);
    if (!url)
        url = WKPageCopyProvisionalURL(page);
    if (!url)
        return { };

    WKStringRef string = WKURLCopyString(url);
    WKRelease(url);
    if (!string)
        return { };

    const size_t capacity = WKStringGetMaximumUTF8CStringSize(string);
    std::vector<char> buffer(capacity ? capacity : 1, '\0');
    WKStringGetUTF8CString(string, buffer.data(), buffer.size());
    WKRelease(string);
    return std::string(buffer.data());
}

static void emit_navigation_event(WKPageRef page, int event_kind, const void* client_info)
{
    auto* self = const_cast<NivoraWebKitView*>(static_cast<const NivoraWebKitView*>(client_info));
    if (!self || !self->callback)
        return;
    const std::string url = copy_page_url(page);
    self->callback(self->callback_context, event_kind, url.c_str(), 0.0);
}

static std::string copy_wk_string(WKStringRef string)
{
    if (!string)
        return { };
    const size_t capacity = WKStringGetMaximumUTF8CStringSize(string);
    std::vector<char> buffer(capacity ? capacity : 1, '\0');
    WKStringGetUTF8CString(string, buffer.data(), buffer.size());
    return std::string(buffer.data());
}

static void did_change_title(const void* client_info)
{
    auto* self = const_cast<NivoraWebKitView*>(static_cast<const NivoraWebKitView*>(client_info));
    if (!self || !self->page || !self->callback)
        return;
    WKStringRef title = WKPageCopyTitle(self->page);
    const std::string value = copy_wk_string(title);
    if (title)
        WKRelease(title);
    self->callback(self->callback_context, NIVORA_PAGE_TITLE_CHANGED, value.c_str(), 0.0);
}

static void did_change_estimated_progress(const void* client_info)
{
    auto* self = const_cast<NivoraWebKitView*>(static_cast<const NivoraWebKitView*>(client_info));
    if (!self || !self->page || !self->callback)
        return;
    self->callback(
        self->callback_context,
        NIVORA_LOAD_PROGRESS_CHANGED,
        "",
        WKPageGetEstimatedProgress(self->page));
}

static void decide_policy_for_action(
    WKPageRef,
    WKNavigationActionRef,
    WKFramePolicyListenerRef listener,
    WKTypeRef,
    const void*)
{
    WKFramePolicyListenerUse(listener);
}

static void decide_policy_for_response(
    WKPageRef,
    WKNavigationResponseRef,
    WKFramePolicyListenerRef listener,
    WKTypeRef,
    const void*)
{
    WKFramePolicyListenerUse(listener);
}

static void did_start_provisional(WKPageRef page, WKNavigationRef, WKTypeRef, const void* client_info)
{
    emit_navigation_event(page, NIVORA_NAVIGATION_STARTED, client_info);
}

static void did_commit(WKPageRef page, WKNavigationRef, WKTypeRef, const void* client_info)
{
    emit_navigation_event(page, NIVORA_NAVIGATION_COMMITTED, client_info);
}

static void did_finish(WKPageRef page, WKNavigationRef, WKTypeRef, const void* client_info)
{
    emit_navigation_event(page, NIVORA_NAVIGATION_FINISHED, client_info);
}

static void did_fail_provisional(WKPageRef page, WKNavigationRef, WKErrorRef, WKTypeRef, const void* client_info)
{
    emit_navigation_event(page, NIVORA_NAVIGATION_FAILED, client_info);
}

static void did_fail(WKPageRef page, WKNavigationRef, WKErrorRef, WKTypeRef, const void* client_info)
{
    emit_navigation_event(page, NIVORA_NAVIGATION_FAILED, client_info);
}

extern "C" NivoraWebKitView* nivora_webkit_view_create(
    HWND parent_window,
    int x,
    int y,
    int width,
    int height,
    NivoraEventCallback callback,
    void* callback_context)
{
    if (!parent_window || width <= 0 || height <= 0)
        return nullptr;

    auto* self = new (std::nothrow) NivoraWebKitView;
    if (!self)
        return nullptr;

    self->callback = callback;
    self->callback_context = callback_context;

    WKPageConfigurationRef configuration = WKPageConfigurationCreate();
    if (!configuration) {
        delete self;
        return nullptr;
    }

    RECT rect { x, y, x + width, y + height };
    self->view = WKViewCreate(rect, configuration, parent_window);
    WKRelease(configuration);
    if (!self->view) {
        delete self;
        return nullptr;
    }

    self->page = WKViewGetPage(self->view);
    self->child_window = WKViewGetWindow(self->view);
    if (!self->page || !self->child_window) {
        if (self->page)
            WKPageClose(self->page);
        WKRelease(self->view);
        delete self;
        return nullptr;
    }

    self->navigation_client = { };
    self->navigation_client.base.version = 0;
    self->navigation_client.base.clientInfo = self;
    self->navigation_client.decidePolicyForNavigationAction = decide_policy_for_action;
    self->navigation_client.decidePolicyForNavigationResponse = decide_policy_for_response;
    self->navigation_client.didStartProvisionalNavigation = did_start_provisional;
    self->navigation_client.didCommitNavigation = did_commit;
    self->navigation_client.didFinishNavigation = did_finish;
    self->navigation_client.didFailProvisionalNavigation = did_fail_provisional;
    self->navigation_client.didFailNavigation = did_fail;
    WKPageSetPageNavigationClient(self->page, &self->navigation_client.base);

    self->state_client = { };
    self->state_client.base.version = 0;
    self->state_client.base.clientInfo = self;
    self->state_client.didChangeTitle = did_change_title;
    self->state_client.didChangeEstimatedProgress = did_change_estimated_progress;
    WKPageSetPageStateClient(self->page, &self->state_client.base);

    // WKViewCreate already receives the parent HWND. Set it explicitly as well
    // so reparenting and ancestry notifications follow the public WinCairo API.
    WKViewSetParentWindow(self->view, parent_window);
    WKViewWindowAncestryDidChange(self->view);
    WKViewSetIsInWindow(self->view, true);
    if (!SetWindowPos(self->child_window, nullptr, x, y, width, height,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW)) {
        WKViewSetIsInWindow(self->view, false);
        WKPageSetPageStateClient(self->page, nullptr);
        WKPageClose(self->page);
        WKRelease(self->view);
        delete self;
        return nullptr;
    }
    return self;
}

extern "C" void nivora_webkit_view_destroy(NivoraWebKitView* self)
{
    if (!self)
        return;
    if (self->view)
        WKViewSetIsInWindow(self->view, false);
    if (self->page) {
        WKPageSetPageStateClient(self->page, nullptr);
        WKPageClose(self->page);
    }
    if (self->view)
        WKRelease(self->view);
    delete self;
}

extern "C" void nivora_webkit_view_resize(NivoraWebKitView* self, int x, int y, int width, int height)
{
    if (!self || !self->view || !self->child_window || width <= 0 || height <= 0)
        return;
    SetWindowPos(self->child_window, nullptr, x, y, width, height,
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER);
}

extern "C" void nivora_webkit_view_set_visible(NivoraWebKitView* self, bool visible)
{
    if (!self || !self->view || !self->child_window || !IsWindow(self->child_window))
        return;
    ShowWindow(self->child_window, visible ? SW_SHOWNA : SW_HIDE);
}

extern "C" void nivora_webkit_view_navigate(NivoraWebKitView* self, const char* address)
{
    if (!self || !self->page || !address || !*address)
        return;
    WKURLRef url = WKURLCreateWithUTF8CString(address);
    if (!url)
        return;
    WKPageLoadURL(self->page, url);
    WKRelease(url);
}

extern "C" void nivora_webkit_view_go_back(NivoraWebKitView* self)
{
    if (self && self->page && WKPageCanGoBack(self->page))
        WKPageGoBack(self->page);
}

extern "C" void nivora_webkit_view_go_forward(NivoraWebKitView* self)
{
    if (self && self->page && WKPageCanGoForward(self->page))
        WKPageGoForward(self->page);
}

extern "C" void nivora_webkit_view_reload(NivoraWebKitView* self)
{
    if (self && self->page)
        WKPageReload(self->page);
}

extern "C" bool nivora_webkit_view_can_go_back(NivoraWebKitView* self)
{
    return self && self->page && WKPageCanGoBack(self->page);
}

extern "C" bool nivora_webkit_view_can_go_forward(NivoraWebKitView* self)
{
    return self && self->page && WKPageCanGoForward(self->page);
}

extern "C" void nivora_webkit_view_focus(NivoraWebKitView* self)
{
    if (self && self->child_window && IsWindow(self->child_window))
        SetFocus(self->child_window);
}


extern "C" bool nivora_install_keyboard_shortcuts(HWND host_window, NivoraShortcutCallback callback, void* context)
{
    if (!host_window || !callback)
        return false;
    const DWORD thread_id = GetWindowThreadProcessId(host_window, nullptr);
    if (!thread_id)
        return false;
    if (g_keyboard_hook)
        UnhookWindowsHookEx(g_keyboard_hook);
    g_shortcut_callback = callback;
    g_shortcut_context = context;
    g_keyboard_hook = SetWindowsHookExW(WH_GETMESSAGE, keyboard_message_hook, nullptr, thread_id);
    if (!g_keyboard_hook) {
        g_shortcut_callback = nullptr;
        g_shortcut_context = nullptr;
        return false;
    }
    return true;
}

extern "C" void nivora_uninstall_keyboard_shortcuts()
{
    // Clear callback state before unhooking so no queued message can call a
    // context that Rust has already released.
    g_shortcut_callback = nullptr;
    g_shortcut_context = nullptr;
    if (g_keyboard_hook) {
        UnhookWindowsHookEx(g_keyboard_hook);
        g_keyboard_hook = nullptr;
    }
}
