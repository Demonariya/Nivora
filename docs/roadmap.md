# Roadmap and implementation status

Status labels describe evidence in this repository, not intended work.

## Milestone 0 — repository and workspace

- [x] Public GitHub repository created.
- [x] Six Cargo crates and manifests added.
- [x] Slint UI source added.
- [x] SQLite schema and tab model source added.
- [x] `cargo fmt --all --check` passed on a GitHub Actions Linux runner.
- [x] `cargo check --workspace --locked` passed on GitHub Actions Linux.
- [x] `cargo test --workspace --locked` passed on GitHub Actions Linux.
- [x] Windows-target Rust type-check passed on GitHub Actions (this check does not compile the C++ bridge).
- [x] Windows rendering-engine candidates evaluated at architecture/documentation level (`docs/rendering-engine-evaluation.md`).
- [x] Engine-neutral Rust backend/view traits added to nivora-engine; existing WebKitEngine preserved behind WebKitBackend.
- [x] Conservative engine capability reporting and unit test added (CI execution pending).
- [ ] CEF Chrome-runtime spike built and tested against a pinned Windows distribution.
- [ ] Production rendering backend decision accepted by Windows runtime tests.
- [ ] Android application/client repository and shared-core integration located or established.

## UI/UX milestone — native Slint chrome

- [x] Shared Dark/Light design tokens.
- [x] Reusable IconButton, TextButton, TabItem, and AddressField components.
- [x] Compact tab strip and navigation/address toolbar.
- [x] Optional vertical presentation for the current tab.
- [x] Functional Quick Settings for appearance and tab placement.
- [x] Hover, pressed, disabled, focus, and loading feedback.
- [x] Accessible names, roles, and default actions for custom controls.
- [x] Keep the settings popover inside the Slint chrome region so the native WebKit child HWND cannot cover it.
- [ ] Follow Windows System appearance automatically.
- [x] Independent WKView/WKPage ownership per tab, lazy view creation, tab switching, and close-time teardown (source implementation; native runtime validation pending).
- [x] Main-page title updates from WKPageStateClient, including background-tab title retention.
- [x] Per-tab close controls and middle-click tab closing (source implementation; native runtime validation pending).
- [ ] Favicon retrieval (the selected WinCairo SDK's public C API path is not yet verified).
- [x] Horizontal tab drag-and-drop reordering using Slint DragArea/DropArea and stable tab IDs (native runtime validation pending).
- [x] Engine-reported loading progress from WKPageGetEstimatedProgress.
- [ ] Search suggestions and security/TLS indicators.
- [ ] Functional bookmarks, history, downloads, passwords, and extensions panels.
- [ ] Settings categories and accessibility keyboard traversal tests.
- [ ] Windows visual QA at small/normal/maximized sizes and 100%–200% DPI.

## Milestone 1 — first WinCairo view

- [x] Inspect upstream WinCairo WKView header and implementation.
- [x] Add C++ bridge calling the public WKView/WKPage APIs.
- [x] Add Rust ownership wrapper and navigation event callback ABI.
- [x] Add Slint raw HWND extraction and geometry synchronization.
- [ ] Compile bridge with MSVC against a real WebKit SDK.
- [ ] Verify required exports in the exact WebKit2.dll.
- [ ] Launch and navigate to a live HTTPS site.
- [ ] Verify Back, Forward, Reload, resize, focus, DPI, and teardown.

## Milestone 2 — browser core

- [x] Multi-tab WebKit view lifecycle source implementation (native runtime validation pending).
- [x] Persist completed normal-mode navigations in SQLite under %APPDATA%\\Nivora\\browser.sqlite3 (history API supports recent, search, delete, and clear; UI panel pending).
- [ ] Persist bookmarks in SQLite.
- [ ] Downloads and permission UI.
- [ ] Private browsing with isolated ephemeral website data store.
- [ ] Password storage using Windows DPAPI and WebAuthn integration.
- [ ] Site permissions, content settings, and privacy controls.

## Milestone 3 — Chromium extensions (conditional on CEF spike)

- [ ] Pin CEF branch, Chromium version, Chrome runtime distribution and license notices.
- [ ] Capture `chrome://extensions-support` API inventory for the pinned build.
- [ ] Verify MV3 service-worker lifecycle and required APIs.
- [ ] Implement isolated `nivora-cef-sys` backend spike without deleting WebKit.
- [ ] Integrate extension action/popup/options UI and permission prompts.
- [ ] Test uBlock Origin, Dark Reader, Bitwarden, Tampermonkey, SponsorBlock and Privacy Badger.
- [ ] Publish an evidence-based extension compatibility matrix.

## Milestone 4 — product hardening

- [ ] Crash recovery and process termination handling.
- [x] Win32 thread-message keyboard shortcuts: Ctrl+T, Ctrl+W, Ctrl+Tab, Ctrl+Shift+Tab (native runtime validation pending).
- [ ] Accessibility keyboard traversal tests.
- [ ] Installer, signed releases, update channel.
- [ ] Native Windows UI automation and end-to-end regression tests.

**Current milestone:** the engine-neutral Rust backend contract is implemented in source and documented; CI has not yet reported on this commit. The existing WebKit implementation remains the active baseline. CEF is the leading candidate, but a pinned Chrome-runtime Windows spike and extension compatibility tests are required before any production backend switch. The first working native browser is not yet accepted because Windows build/runtime tests remain outstanding.
