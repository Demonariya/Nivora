# Contributing

1. Open an issue for substantial changes before starting implementation.
2. Keep the Rust/C++ boundary narrow. Do not expose WebKit opaque structures to Rust.
3. Use upstream public WinCairo headers; do not re-declare WebKit ABI structures.
4. Run cargo fmt --all --check, cargo check --workspace, and cargo test --workspace.
5. For native changes, build with MSVC against one exact WinCairo revision and run scripts/verify-webkit-exports.ps1.
6. Record Windows runtime test results, including WebKit revision, Windows version, renderer, DPI scale, and whether helper processes launched.
7. Do not commit WebKit binaries, generated build products, credentials, or user data.

Keep commits focused and use conventional commit prefixes: feat, fix, docs, test, build.
