fn main() {
    slint_build::compile("ui/main.slint").expect("failed to compile Nivora Slint UI");
    println!("cargo:rerun-if-changed=ui/main.slint");
}
