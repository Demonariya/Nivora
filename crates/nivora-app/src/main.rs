fn main() {
    if let Err(error) = nivora_app::run_ui() {
        eprintln!("Nivora failed to start: {error}");
        std::process::exit(1);
    }
}
