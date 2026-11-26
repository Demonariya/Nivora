//! Slint shell and Win32 child-window host for the WebKit WinCairo view.

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
            "WinCairo WebKit is Windows-only. Run Nivora on Windows with WEBKIT_INCLUDE_DIR and WEBKIT_LIB_DIR configured.",
        );
        ui.run()?;
    }

    Ok(())
}

#[cfg(windows)]
fn run_with_webkit(ui: MainWindow) -> Result<(), Box<dyn Error>> {
    use nivora_engine::{
        callback_url, WebKitEngine, NAVIGATION_COMMITTED, NAVIGATION_FAILED,
        NAVIGATION_FINISHED, NAVIGATION_STARTED,
    };
    use raw_window_handle_06::{HasWindowHandle, RawWindowHandle};
    use slint::{Timer, TimerMode};
    use std::{
        cell::{Cell, RefCell},
        ffi::c_void,
        rc::Rc,
        time::Duration,
    };

    struct EventSink {
        ui: slint::Weak<MainWindow>,
    }

    unsafe extern "C" fn on_navigation_event(
        context: *mut c_void,
        kind: i32,
        url: *const std::ffi::c_char,
    ) {
        if context.is_null() {
            return;
        }
        // SAFETY: context points to the EventSink Box kept alive until after the
        // WebKit view is explicitly destroyed on the same UI thread.
        let sink = unsafe { &*(context.cast::<EventSink>()) };
        let Some(ui) = sink.ui.upgrade() else {
            return;
        };
        let url = callback_url(url);
        match kind {
            NAVIGATION_STARTED => {
                ui.set_is_loading(true);
                ui.set_page_status("Loading…");
            }
            NAVIGATION_COMMITTED => {
                if !url.is_empty() {
                    ui.set_address(url.into());
                }
                ui.set_page_status("Connected");
            }
            NAVIGATION_FINISHED => {
                ui.set_is_loading(false);
                if !url.is_empty() {
                    ui.set_address(url.into());
                }
                ui.set_page_status("Ready");
            }
            NAVIGATION_FAILED => {
                ui.set_is_loading(false);
                ui.set_page_status("Navigation failed. Check the address or WebKit network process.");
            }
            _ => {}
        }
    }

    let browser: Rc<RefCell<Option<WebKitEngine>>> = Rc::new(RefCell::new(None));
    let initialized = Rc::new(Cell::new(false));
    let event_sink = Box::new(EventSink { ui: ui.as_weak() });
    let event_context = (&*event_sink as *const EventSink)
        .cast_mut()
        .cast::<c_void>();

    {
        let browser = browser.clone();
        ui.on_navigate(move |address| {
            if let Some(engine) = browser.borrow_mut().as_mut() {
                if let Err(error) = engine.navigate(address.as_str()) {
                    eprintln!("Navigation request rejected: {error}");
                }
            }
        });
    }
    {
        let browser = browser.clone();
        ui.on_go_back(move || {
            if let Some(engine) = browser.borrow_mut().as_mut() {
                engine.back();
            }
        });
    }
    {
        let browser = browser.clone();
        ui.on_go_forward(move || {
            if let Some(engine) = browser.borrow_mut().as_mut() {
                engine.forward();
            }
        });
    }
    {
        let browser = browser.clone();
        ui.on_reload(move || {
            if let Some(engine) = browser.borrow_mut().as_mut() {
                engine.reload();
            }
        });
    }

    // Release WebKit while the parent HWND is still alive. Slint hides the
    // window only after this close-request callback returns.
    {
        let browser = browser.clone();
        ui.window().on_close_requested(move || {
            browser.borrow_mut().take();
            slint::CloseRequestResponse::HideWindow
        });
    }

    let timer = Timer::default();
    {
        let ui = ui.clone_strong();
        let browser = browser.clone();
        let initialized = initialized.clone();
        timer.start(
            TimerMode::Repeated,
            Duration::from_millis(80),
            move || {
                // Slint maps length properties to physical pixels in Rust,
                // applying the window's scale factor exactly once.
                let x = ui.get_browser_x().round() as i32;
                let y = ui.get_browser_y().round() as i32;
                let width = ui.get_browser_width().round() as i32;
                let height = ui.get_browser_height().round() as i32;

                if !initialized.get() {
                    let handle = ui.window().window_handle();
                    let Ok(raw_handle) = handle.window_handle() else {
                        return;
                    };
                    let RawWindowHandle::Win32(win32) = raw_handle.as_raw() else {
                        ui.set_page_status(
                            "Slint did not provide a Win32 HWND; WebKit embedding is unavailable.",
                        );
                        return;
                    };
                    let hwnd = win32.hwnd.get() as *mut c_void;
                    // SAFETY: Slint's Win32 backend owns this HWND and this timer
                    // runs on the Slint UI thread after the window has been shown.
                    match unsafe {
                        WebKitEngine::new(
                            hwnd,
                            (x, y, width, height),
                            on_navigation_event,
                            event_context,
                        )
                    } {
                        Ok(mut engine) => {
                            if let Err(error) = engine.navigate(ui.get_address().as_str()) {
                                ui.set_page_status(
                                    format!("Initial navigation failed: {error}").into(),
                                );
                            } else {
                                ui.set_page_status("WebKit view created; loading initial page…");
                            }
                            engine.focus();
                            *browser.borrow_mut() = Some(engine);
                            initialized.set(true);
                        }
                        Err(error) => {
                            ui.set_page_status(
                                format!("WebKit initialization failed: {error}").into(),
                            );
                            initialized.set(true);
                        }
                    }
                } else if let Some(engine) = browser.borrow_mut().as_mut() {
                    engine.resize(x, y, width, height);
                    ui.set_can_go_back(engine.can_go_back());
                    ui.set_can_go_forward(engine.can_go_forward());
                }
            },
        );
    }

    let result = ui.run();
    timer.stop();
    // Destroy WKPage/WKView before releasing callback context.
    browser.borrow_mut().take();
    drop(event_sink);
    result?;
    Ok(())
}
