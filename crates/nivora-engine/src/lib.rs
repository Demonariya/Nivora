//! Browser-engine facade. The only production engine in this workspace is WebKit WinCairo.

use std::ffi::{CStr, c_char, c_void};

use nivora_webkit_sys::WebKitView;
pub use nivora_webkit_sys::{
    KeyboardShortcutCallback, LOAD_PROGRESS_CHANGED, NAVIGATION_COMMITTED, NAVIGATION_FAILED,
    NAVIGATION_FINISHED, NAVIGATION_STARTED, PAGE_TITLE_CHANGED, install_keyboard_shortcuts,
    uninstall_keyboard_shortcuts,
};

pub type NavigationCallback = unsafe extern "C" fn(*mut c_void, i32, *const c_char, f64);

pub struct WebKitEngine {
    view: WebKitView,
}

impl WebKitEngine {
    /// # Safety
    /// The HWND must remain valid until this engine is dropped, and calls must
    /// stay on the same Win32 UI thread used to create the view.
    pub unsafe fn new(
        parent_hwnd: *mut c_void,
        bounds: (i32, i32, i32, i32),
        callback: NavigationCallback,
        callback_context: *mut c_void,
    ) -> Result<Self, &'static str> {
        // SAFETY: forwarded caller contract; the sys crate owns the native WKView.
        let view = unsafe { WebKitView::create(parent_hwnd, bounds, callback, callback_context)? };
        Ok(Self { view })
    }

    pub fn navigate(&mut self, address: &str) -> Result<(), String> {
        let address = normalize_address(address);
        self.view
            .navigate(&address)
            .map_err(|error| error.to_string())
    }

    pub fn back(&mut self) {
        self.view.go_back();
    }

    pub fn forward(&mut self) {
        self.view.go_forward();
    }

    pub fn reload(&mut self) {
        self.view.reload();
    }

    pub fn focus(&mut self) {
        self.view.focus();
    }

    pub fn resize(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.view.resize(x, y, width, height);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.view.set_visible(visible);
    }

    pub fn can_go_back(&self) -> bool {
        self.view.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        self.view.can_go_forward()
    }
}

pub fn normalize_address(input: &str) -> String {
    let value = input.trim();
    if value.is_empty() {
        return "about:blank".to_owned();
    }
    if value.starts_with("https://")
        || value.starts_with("http://")
        || value.starts_with("file://")
        || value.starts_with("about:")
    {
        return value.to_owned();
    }
    if value.contains(' ') || !value.contains('.') {
        return format!("https://duckduckgo.com/?q={}", encode_search_query(value));
    }
    format!("https://{value}")
}

fn encode_search_query(value: &str) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b' ' => encoded.push('+'),
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => write!(&mut encoded, "%{byte:02X}").expect("writing to String cannot fail"),
        }
    }
    encoded
}

pub fn callback_url(url: *const c_char) -> String {
    if url.is_null() {
        return String::new();
    }
    // SAFETY: the native callback guarantees a NUL-terminated URL valid for this call.
    unsafe { CStr::from_ptr(url) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::normalize_address;

    #[test]
    fn preserves_explicit_schemes() {
        assert_eq!(
            normalize_address("https://example.com"),
            "https://example.com"
        );
        assert_eq!(
            normalize_address("http://example.com"),
            "http://example.com"
        );
    }

    #[test]
    fn adds_https_to_hostnames() {
        assert_eq!(normalize_address("example.com"), "https://example.com");
    }

    #[test]
    fn turns_plain_text_into_search() {
        assert_eq!(
            normalize_address("rust browser"),
            "https://duckduckgo.com/?q=rust+browser"
        );
    }

    #[test]
    fn percent_encodes_search_query_delimiters_and_unicode() {
        assert_eq!(
            normalize_address("c++ & café"),
            "https://duckduckgo.com/?q=c%2B%2B+%26+caf%C3%A9"
        );
    }
}


/// Stable, engine-neutral description of the rendering backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub version: Option<String>,
}

/// Capabilities are explicit so the UI never infers support from the backend name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EngineCapabilities {
    pub hardware_acceleration: bool,
    pub downloads: bool,
    pub devtools: bool,
    pub multiple_profiles: bool,
    pub chrome_extensions: bool,
    pub manifest_v3: bool,
    pub web_rtc: bool,
}

/// Platform-independent operations exposed by a single browser view.
///
/// Implementations may have UI-thread affinity. They deliberately do not require
/// Send or Sync; callers must create and use a view on its owning UI thread.
pub trait EngineView {
    fn navigate(&mut self, address: &str) -> Result<(), String>;
    fn back(&mut self);
    fn forward(&mut self);
    fn reload(&mut self);
    fn focus(&mut self);
    fn resize(&mut self, x: i32, y: i32, width: i32, height: i32);
    fn set_visible(&mut self, visible: bool);
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;
}

/// Factory interface for a rendering backend.
///
/// Native handles are confined to this platform-facing boundary. The returned
/// view must be destroyed on the thread required by its rendering engine.
pub trait EngineBackend {
    fn info(&self) -> EngineInfo;
    fn capabilities(&self) -> EngineCapabilities;

    /// # Safety
    /// parent_hwnd must be a valid parent window on the current UI thread and
    /// remain valid until the returned view is dropped. The callback and context
    /// must remain valid for every native callback until view destruction.
    unsafe fn create_view(
        &self,
        parent_hwnd: *mut c_void,
        bounds: (i32, i32, i32, i32),
        callback: NavigationCallback,
        callback_context: *mut c_void,
    ) -> Result<Box<dyn EngineView>, String>;
}

/// Existing WinCairo implementation, retained as the current backend while the
/// CEF Chrome-runtime spike is evaluated.
#[derive(Debug, Default, Clone, Copy)]
pub struct WebKitBackend;

impl EngineBackend for WebKitBackend {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "webkit-wincairo",
            name: "WebKit WinCairo",
            version: None,
        }
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            // These remain false until verified against the exact deployed SDK.
            hardware_acceleration: false,
            downloads: false,
            devtools: false,
            multiple_profiles: false,
            chrome_extensions: false,
            manifest_v3: false,
            web_rtc: false,
        }
    }

    unsafe fn create_view(
        &self,
        parent_hwnd: *mut c_void,
        bounds: (i32, i32, i32, i32),
        callback: NavigationCallback,
        callback_context: *mut c_void,
    ) -> Result<Box<dyn EngineView>, String> {
        // SAFETY: forwards the caller's documented native-handle and callback
        // lifetime contract to the existing WebKit constructor.
        let view = unsafe {
            WebKitEngine::new(parent_hwnd, bounds, callback, callback_context)
        }
        .map_err(str::to_owned)?;
        Ok(Box::new(view))
    }
}

impl EngineView for WebKitEngine {
    fn navigate(&mut self, address: &str) -> Result<(), String> {
        WebKitEngine::navigate(self, address)
    }

    fn back(&mut self) {
        WebKitEngine::back(self);
    }

    fn forward(&mut self) {
        WebKitEngine::forward(self);
    }

    fn reload(&mut self) {
        WebKitEngine::reload(self);
    }

    fn focus(&mut self) {
        WebKitEngine::focus(self);
    }

    fn resize(&mut self, x: i32, y: i32, width: i32, height: i32) {
        WebKitEngine::resize(self, x, y, width, height);
    }

    fn set_visible(&mut self, visible: bool) {
        WebKitEngine::set_visible(self, visible);
    }

    fn can_go_back(&self) -> bool {
        WebKitEngine::can_go_back(self)
    }

    fn can_go_forward(&self) -> bool {
        WebKitEngine::can_go_forward(self)
    }
}
