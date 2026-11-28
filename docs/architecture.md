# Architecture

## Process and ownership

nivora-app starts nivora-ui. Slint's Winit backend owns the top-level Windows HWND. After the window is shown and its raw-window-handle-06 handle is available, the UI extracts the Win32 HWND and creates a WebKit view as a native child window.

The ownership chain is: Rust UI -> nivora-engine -> nivora-webkit-sys -> C++ ABI bridge -> WebKit2 WinCairo WKView/WKPage.

The C++ bridge owns the WebKit references and navigation client structure. Rust receives one opaque bridge pointer and never recreates WebKit structure layouts.

## Native creation and navigation

The bridge calls WKPageConfigurationCreate, WKViewCreate, WKViewGetPage, and WKViewGetWindow. It registers a WKPageNavigationClientV0 using WKPageSetPageNavigationClient, then calls WKViewSetParentWindow, WKViewWindowAncestryDidChange, and WKViewSetIsInWindow. The returned child HWND is positioned with SetWindowPos.

Navigation uses WKURLCreateWithUTF8CString and WKPageLoadURL. Back/Forward and state use WKPageGoBack, WKPageGoForward, WKPageCanGoBack, and WKPageCanGoForward. Reload uses WKPageReload. Temporary WK references are released with WKRelease. On teardown the bridge marks the view out-of-window, closes the page, releases the view, and frees its bridge object.

## Slint HWND integration

Slint exposes Window::window_handle() behind raw-window-handle-06. Its documentation notes that the handle may not be available until after the native window is created, so Nivora starts its timer before ComponentHandle::run() and retries after the window has been shown.

The browser anchor exposes its bounds as Slint length properties. Slint maps those logical lengths to physical-pixel f32 values in Rust, applying the display scale factor automatically. Nivora passes those values directly to SetWindowPos and does not multiply by scale_factor a second time. A timer synchronizes bounds and Back/Forward state.

The child HWND receives native Windows input; the address bar and toolbar remain Slint controls. This API-level integration does not prove renderer composition or z-order behavior. Windows acceptance tests must verify clipping, z-order, focus, keyboard/mouse input, DPI changes, minimize/restore, and teardown. If a renderer cannot host a child HWND correctly, the appropriate fallback is a custom Slint Platform/WindowAdapter using a Win32 top-level window, not a different browser engine.

## Threading and lifetime

All bridge calls are intended for the same Win32 UI thread that owns the Slint window. The view is not Send or Sync. Navigation callbacks receive a temporary UTF-8 URL buffer; Rust copies it immediately. The close-request callback releases the WebKit view before Slint hides the parent window.

## Persistence and tabs

nivora-storage provides a SQLite history schema and query methods. nivora-tabs provides the in-memory tab model. The first native milestone hosts one WebKit view; multi-view tab lifecycle and persistence wiring remain future work.

## Rendering backend boundary

The nivora-engine crate now defines an engine-neutral contract:

- EngineBackend reports stable engine identity and verified capabilities, then creates a browser view.
- EngineView exposes navigation, history traversal, reload, focus, resize, visibility and back/forward availability.
- EngineInfo identifies the backend without leaking engine-specific native types.
- EngineCapabilities lets the UI gate features based on verified runtime support instead of assumptions.

WebKitBackend adapts the existing WebKitEngine to this contract. The original WebKitEngine API remains available so the current app is not forced into a risky all-at-once migration. The current capability report is deliberately conservative: features not yet verified against the deployed WinCairo SDK are reported as unavailable.

The traits do not require Send or Sync because the current native view is UI-thread-affine. Native HWNDs and callback pointers cross only the backend creation boundary. The caller remains responsible for callback/context lifetime and for destroying the view on the engine's required thread.

A future CEF implementation belongs in a separate native sys crate and must implement EngineBackend/EngineView. CEF-specific extension interfaces must be exposed through a separate capability-gated extension subsystem; the generic EngineView must not grow fake Chrome API methods. The app should migrate from the concrete WebKitEngine to a selected EngineBackend only after the CEF spike has passed Windows runtime and extension acceptance tests.
