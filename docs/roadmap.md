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
- [x] Independent WKView/WKPage ownership per tab, lazy view creation, tab switching, and close-time teardown (source implementation; native runtime validation pending).\n- [x] Main-page title updates from WKPageStateClient, including background-tab title retention.\n- [ ] Favicon retrieval and tab drag-and-drop.
- [x] Engine-reported loading progress from WKPageGetEstimatedProgress.\n- [ ] Search suggestions and security/TLS indicators.
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
- [ ] Persist history and bookmarks in SQLite.
- [ ] Downloads and permission UI.
- [ ] Private browsing with isolated ephemeral website data store.
- [ ] Password storage using Windows DPAPI and WebAuthn integration.
- [ ] Site permissions, content settings, and privacy controls.

## Milestone 3 — product hardening

- [ ] Crash recovery and process termination handling.
- [ ] Accessibility and keyboard shortcuts.
- [ ] Installer, signed releases, update channel.
- [ ] Native Windows UI automation and end-to-end regression tests.

**Current milestone:** source implementation is present, but the first working native browser is not yet accepted because Windows build/runtime tests remain outstanding.
