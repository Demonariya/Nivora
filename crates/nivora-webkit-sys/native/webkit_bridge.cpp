#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>

#include <WebKit/WKBase.h>
#include <WebKit/WKPage.h>
#include <WebKit/WKPageConfigurationRef.h>
#include <WebKit/WKFramePolicyListener.h>
#include <WebKit/WKPageNavigationClient.h>
#include <WebKit/WKString.h>
#include <WebKit/WKType.h>
#include <WebKit/WKURL.h>
#include <WebKit/WKView.h>

#include <new>
#include <string>
#include <vector>

using NivoraEventCallback = void(__cdecl*)(void* context, int event_kind, const char* url);

enum : int {
    NIVORA_NAVIGATION_STARTED = 1,
    NIVORA_NAVIGATION_COMMITTED = 2,
    NIVORA_NAVIGATION_FINISHED = 3,
    NIVORA_NAVIGATION_FAILED = 4,
};

struct NivoraWebKitView {
    WKViewRef view { nullptr };
    WKPageRef page { nullptr };
    HWND child_window { nullptr };
    NivoraEventCallback callback { nullptr };
    void* callback_context { nullptr };
    WKPageNavigationClientV0 navigation_client { };
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
    self->callback(self->callback_context, event_kind, url.c_str());
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

    // WKViewCreate already receives the parent HWND. Set it explicitly as well
    // so reparenting and ancestry notifications follow the public WinCairo API.
    WKViewSetParentWindow(self->view, parent_window);
    WKViewWindowAncestryDidChange(self->view);
    WKViewSetIsInWindow(self->view, true);
    if (!SetWindowPos(self->child_window, nullptr, x, y, width, height,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW)) {
        WKViewSetIsInWindow(self->view, false);
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
    if (self->page)
        WKPageClose(self->page);
    if (self->view)
        WKRelease(self->view);
    delete self;
}

extern "C" void nivora_webkit_view_resize(NivoraWebKitView* self, int x, int y, int width, int height)
{
    if (!self || !self->view || !self->child_window || width <= 0 || height <= 0)
        return;
    SetWindowPos(self->child_window, nullptr, x, y, width, height,
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW);
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
