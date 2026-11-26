# Third-party license notes

This document records the direct engine/UI licensing checks made for the initial Nivora workspace. It is not a substitute for reviewing the exact source files and notices in the WebKit revision and Cargo dependency graph used for a release.

## Slint 1.19.1

The Slint framework is triple-licensed under the Royalty-free Desktop/Mobile/Web Applications License, GPLv3, or a commercial license. Its upstream license states that the royalty-free option permits proprietary desktop applications at no cost when the application visibly discloses Slint, for example through the AboutSlint widget or the Slint attribution badge on a public webpage. Embedded-system use is excluded from that royalty-free option.

Nivora's README displays the official Slint attribution badge. Keep that attribution visible on the public project/download page when distributing under the royalty-free option. If choosing GPLv3 or a commercial license instead, follow the corresponding upstream terms. Nivora's own source files remain under the MIT license; that does not relicense Slint or other dependencies.

Upstream version-pinned license: https://github.com/slint-ui/slint/blob/v1.19.1/LICENSE.md
Upstream licensing FAQ: https://github.com/slint-ui/slint/blob/v1.19.1/FAQ.md#licensing

## WebKit WinCairo

WebKit contains components under LGPL and BSD-style licenses, with licensing varying by file. Do not assume one license covers the entire WebKit tree. Nivora does not vendor WebKit source or binary artifacts. When building or redistributing WebKit2.dll, helper processes, libraries, or other runtime files, preserve the notices and corresponding source/offer obligations applicable to the exact WebKit revision and all bundled components.

Upstream overview: https://webkit.org/project/
Upstream licensing page: https://webkit.org/licensing-webkit/
Detailed licensing documentation: https://docs.webkit.org/Other/Licensing.html

## Rust dependencies

Cargo.lock pins the resolved Rust dependency graph. Review the license metadata and notices for every resolved crate before producing a redistributable installer. No compiled third-party dependencies or WebKit binaries are committed to this repository.
