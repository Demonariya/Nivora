//! Slint shell and multi-view Win32 host for WebKit WinCairo.

slint::include_modules!();

use slint::ComponentHandle;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let ui = MainWindow::new()?;

    #[cfg(windows)]
    run_with_webkit(ui)?;

    #[cfg(not(windows))]
    {
        ui.set_page_status(
            "WinCairo WebKit is Windows-only. Run Nivora on Windows with WEBKIT_INCLUDE_DIR and WEBKIT_LIB_DIR configured.".into(),
        );
        ui.run()?;
    }

    Ok(())
}

#[cfg(windows)]
fn run_with_webkit(ui: MainWindow) -> Result<(), Box<dyn Error>> {
    use nivora_engine::{
        LOAD_PROGRESS_CHANGED, NAVIGATION_COMMITTED, NAVIGATION_FAILED, NAVIGATION_FINISHED,
        NAVIGATION_STARTED, PAGE_TITLE_CHANGED, WebKitEngine, callback_url, normalize_address,
    };
    use raw_window_handle_06::{HasWindowHandle, RawWindowHandle};
    use slint::{ModelRc, Timer, TimerMode, VecModel};
    use std::{
        cell::{Cell, RefCell},
        ffi::c_void,
        rc::Rc,
        time::Duration,
    };

    struct EventSink {
        ui: slint::Weak<MainWindow>,
        tab_id: u64,
        active_tab_id: Rc<Cell<u64>>,
        tab_url: Rc<RefCell<String>>,
        tab_title: Rc<RefCell<String>>,
        title_dirty: Rc<Cell<bool>>,
        is_loading: Rc<Cell<bool>>,
        progress: Rc<Cell<f64>>,
    }

    // Keep the engine before its callback context in TabState. Rust drops
    // struct fields in declaration order, so WKPage/WKView is destroyed first.
    struct TabState {
        id: u64,
        title: Rc<RefCell<String>>,
        title_dirty: Rc<Cell<bool>>,
        url: Rc<RefCell<String>>,
        is_loading: Rc<Cell<bool>>,
        progress: Rc<Cell<f64>>,
        engine: Option<WebKitEngine>,
        event_sink: Option<Box<EventSink>>,
    }

    struct BrowserState {
        tabs: Vec<TabState>,
        active: usize,
        next_id: u64,
    }

    impl BrowserState {
        fn new() -> Self {
            Self {
                tabs: vec![TabState {
                    id: 1,
                    title: Rc::new(RefCell::new("New Tab".into())),
                    title_dirty: Rc::new(Cell::new(false)),
                    url: Rc::new(RefCell::new("https://example.com".into())),
                    is_loading: Rc::new(Cell::new(false)),
                    progress: Rc::new(Cell::new(0.0)),
                    engine: None,
                    event_sink: None,
                }],
                active: 0,
                next_id: 1,
            }
        }

        fn active_tab(&self) -> &TabState {
            &self.tabs[self.active]
        }

        fn active_tab_mut(&mut self) -> &mut TabState {
            &mut self.tabs[self.active]
        }

        fn open_tab(&mut self) {
            self.next_id += 1;
            self.tabs.push(TabState {
                id: self.next_id,
                title: Rc::new(RefCell::new("New Tab".into())),
                title_dirty: Rc::new(Cell::new(false)),
                url: Rc::new(RefCell::new("about:blank".into())),
                is_loading: Rc::new(Cell::new(false)),
                progress: Rc::new(Cell::new(0.0)),
                engine: None,
                event_sink: None,
            });
            self.active = self.tabs.len() - 1;
        }
    }

    fn sync_tab_titles(ui: &MainWindow, state: &BrowserState) {
        let titles = state
            .tabs
            .iter()
            .map(|tab| slint::SharedString::from(tab.title.borrow().as_str()))
            .collect::<Vec<_>>();
        ui.set_tab_titles(ModelRc::new(VecModel::from(titles)));
    }

    fn sync_ui(ui: &MainWindow, state: &BrowserState) {
        sync_tab_titles(ui, state);
        ui.set_active_tab_index(state.active as i32);
        ui.set_address(state.active_tab().url.borrow().clone().into());
        if let Some(engine) = state.active_tab().engine.as_ref() {
            ui.set_can_go_back(engine.can_go_back());
            ui.set_can_go_forward(engine.can_go_forward());
        } else {
            ui.set_can_go_back(false);
            ui.set_can_go_forward(false);
        }
        ui.set_is_loading(state.active_tab().is_loading.get());
        ui.set_load_progress(state.active_tab().progress.get() as f32);
    }

    unsafe extern "C" fn on_navigation_event(
        context: *mut c_void,
        kind: i32,
        value: *const std::ffi::c_char,
        progress: f64,
    ) {
        if context.is_null() {
            return;
        }
        // SAFETY: the Box<EventSink> is retained by the matching TabState until
        // after its WebKitEngine has been explicitly destroyed on this UI thread.
        let sink = unsafe { &*(context.cast::<EventSink>()) };
        let value = callback_url(value);
        if (NAVIGATION_STARTED..=NAVIGATION_FAILED).contains(&kind) && !value.is_empty() {
            *sink.tab_url.borrow_mut() = value.clone();
        }
        match kind {
            NAVIGATION_STARTED => {
                sink.is_loading.set(true);
                sink.progress.set(0.0);
            }
            NAVIGATION_FINISHED => {
                sink.is_loading.set(false);
                sink.progress.set(1.0);
            }
            NAVIGATION_FAILED => {
                sink.is_loading.set(false);
                sink.progress.set(0.0);
            }
            PAGE_TITLE_CHANGED => {
                let title = if value.trim().is_empty() {
                    "New Tab"
                } else {
                    value.trim()
                };
                *sink.tab_title.borrow_mut() = title.to_owned();
                sink.title_dirty.set(true);
            }
            LOAD_PROGRESS_CHANGED => {
                if progress.is_finite() {
                    sink.progress.set(progress.clamp(0.0, 1.0));
                }
            }
            _ => {}
        }
        if sink.active_tab_id.get() != sink.tab_id {
            return;
        }
        let Some(ui) = sink.ui.upgrade() else {
            return;
        };
        match kind {
            NAVIGATION_STARTED => {
                ui.set_is_loading(true);
                ui.set_load_progress(0.0);
                ui.set_page_status("Loading…".into());
            }
            NAVIGATION_COMMITTED => {
                if !value.is_empty() {
                    ui.set_address(value.clone().into());
                }
                ui.set_page_status("Connected".into());
            }
            NAVIGATION_FINISHED => {
                ui.set_is_loading(false);
                ui.set_load_progress(1.0);
                if !value.is_empty() {
                    ui.set_address(value.clone().into());
                }
                ui.set_page_status("Ready".into());
            }
            NAVIGATION_FAILED => {
                ui.set_is_loading(false);
                ui.set_load_progress(0.0);
                ui.set_page_status(
                    "Navigation failed. Check the address or WebKit network process.".into(),
                );
            }
            _ => {}
        }
    }

    let browser = Rc::new(RefCell::new(BrowserState::new()));
    let active_tab_id = Rc::new(Cell::new(1_u64));
    sync_ui(&ui, &browser.borrow());

    {
        let browser = browser.clone();
        let active_tab_id = active_tab_id.clone();
        let ui_weak = ui.as_weak();
        ui.on_navigate(move |address| {
            let mut state = browser.borrow_mut();
            let normalized = normalize_address(address.as_str());
            let tab = state.active_tab_mut();
            *tab.url.borrow_mut() = normalized.clone();
            tab.is_loading.set(true);
            tab.progress.set(0.0);
            if let Some(engine) = tab.engine.as_mut() {
                if let Err(error) = engine.navigate(&normalized) {
                    eprintln!("Navigation request rejected: {error}");
                }
            }
            active_tab_id.set(tab.id);
            if let Some(ui) = ui_weak.upgrade() {
                sync_ui(&ui, &state);
                ui.set_page_status("Loading…".into());
                ui.set_is_loading(true);
            }
        });
    }
    {
        let browser = browser.clone();
        let active_tab_id = active_tab_id.clone();
        let ui_weak = ui.as_weak();
        ui.on_new_tab(move || {
            let mut state = browser.borrow_mut();
            state.open_tab();
            active_tab_id.set(state.active_tab().id);
            if let Some(ui) = ui_weak.upgrade() {
                sync_ui(&ui, &state);
                ui.set_page_status("New tab".into());
            }
        });
    }
    {
        let browser = browser.clone();
        let active_tab_id = active_tab_id.clone();
        let ui_weak = ui.as_weak();
        ui.on_select_tab(move |index| {
            let mut state = browser.borrow_mut();
            if index < 0 || index as usize >= state.tabs.len() {
                return;
            }
            state.active = index as usize;
            let id = state.active_tab().id;
            active_tab_id.set(id);
            if let Some(engine) = state.active_tab_mut().engine.as_mut() {
                engine.set_visible(true);
                engine.focus();
            }
            if let Some(ui) = ui_weak.upgrade() {
                sync_ui(&ui, &state);
                ui.set_page_status("Tab switched".into());
            }
        });
    }
    {
        let browser = browser.clone();
        let active_tab_id = active_tab_id.clone();
        let ui_weak = ui.as_weak();
        ui.on_close_active_tab(move || {
            let mut state = browser.borrow_mut();
            if state.tabs.len() == 1 {
                let tab = state.active_tab_mut();
                *tab.url.borrow_mut() = "about:blank".into();
                *tab.title.borrow_mut() = "New Tab".into();
                tab.title_dirty.set(true);
                tab.is_loading.set(false);
                tab.progress.set(0.0);
                if let Some(engine) = tab.engine.as_mut() {
                    let _ = engine.navigate("about:blank");
                }
            } else {
                let index = state.active;
                // Explicitly destroy the native view before its callback context.
                state.tabs[index].engine.take();
                state.tabs[index].event_sink.take();
                state.tabs.remove(index);
                state.active = index.min(state.tabs.len() - 1);
            }
            active_tab_id.set(state.active_tab().id);
            if let Some(ui) = ui_weak.upgrade() {
                sync_ui(&ui, &state);
                ui.set_page_status("Tab closed".into());
            }
        });
    }
    {
        let browser = browser.clone();
        ui.on_go_back(move || {
            if let Some(engine) = browser.borrow_mut().active_tab_mut().engine.as_mut() {
                engine.back();
            }
        });
    }
    {
        let browser = browser.clone();
        ui.on_go_forward(move || {
            if let Some(engine) = browser.borrow_mut().active_tab_mut().engine.as_mut() {
                engine.forward();
            }
        });
    }
    {
        let browser = browser.clone();
        ui.on_reload(move || {
            if let Some(engine) = browser.borrow_mut().active_tab_mut().engine.as_mut() {
                engine.reload();
            }
        });
    }

    // Release every WebKit page while the parent HWND is still alive.
    {
        let browser = browser.clone();
        ui.window().on_close_requested(move || {
            let mut state = browser.borrow_mut();
            for tab in &mut state.tabs {
                tab.engine.take();
                tab.event_sink.take();
            }
            slint::CloseRequestResponse::HideWindow
        });
    }

    let timer = Timer::default();
    {
        let ui = ui.clone_strong();
        let browser = browser.clone();
        let active_tab_id = active_tab_id.clone();
        let host_hwnd = Cell::new(std::ptr::null_mut::<c_void>());
        timer.start(TimerMode::Repeated, Duration::from_millis(80), move || {
            let x = ui.get_browser_x().round() as i32;
            let y = ui.get_browser_y().round() as i32;
            let width = ui.get_browser_width().round() as i32;
            let height = ui.get_browser_height().round() as i32;
            if width <= 0 || height <= 0 {
                return;
            }

            if host_hwnd.get().is_null() {
                let handle = ui.window().window_handle();
                let Ok(raw_handle) = handle.window_handle() else {
                    return;
                };
                let RawWindowHandle::Win32(win32) = raw_handle.as_raw() else {
                    ui.set_page_status(
                        "Slint did not provide a Win32 HWND; WebKit embedding is unavailable."
                            .into(),
                    );
                    return;
                };
                host_hwnd.set(win32.hwnd.get() as *mut c_void);
            }

            let mut state = browser.borrow_mut();
            let active_index = state.active;
            let titles_changed = state.tabs.iter().any(|tab| tab.title_dirty.replace(false));
            if titles_changed {
                sync_tab_titles(&ui, &state);
            }
            ui.set_is_loading(state.active_tab().is_loading.get());
            ui.set_load_progress(state.active_tab().progress.get() as f32);
            if state.tabs[active_index].engine.is_none() {
                let tab_id = state.tabs[active_index].id;
                let sink = Box::new(EventSink {
                    ui: ui.as_weak(),
                    tab_id,
                    active_tab_id: active_tab_id.clone(),
                    tab_url: state.tabs[active_index].url.clone(),
                    tab_title: state.tabs[active_index].title.clone(),
                    title_dirty: state.tabs[active_index].title_dirty.clone(),
                    is_loading: state.tabs[active_index].is_loading.clone(),
                    progress: state.tabs[active_index].progress.clone(),
                });
                let context = (&*sink as *const EventSink).cast_mut().cast::<c_void>();
                // SAFETY: the Slint HWND and all WKViews are created, resized,
                // hidden, and destroyed on this same UI thread.
                match unsafe {
                    WebKitEngine::new(
                        host_hwnd.get(),
                        (x, y, width, height),
                        on_navigation_event,
                        context,
                    )
                } {
                    Ok(mut engine) => {
                        let initial_url = state.tabs[active_index].url.borrow().clone();
                        if let Err(error) = engine.navigate(&initial_url) {
                            ui.set_page_status(
                                format!("Initial navigation failed: {error}").into(),
                            );
                        }
                        engine.set_visible(true);
                        state.tabs[active_index].engine = Some(engine);
                        state.tabs[active_index].event_sink = Some(sink);
                        ui.set_page_status("WebKit tab created".into());
                    }
                    Err(error) => {
                        ui.set_page_status(format!("WebKit initialization failed: {error}").into());
                        return;
                    }
                }
            }

            for (index, tab) in state.tabs.iter_mut().enumerate() {
                if let Some(engine) = tab.engine.as_mut() {
                    engine.resize(x, y, width, height);
                    engine.set_visible(index == active_index);
                }
            }
            if let Some(engine) = state.tabs[active_index].engine.as_ref() {
                ui.set_can_go_back(engine.can_go_back());
                ui.set_can_go_forward(engine.can_go_forward());
            }
        });
    }

    let result = ui.run();
    timer.stop();
    {
        let mut state = browser.borrow_mut();
        for tab in &mut state.tabs {
            tab.engine.take();
            tab.event_sink.take();
        }
    }
    result?;
    Ok(())
}
