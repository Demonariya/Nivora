# Extension API Compatibility Matrix

**Status:** Not yet tested against a pinned Nivora CEF build. Every result is intentionally marked **Not tested**; API names or documentation references are not evidence of working behavior.

## Test environment record

| Field | Required value |
|---|---|
| CEF branch / version | Not pinned |
| Chromium version | Not pinned |
| Runtime | Chrome runtime required; not yet built |
| Windows version / build | Not recorded |
| Architecture | x64 target; not yet tested |
| GPU / driver | Not recorded |
| Nivora commit | Record the exact tested commit |
| Test date | Not run |

## Required API matrix

| API | Priority | Result | Required evidence |
|---|---|---|---|
| `chrome.runtime` | P0 | Not tested | getManifest, getURL, messaging, lifecycle events, lastError |
| `chrome.storage` | P0 | Not tested | local/sync/session areas, quota, onChanged, persistence |
| `chrome.tabs` | P0 | Not tested | query/get/create/update/remove, events, active tab semantics |
| `chrome.windows` | P1 | Not tested | create/get/update/remove and Nivora window mapping |
| `chrome.action` | P0 | Not tested | icon, title, badge, popup, enable/disable per tab |
| `chrome.scripting` | P0 | Not tested | executeScript, insertCSS, frame targeting, isolated world |
| `chrome.webNavigation` | P1 | Not tested | event ordering, redirects, frame IDs |
| `chrome.webRequest` | P1 | Not tested | listeners, blocking support, request stages |
| `chrome.declarativeNetRequest` | P0 | Not tested | static/dynamic/session rules, quotas, redirects, headers |
| `chrome.contextMenus` | P1 | Not tested | create/update/remove, click dispatch, contexts |
| `chrome.notifications` | P2 | Not tested | create/update/clear and native notification mapping |
| `chrome.cookies` | P1 | Not tested | get/set/remove, store IDs, change events |
| `chrome.permissions` | P0 | Not tested | request/contains/remove, optional permissions |
| `chrome.commands` | P1 | Not tested | shortcut registration, conflicts, command dispatch |
| `chrome.i18n` | P2 | Not tested | message substitution, locale fallback, RTL strings |

P0 denotes a blocker for the first extension compatibility release. P1 denotes a high-priority feature. P2 denotes a later feature unless a target extension depends on it.

## Target extension smoke-test matrix

| Extension | Install | Content script | Background worker | Main UI / popup | Critical workflow | Overall |
|---|---|---|---|---|---|---|
| uBlock Origin | Not tested | Not tested | Not tested | Not tested | Not tested | Not tested |
| Dark Reader | Not tested | Not tested | Not tested | Not tested | Not tested | Not tested |
| Bitwarden | Not tested | Not tested | Not tested | Not tested | Not tested | Not tested |
| Tampermonkey | Not tested | Not tested | Not tested | Not tested | Not tested | Not tested |
| SponsorBlock | Not tested | Not tested | Not tested | Not tested | Not tested | Not tested |
| Privacy Badger | Not tested | Not tested | Not tested | Not tested | Not tested | Not tested |

## Acceptance rules

- Record exact extension version, source, manifest version and tested workflow.
- Run tests on the exact CEF build shipped by Nivora; do not extrapolate between CEF/Chromium versions.
- Verify MV3 service-worker wake-up, event delivery and persistence across browser restart.
- Test host permissions and optional permission prompts with allow and deny paths.
- Verify extension UI in normal, maximized, multiple-window and multiple-profile states.
- A test passes only when the user-visible workflow works, not merely when installation succeeds.
- Record failures, unsupported APIs and workarounds. Never convert an unknown result to supported.
