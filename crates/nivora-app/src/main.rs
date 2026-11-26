fn main() {
    if let Err(error) = nivora_ui::run() {
        eprintln!("Nivora failed to start: {error}");
        std::process::exit(1);
    }
}
