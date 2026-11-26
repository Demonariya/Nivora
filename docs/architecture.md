# Architecture

## Process and ownership

nivora-app starts nivora-ui. Slint's Winit backend owns the top-level Windows HWND. After the window is shown and its raw-window-handle-06 handle is available, the UI extracts the Win32 HWND and creates a WebKit view as a native child window.

The ownership chain is: Rust UI -> nivora-engine -> nivora-webkit-sys -> C++ ABI bridge -> WebKit2 WinCairo WKView/WKPage.

The C++ bridge owns the WebKit references and navigation client structure. Rust receives one opaque bridge pointer and never recreates WebKit structure layouts.

## Native creation and navigation

The bridge calls WKPageConfigurationCreate, WKViewCreate, WKViewGetPage, and WKViewGetWindow. It registers a WKPageNavigationClientV0 using WKPageSetPageNavigationClient, then calls WKViewSetParentWindow, WKViewWindowAncestryDidChange, and WKViewSetIsInWindow. The returned child HWND is positioned with SetWindowPos.

Navigation uses WKURLCreateWithUTF8CString and WKPageLoadURL. Back/Forward and state use WKPageGoBack, WKPageGoForward, WKPageCanGoBack, and WKPageCanGoForward. Reload uses WKPageReload. Temporary WK references are released with WKRelease. Shutdown calls WKPageClose and releases WKView before freeing the bridge.

## Slint HWND integration

Slint exposes Window::window_handle() behind raw-window-handle-06. Its documentation warns that the handle may not be available until after the native window is created, so Nivora retries from a Slint event-loop timer after show().

Slint geometry is logical pixels. Nivora multiplies the browser host rectangle by Window::scale_factor() and uses physical pixels in SetWindowPos. A timer synchronizes bounds and Back/Forward state. The child HWND receives native Windows input; the address bar and toolbar remain Slint controls.

This API-level integration does not prove renderer composition or z-order behavior. Windows acceptance tests must verify clipping, z-order, focus, keyboard/mouse input, DPI changes, minimize/restore, and teardown. If a renderer cannot host a child HWND correctly, the appropriate fallback is a custom Slint Platform/WindowAdapter using a Win32 top-level window, not a different browser engine.

## Threading and lifetime

All bridge calls are intended for the same Win32 UI thread that owns the Slint window. The view is not Send or Sync. Navigation callbacks receive a temporary UTF-8 URL buffer; Rust copies it immediately. The close-request callback releases the WebKit view before Slint hides the parent window.

## Persistence and tabs

nivora-storage provides a SQLite history schema and query methods. nivora-tabs provides the in-memory tab model. The first native milestone hosts one WebKit view; multi-view tab lifecycle and persistence wiring remain future work.
