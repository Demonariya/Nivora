//! Nivora application entrypoints shared by the desktop executable and CEF bootstrap DLL.

use std::error::Error;

/// Start the Slint application on the calling UI thread.
pub fn run_ui() -> Result<(), Box<dyn Error>> {
    nivora_ui::run()
}

/// Stable C ABI entrypoint called by the native CEF bootstrap client.
///
/// The CEF bootstrap client performs CEF child-process dispatch first. Only
/// the browser-process path reaches this function and creates the Slint UI.
#[unsafe(no_mangle)]
pub extern "C" fn nivora_rust_run_ui() -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_ui)) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Nivora UI startup failed: {error}");
            1
        }
        Err(_) => {
            eprintln!("Nivora UI startup panicked");
            2
        }
    }
}
