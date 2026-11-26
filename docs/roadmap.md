# Roadmap and implementation status

Status reflects evidence in the repository, not intended work.

## Milestone 0 — repository and workspace

- [x] Public GitHub repository created.
- [x] Six Cargo crates and manifests added.
- [x] Slint UI source added.
- [x] SQLite schema and tab model source added.
- [ ] Rust formatting, check, and tests run on a Rust-enabled host.

## Milestone 1 — first WinCairo view

- [x] Inspect upstream WinCairo WKView header and implementation.
- [x] Add C++ bridge calling public WKView/WKPage APIs.
- [x] Add Rust ownership wrapper and navigation event callback ABI.
- [x] Add Slint raw HWND extraction and geometry synchronization.
- [ ] Compile bridge with MSVC against a real WebKit SDK.
- [ ] Verify exports in the exact WebKit2.dll.
- [ ] Launch and navigate to a live HTTPS site.
- [ ] Verify Back, Forward, Reload, resize, focus, DPI, and teardown.

## Milestone 2 — browser core

- [ ] Multi-tab WebKit view lifecycle.
- [ ] Persist history and bookmarks in SQLite.
- [ ] Downloads and permission UI.
- [ ] Private browsing with isolated ephemeral website data store.
- [ ] Password storage using Windows DPAPI and WebAuthn.
- [ ] Site permissions, content settings, and privacy controls.

## Milestone 3 — product hardening

- [ ] Crash recovery and process termination handling.
- [ ] Accessibility and keyboard shortcuts.
- [ ] Installer, signed releases, and update channel.
- [ ] Native Windows UI automation and end-to-end regression tests.

Current milestone: source implementation is present, but the first working native browser is not accepted until Windows build and runtime tests pass.
