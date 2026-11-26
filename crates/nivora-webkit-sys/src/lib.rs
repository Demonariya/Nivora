//! Narrow, ownership-safe Rust wrapper over the official WebKit WinCairo C API.
//!
//! The C++ bridge includes the upstream headers directly. No opaque WebKit
//! structure layout or API calling convention is re-declared in Rust.

use std::ffi::{c_char, c_void};
#[cfg(windows)]
use std::ptr::NonNull;

pub const NAVIGATION_STARTED: i32 = 1;
pub const NAVIGATION_COMMITTED: i32 = 2;
pub const NAVIGATION_FINISHED: i32 = 3;
pub const NAVIGATION_FAILED: i32 = 4;
pub const PAGE_TITLE_CHANGED: i32 = 5;
pub const LOAD_PROGRESS_CHANGED: i32 = 6;

pub type EventCallback = unsafe extern "C" fn(*mut c_void, i32, *const c_char, f64);
pub type KeyboardShortcutCallback = unsafe extern "C" fn(*mut c_void, i32);

#[cfg(windows)]
mod native {
    use super::{EventCallback, c_char, c_void};

    unsafe extern "C" {
        pub fn nivora_webkit_view_create(
            parent_window: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            callback: Option<EventCallback>,
            callback_context: *mut c_void,
        ) -> *mut c_void;
        pub fn nivora_webkit_view_destroy(view: *mut c_void);
        pub fn nivora_webkit_view_resize(
            view: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        );
        pub fn nivora_webkit_view_set_visible(view: *mut c_void, visible: bool);
        pub fn nivora_webkit_view_navigate(view: *mut c_void, address: *const c_char);
        pub fn nivora_webkit_view_go_back(view: *mut c_void);
        pub fn nivora_webkit_view_go_forward(view: *mut c_void);
        pub fn nivora_webkit_view_reload(view: *mut c_void);
        pub fn nivora_webkit_view_can_go_back(view: *mut c_void) -> bool;
        pub fn nivora_webkit_view_can_go_forward(view: *mut c_void) -> bool;
        pub fn nivora_webkit_view_focus(view: *mut c_void);
        pub fn nivora_install_keyboard_shortcuts(
            host_window: *mut c_void,
            callback: Option<KeyboardShortcutCallback>,
            context: *mut c_void,
        ) -> bool;
        pub fn nivora_uninstall_keyboard_shortcuts();
    }
}

/// Owns one WinCairo `WKViewRef` through the native compatibility bridge.
pub struct WebKitView {
    #[cfg(windows)]
    raw: NonNull<c_void>,
}

impl WebKitView {
    /// # Safety
    /// `parent_window` must be a live Win32 HWND on the WebKit UI thread.
    #[cfg(windows)]
    pub unsafe fn create(
        parent_window: *mut c_void,
        bounds: (i32, i32, i32, i32),
        callback: EventCallback,
        callback_context: *mut c_void,
    ) -> Result<Self, &'static str> {
        let (x, y, width, height) = bounds;
        // SAFETY: the caller supplies a live HWND; the C++ bridge validates it
        // and constructs/destroys the actual upstream WKView object.
        let raw = unsafe {
            native::nivora_webkit_view_create(
                parent_window,
                x,
                y,
                width,
                height,
                Some(callback),
                callback_context,
            )
        };
        NonNull::new(raw)
            .map(|raw| Self { raw })
            .ok_or("WKViewCreate failed")
    }

    #[cfg(not(windows))]
    pub unsafe fn create(
        _parent_window: *mut c_void,
        _bounds: (i32, i32, i32, i32),
        _callback: EventCallback,
        _callback_context: *mut c_void,
    ) -> Result<Self, &'static str> {
        Err("the official WebKit WinCairo view is available only on Windows")
    }

    #[cfg(windows)]
    pub fn resize(&mut self, x: i32, y: i32, width: i32, height: i32) {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_resize(self.raw.as_ptr(), x, y, width, height) };
    }

    #[cfg(not(windows))]
    pub fn resize(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {}

    #[cfg(windows)]
    pub fn set_visible(&mut self, visible: bool) {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_set_visible(self.raw.as_ptr(), visible) };
    }

    #[cfg(not(windows))]
    pub fn set_visible(&mut self, _visible: bool) {}

    pub fn navigate(&mut self, address: &str) -> Result<(), std::ffi::NulError> {
        let address = std::ffi::CString::new(address)?;
        #[cfg(windows)]
        {
            // SAFETY: raw is owned by self; CString is NUL-terminated for call duration.
            unsafe { native::nivora_webkit_view_navigate(self.raw.as_ptr(), address.as_ptr()) };
        }
        #[cfg(not(windows))]
        let _ = address;
        Ok(())
    }

    #[cfg(windows)]
    pub fn go_back(&mut self) {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_go_back(self.raw.as_ptr()) };
    }

    #[cfg(not(windows))]
    pub fn go_back(&mut self) {}

    #[cfg(windows)]
    pub fn go_forward(&mut self) {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_go_forward(self.raw.as_ptr()) };
    }

    #[cfg(not(windows))]
    pub fn go_forward(&mut self) {}

    #[cfg(windows)]
    pub fn reload(&mut self) {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_reload(self.raw.as_ptr()) };
    }

    #[cfg(not(windows))]
    pub fn reload(&mut self) {}

    #[cfg(windows)]
    pub fn can_go_back(&self) -> bool {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_can_go_back(self.raw.as_ptr()) }
    }

    #[cfg(not(windows))]
    pub fn can_go_back(&self) -> bool {
        false
    }

    #[cfg(windows)]
    pub fn can_go_forward(&self) -> bool {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_can_go_forward(self.raw.as_ptr()) }
    }

    #[cfg(not(windows))]
    pub fn can_go_forward(&self) -> bool {
        false
    }

    #[cfg(windows)]
    pub fn focus(&mut self) {
        // SAFETY: raw is owned by self and valid until Drop.
        unsafe { native::nivora_webkit_view_focus(self.raw.as_ptr()) };
    }

    #[cfg(not(windows))]
    pub fn focus(&mut self) {}
}

impl Drop for WebKitView {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            // SAFETY: this is the unique owner; native destroy closes WKPage and releases WKView.
            unsafe { native::nivora_webkit_view_destroy(self.raw.as_ptr()) };
        }
    }
}


/// Installs a thread-scoped Win32 message hook for browser keyboard shortcuts.
///
/// # Safety
/// The host HWND and callback context must remain valid until uninstall is called
/// on the same UI thread.
#[cfg(windows)]
pub unsafe fn install_keyboard_shortcuts(
    host_window: *mut c_void,
    callback: KeyboardShortcutCallback,
    context: *mut c_void,
) -> bool {
    // SAFETY: forwarded caller contract; hook is scoped to the HWND's UI thread.
    unsafe { native::nivora_install_keyboard_shortcuts(host_window, Some(callback), context) }
}

#[cfg(not(windows))]
pub unsafe fn install_keyboard_shortcuts(
    _host_window: *mut c_void,
    _callback: KeyboardShortcutCallback,
    _context: *mut c_void,
) -> bool { false }

#[cfg(windows)]
pub unsafe fn uninstall_keyboard_shortcuts() {
    // SAFETY: the hook is owned by this process and installed on the current UI thread.
    unsafe { native::nivora_uninstall_keyboard_shortcuts() };
}

#[cfg(not(windows))]
pub unsafe fn uninstall_keyboard_shortcuts() {}
