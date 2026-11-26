# Nivora UI system

This is the first native Slint chrome pass. It deliberately keeps the browser surface dominant and leaves page rendering to the existing WinCairo child HWND.

## Tokens

`ui/main.slint` defines a shared `Palette` global with Dark and Light color tokens for:

- Window and chrome surfaces
- Address field and browser page surfaces
- Borders, primary/secondary/muted text
- Hover, pressed, selected, and accent colors
- Text selection

The palette is switched from the Quick Settings popover. It currently offers explicit Dark and Light modes. Automatic Windows System appearance detection is not implemented yet and is not presented as a working option.

## Reusable controls

- `IconButton`: compact navigation and toolbar control with hover, pressed, selected, and disabled visuals.
- `TextButton`: compact action used by the settings popover.
- `TabItem`: active/inactive tab treatment.
- `AddressField`: custom TextInput inside a tokenized rounded field, with focus feedback and loading state.
- `MainWindow`: top tab strip, compact navigation row, optional vertical tab rail, quick settings, and native browser geometry anchor.

## Interaction behavior

- Back and Forward buttons are disabled when the engine reports no history entry.
- Reload, address submission, and navigation remain connected to the existing Rust callbacks.
- The tab-placement control switches between the horizontal active-tab strip and a vertical active-tab rail. The current engine still owns one WebKit view; this layout toggle does not claim multi-tab engine support.
- The appearance control switches between the implemented Dark and Light palettes.
- Quick Settings is kept entirely within the 92-DIP chrome region. It does not overlap the WebKit child HWND, which would otherwise cover Slint-rendered overlays due to native child-window z-order.

## Motion and accessibility

Small control surfaces use 120 ms color transitions; the quick-settings surface uses a 160 ms opacity transition. Slint 1.19 respects Windows reduced-motion preferences by skipping animations when configured by the operating system. The address input retains Slint's native TextInput keyboard and accessibility behavior.

## Known limits

- Multi-tab WebKit view ownership, drag-and-drop tab reordering, favicons, suggestions, security/TLS indicators, and real byte-based loading progress are not implemented.
- A native child HWND is not clipped to Slint rounded corners. The WebKit surface therefore uses square edges rather than pretending to provide unsupported clipping.
- Visual QA at Windows DPI scales and a running WinCairo build still requires a Windows machine with the matching WebKit SDK/runtime.
