//! CEF SDK gate for Nivora.
//!
//! The pinned CEF 156 Windows distribution uses the official bootstrap
//! executable for sandboxed clients. This crate currently validates the SDK;
//! it does not yet initialize or embed CEF in the Rust/Slint application.

/// Pinned CEF distribution selected for the initial Windows integration spike.
pub const CEF_VERSION: &str = "156.0.14+g4fa4ab5+chromium-156.0.8078.48";
pub const CHROMIUM_VERSION: &str = "156.0.8078.48";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CefSdkError {
    NotConfigured,
    InvalidRoot,
    MissingPath(&'static str),
}

impl std::fmt::Display for CefSdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => write!(f, "NIVORA_CEF_ROOT is not configured"),
            Self::InvalidRoot => write!(
                f,
                "NIVORA_CEF_ROOT does not point to the pinned CEF Windows x64 distribution"
            ),
            Self::MissingPath(path) => {
                write!(f, "CEF distribution is missing required path: {path}")
            }
        }
    }
}

impl std::error::Error for CefSdkError {}

/// Returns whether this build was configured with a validated CEF SDK.
pub const fn sdk_available() -> bool {
    cfg!(cef_sdk_available)
}

/// Validate the standard CEF Windows x64 distribution layout and version.
///
/// Set NIVORA_CEF_ROOT to the standard distribution root containing
/// include/, Release/libcef.dll, Release/libcef.lib and Resources/.
pub fn validate_sdk(root: impl AsRef<std::path::Path>) -> Result<(), CefSdkError> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Err(CefSdkError::InvalidRoot);
    }

    for (relative, label) in [
        ("include/cef_version.h", "include/cef_version.h"),
        ("include/cef_app.h", "include/cef_app.h"),
        ("include/cef_sandbox_win.h", "include/cef_sandbox_win.h"),
        ("include/cef_version_info.h", "include/cef_version_info.h"),
        ("Release/libcef.dll", "Release/libcef.dll"),
        ("Release/libcef.lib", "Release/libcef.lib"),
        ("Resources/icudtl.dat", "Resources/icudtl.dat"),
        ("Resources/cef.pak", "Resources/cef.pak"),
    ] {
        if !root.join(relative).is_file() {
            return Err(CefSdkError::MissingPath(label));
        }
    }

    let header = std::fs::read_to_string(root.join("include/cef_version.h"))
        .map_err(|_| CefSdkError::MissingPath("readable include/cef_version.h"))?;

    for (key, expected) in [
        ("CEF_VERSION_MAJOR", "156"),
        ("CEF_VERSION_MINOR", "0"),
        ("CEF_VERSION_PATCH", "14"),
        ("CHROME_VERSION_MAJOR", "156"),
        ("CHROME_VERSION_MINOR", "0"),
        ("CHROME_VERSION_BUILD", "8078"),
        ("CHROME_VERSION_PATCH", "48"),
    ] {
        let prefix = format!("#define {key} ");
        let actual = header
            .lines()
            .find_map(|line| line.trim().strip_prefix(&prefix))
            .map(str::trim);

        if actual != Some(expected) {
            return Err(CefSdkError::InvalidRoot);
        }
    }

    Ok(())
}

/// Validate the SDK path configured for this build.
pub fn configured_sdk() -> Result<std::path::PathBuf, CefSdkError> {
    let root = std::env::var_os("NIVORA_CEF_ROOT").ok_or(CefSdkError::NotConfigured)?;
    let root = std::path::PathBuf::from(root);
    validate_sdk(&root)?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::{CEF_VERSION, CHROMIUM_VERSION, CefSdkError, validate_sdk};
    use std::fs;

    #[test]
    fn pins_cef_and_chromium_versions() {
        assert!(CEF_VERSION.starts_with("156.0.14+"));
        assert_eq!(CHROMIUM_VERSION, "156.0.8078.48");
    }

    #[test]
    fn rejects_nonexistent_sdk_root() {
        let path = std::env::temp_dir().join("nivora-cef-sdk-does-not-exist");
        assert_eq!(validate_sdk(path), Err(CefSdkError::InvalidRoot));
    }

    #[test]
    fn validates_standard_distribution_layout() {
        let root = std::env::temp_dir().join(format!("nivora-cef-sdk-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        for relative in [
            "include/cef_version.h",
            "include/cef_app.h",
            "include/cef_sandbox_win.h",
            "include/cef_version_info.h",
            "Release/libcef.dll",
            "Release/libcef.lib",
            "Resources/icudtl.dat",
            "Resources/cef.pak",
        ] {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();

            let contents = if relative == "include/cef_version.h" {
                b"#define CEF_VERSION_MAJOR 156\n#define CEF_VERSION_MINOR 0\n#define CEF_VERSION_PATCH 14\n#define CHROME_VERSION_MAJOR 156\n#define CHROME_VERSION_MINOR 0\n#define CHROME_VERSION_BUILD 8078\n#define CHROME_VERSION_PATCH 48\n".as_slice()
            } else {
                b"test fixture".as_slice()
            };

            fs::write(path, contents).unwrap();
        }

        assert_eq!(validate_sdk(&root), Ok(()));

        fs::write(
            root.join("include/cef_version.h"),
            b"#define CEF_VERSION_MAJOR 155",
        )
        .unwrap();
        assert_eq!(validate_sdk(&root), Err(CefSdkError::InvalidRoot));

        fs::remove_dir_all(root).unwrap();
    }
}
