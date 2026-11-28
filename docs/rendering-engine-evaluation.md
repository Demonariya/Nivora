# Windows Rendering Engine Evaluation

**Status:** Architecture decision gate — no engine migration is authorized by this document alone.

**Scope:** Select a Windows rendering backend that can be hosted by the existing Rust + Slint application and that provides the strongest practical foundation for desktop Chrome extension compatibility.

## Executive conclusion

The current repository uses a custom Rust/C++ bridge to WebKit WinCairo. That implementation must remain intact until a replacement has passed a Windows build and runtime acceptance test.

For the new product requirements, **CEF with its Chrome runtime is the leading candidate**. CEF has a native extension subsystem and exposes a runtime-specific supported-API list through `chrome://extensions-support`. This is materially closer to the requested Chrome extension model than implementing extension APIs over a generic webview. It is not equivalent to Google Chrome: API coverage, lifecycle behavior, UI hosting, installation, and MV3 service-worker behavior must be measured against the exact pinned CEF/Chromium build.

**WebView2 is a viable fallback for ordinary browsing and selected extensions, but is not the preferred engine for the stated compatibility target.** Its documented browser-extension API installs unpacked extension folders and exposes profile-level enable/remove management. Microsoft also documents that extensions requiring browser UI entry points (for example action icons, badges, and popups) can load but have missing entry points unless the host supplies equivalent UI. Its API surface is Edge/WebView2-specific, not a guarantee of Chrome Web Store parity.

No engine is selected as production until the acceptance gates below are satisfied.

## Candidate comparison

| Area | Existing WebKit WinCairo | CEF (Chrome runtime) | Microsoft WebView2 |
|---|---|---|---|
| Current repository integration | Implemented Rust/C++ bridge and Slint child HWND hosting; native Windows runtime still unverified | Not integrated | Not integrated |
| Chromium engine | No | Yes | Yes (Microsoft Edge WebView2 Runtime) |
| Chrome extension subsystem | No built-in Chrome extension platform | Yes, with a documented supported API subset / runtime-specific behavior | Supports browser extensions through WebView2 profile APIs; do not assume Chrome parity |
| MV3 | Not a native Chrome extension runtime | Must verify exact CEF build: service workers, events, permissions, DNR and APIs | Must verify exact runtime and extension; WebView2 documents Edge extension support, not full Chrome compatibility |
| Extension UI | Must be built by Nivora | Host must provide browser chrome / extension views appropriately; verify popup and action integration | Microsoft documents missing UI entry points for extension action/badge/popup scenarios unless the host provides them |
| Background execution | WebKit-specific facilities; no Chrome extension worker model | Extension runtime support exists; exact MV3 lifecycle must be tested | Extension runtime is managed by WebView2; exact MV3 behavior and API coverage must be tested |
| Native integration | Existing HWND child-view bridge | C API / C++ wrapper; Rust FFI required | Win32 COM interfaces; Rust COM/Win32 bindings required |
| Runtime distribution | Large WebKit build and helper runtime | CEF binary distribution and subprocess/resource packaging | Evergreen runtime can be installed/shared or Fixed Version can be deployed; runtime policy and updates are Microsoft-controlled |
| Maintenance | Track upstream WebKit WinCairo and SDK exports | Pin CEF branch/build and Chromium version; update regularly for security fixes | Track WebView2 SDK and installed Edge Runtime |
| Best fit | Existing work preservation and WebKit-first browser | Best candidate for practical desktop Chrome extension compatibility | Lower-friction Chromium embedding when Edge extension compatibility is sufficient |

## CEF findings

CEF is an embedding framework over Chromium's Content layer; it is not the complete Google Chrome product. Extension support is implemented by CEF's Chrome runtime and must be distinguished from the older Alloy runtime and from custom JavaScript injection.

Relevant upstream behavior:

- CEF exposes extension loading and extension lifecycle interfaces, including request-context extension loading and extension handlers.
- CEF documents that extension pages can access supported `chrome.*` APIs according to the manifest and that the authoritative runtime list is exposed at `chrome://extensions-support`.
- CEF documentation describes creating background browsers for extension background scripts and hosting visible extension views using `chrome-extension://<id>/<path>` URLs.
- The supported API set is not assumed to include every API requested by Nivora. Capture the exact `chrome://extensions-support` output from the pinned build as a checked-in compatibility artifact.
- A Chrome runtime build, not an arbitrary CEF build/runtime configuration, is required for the intended extension architecture.
- The exact CEF branch, Chromium version, Windows architecture, distribution package, and licensing notices must be pinned before implementation.

### Required CEF spike

Build a small native CEF host outside the production app first. Use the same CEF binary distribution that would ship with Nivora. Verify:

1. Startup, shutdown, subprocess launch, resources, locales, and sandbox configuration.
2. Browser creation, navigation, multiple browser instances, profiles/request contexts, cookies, cache, downloads, and hardware acceleration.
3. Extension install from unpacked directory and CRX; ZIP import must be unpacked and validated by Nivora before invoking the supported CEF installation path.
4. `chrome://extensions-support` output and exact support for each required API.
5. MV3 background service worker startup, suspension, wake-up, event delivery, persistence, and shutdown.
6. Content scripts, isolated worlds, extension messaging, storage, permissions, action popup/options pages, context menus, commands, notifications, cookies and DNR.
7. Browser UI integration for extension action icons, badges, popup windows and permission prompts.
8. Real extension smoke tests for uBlock Origin, Dark Reader, Bitwarden, Tampermonkey, SponsorBlock and Privacy Badger.
9. GPU process behavior, crash recovery, process isolation, memory baseline and media/WebRTC.
10. Runtime update cadence, package size, redistribution obligations, and security patch process.

Do not report an extension as compatible based only on successful installation. A compatibility result requires the extension's key user workflows to pass.

## WebView2 findings

WebView2 has a supported extension model with profile APIs such as `AddBrowserExtension`, `GetBrowserExtensions`, enable/disable and removal. The documented installation API takes a local unpacked extension folder. Microsoft explicitly notes that extensions designed around browser UI interactions such as icons, badges and popups can load while their UI entry points are absent in the WebView2 host.

WebView2 therefore remains a credible fallback for selected unpacked extensions, but it does not meet Nivora's Chrome-like extension objective without substantial host UI work and per-extension validation. Do not advertise Chrome Web Store support or general Chrome compatibility on the basis of the WebView2 extension API alone.

## Architecture requirements independent of engine

Keep the rendering backend behind a Rust trait in `nivora-engine`. The browser core must not depend on CEF, WebKit, WebView2, C++ ABI structures, or raw renderer handles.

The abstraction should cover at least:

- Engine initialization and shutdown.
- Browser/profile context creation and disposal.
- View creation, resize, visibility, focus and teardown.
- Navigation, history traversal, reload, URL/title/loading state.
- Navigation and resource request policy hooks.
- Permission requests and decisions.
- Downloads and download lifecycle.
- Cookie/storage operations needed by browser features.
- DevTools attachment where supported.
- Extension runtime capabilities through a separate capability interface, not fake methods on the base engine trait.

All engine callbacks must be marshalled to the owning UI thread. Raw native handles and engine-owned objects must not cross platform-independent core boundaries. Each backend must declare capabilities so the UI can disable unsupported features honestly.

## Decision gates

CEF becomes the production backend only after all of the following are true:

- A pinned CEF Chrome-runtime build launches on supported Windows versions.
- The Rust FFI boundary is memory-safe by construction as far as practical, audited for ownership/thread rules, and covered by integration tests.
- MV3 service workers and required extension APIs pass tests on the exact build.
- At least the six named target extensions have a recorded, repeatable compatibility result; failures are disclosed.
- Extension UI, permissions and lifecycle are integrated into Nivora's native chrome.
- GPU, audio/video, downloads, profiles, crash handling, installer packaging and update strategy pass acceptance.
- Binary size, idle memory, cold start and security-update cadence are documented.

If CEF fails these gates, retain WebKit for the current browser baseline and reassess WebView2 or a Chromium source-level integration. Do not silently fall back to a backend that cannot meet the extension requirement.

## Current migration policy

- Keep the current WebKit WinCairo implementation buildable while the CEF spike is evaluated.
- Do not delete `nivora-webkit-sys` or rewrite the Slint UI during the spike.
- Put any new backend in a separate crate (for example `nivora-cef-sys`) and keep the production facade in `nivora-engine`.
- Only switch the default backend after the decision gates pass and the Windows acceptance tests are green.
- Keep engine-specific browser data isolated or provide an explicit migration plan; do not assume WebKit cookies/cache can be transparently imported into Chromium.

## References

- CEF request-context extension documentation: https://cef-builds.spotifycdn.com/docs/125.0/classCefRequestContext.html
- CEF extension support history and Chrome runtime distinction: https://github.com/chromiumembedded/cef/issues/1947
- CEF discussion of extension UI / Views hosting: https://magpcss.org/ceforum/viewtopic.php?p=57489
- Chrome MV3 service worker model: https://developer.chrome.com/docs/extensions/develop/migrate/to-service-workers
- Microsoft WebView2 API overview: https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/overview-features-apis
- WebView2 extension profile API and UI limitations: https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2profile7
