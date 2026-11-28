fn main() {
    println!("cargo:rustc-check-cfg=cfg(cef_sdk_available)");
    println!("cargo:rerun-if-env-changed=NIVORA_CEF_ROOT");

    // SDK validation is opt-in so Linux CI and developers without the large
    // binary distribution can compile the workspace without pretending CEF runs.
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
        "Release/cef_sandbox.lib",
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

    let header = std::fs::read_to_string(root.join("include/cef_version.h"))
        .expect("failed to read include/cef_version.h");
    let expected = [
        ("CEF_VERSION_MAJOR", "156"),
        ("CEF_VERSION_MINOR", "0"),
        ("CEF_VERSION_PATCH", "14"),
        ("CHROME_VERSION_MAJOR", "156"),
        ("CHROME_VERSION_MINOR", "0"),
        ("CHROME_VERSION_BUILD", "8078"),
        ("CHROME_VERSION_PATCH", "48"),
    ];

    for (key, value) in expected {
        let prefix = format!("#define {key} ");
        let actual = header
            .lines()
            .find_map(|line| line.trim().strip_prefix(&prefix))
            .map(str::trim);
        if actual != Some(value) {
            panic!(
                "CEF SDK version mismatch for {key}: expected {value}, got {}",
                actual.unwrap_or("<missing>")
            );
        }
    }

    println!("cargo:rustc-cfg=cef_sdk_available");
}
