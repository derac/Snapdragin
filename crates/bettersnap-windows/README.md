# bettersnap-windows

Windows desktop app for Snapdragin'.

This crate is intentionally dependency-free for the first MVP. It uses raw Win32 FFI for:

- tray icon lifecycle
- low-level mouse hook
- active-window discovery
- drag-loop cancellation
- transparent grid overlay
- target window move/resize

## Use

Run `snapdragin.exe`, then:

1. Drag a window with the left mouse button.
2. Right-click while still holding left click to open the grid overlay.
3. Move across the grid to preview the target size.
4. Right-click again or release left click to snap.

Right-click the tray icon to open Settings or exit.
Double-click the tray icon to open Settings directly.

The settings window mirrors the original The Griddler layout: per-monitor rows/columns, visual colors, startup, and usage info. Changes apply immediately and are persisted in `%APPDATA%\Snapdragin\settings.ini`.
