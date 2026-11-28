fn main() {
    println!("cargo:rustc-check-cfg=cfg(cef_sdk_available)");
    println!("cargo:rerun-if-env-changed=NIVORA_CEF_ROOT");

    // SDK validation is opt-in at build time so Linux CI and developers without
    // the proprietary-size binary distribution can still compile the workspace.
    let Some(root) = std::env::var_os("NIVORA_CEF_ROOT") else {
        println!("cargo:warning=NIVORA_CEF_ROOT is unset; CEF native runtime is not enabled");
        return;
    };

    let root = std::path::PathBuf::from(root);
    let required = [
        "include/cef_version.h",
        "include/cef_app.h",
        "Release/libcef.dll",
        "Release/libcef.lib",
        "Resources/icudtl.dat",
        "Resources/resources.pak",
    ];

    for relative in required {
        if !root.join(relative).is_file() {
            panic!(
                "NIVORA_CEF_ROOT={} is invalid: missing {}",
                root.display(),
                relative
            );
        }
    }

    println!("cargo:rustc-cfg=cef_sdk_available");
}
