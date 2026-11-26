use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=WEBKIT_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=WEBKIT_LIB_DIR");
    println!("cargo:rerun-if-env-changed=NIVORA_SKIP_WEBKIT_NATIVE_FOR_CHECK");
    println!("cargo:rerun-if-changed=native/CMakeLists.txt");
    println!("cargo:rerun-if-changed=native/webkit_bridge.cpp");

    let target = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target != "windows" {
        println!(
            "cargo:warning=WinCairo native bridge is Windows-only; this host build uses the explicit unsupported-platform Rust path."
        );
        return;
    }

    if env::var_os("NIVORA_SKIP_WEBKIT_NATIVE_FOR_CHECK").is_some() {
        println!(
            "cargo:warning=Rust cross-target type-check only: the WinCairo C++ bridge was NOT compiled or linked. Never set this variable for a native build."
        );
        return;
    }

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    assert_eq!(
        target_arch, "x86_64",
        "Nivora's initial WinCairo target is Windows x86_64 only"
    );

    let include_dir = PathBuf::from(env::var_os("WEBKIT_INCLUDE_DIR").expect(
        "WEBKIT_INCLUDE_DIR is required for Windows builds (directory containing WebKit/WKBase.h)",
    ));
    let lib_dir = PathBuf::from(env::var_os("WEBKIT_LIB_DIR").expect(
        "WEBKIT_LIB_DIR is required for Windows builds (directory containing WebKit2.lib)",
    ));

    for header in [
        "WKBase.h",
        "WKPage.h",
        "WKPageConfigurationRef.h",
        "WKFramePolicyListener.h",
        "WKPageNavigationClient.h",
        "WKString.h",
        "WKType.h",
        "WKURL.h",
        "WKView.h",
    ] {
        let path = include_dir.join("WebKit").join(header);
        assert!(
            path.is_file(),
            "missing required upstream WebKit header: {}",
            path.display()
        );
    }
    let import_library = lib_dir.join("WebKit2.lib");
    assert!(
        import_library.is_file(),
        "missing WebKit2.lib at {}; set WEBKIT_LIB_DIR to the directory containing the WinCairo import library",
        import_library.display()
    );

    let destination = cmake::Config::new("native")
        .define("WEBKIT_INCLUDE_DIR", include_dir.to_string_lossy().as_ref())
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        destination.join("lib").display()
    );
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=nivora_webkit_bridge");
    println!("cargo:rustc-link-lib=dylib=WebKit2");
    println!("cargo:rustc-link-lib=user32");
}
