# Snapdragin' Source Notes

Date reviewed: 2026-05-28

These sources ground the design assumptions in `DESIGN.md`. They are not an exhaustive implementation reference.

## Current Project

- `TheGriddler.csproj` - confirms `net10.0-windows`, WPF, Windows Forms, and Windows-only app shape.
- `README.md` - describes current tray utility behavior, grid workflow, Windows APIs, and project structure.

## Microsoft Windows APIs

- `SetWindowsHookExW` - low-level hooks for global mouse observation on Windows: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw

- `CallNextHookEx` - hook chain behavior: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-callnexthookex

- `UnhookWindowsHookEx` - hook cleanup: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex

- `GetAsyncKeyState` - physical key/button state checks: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate

- `GetForegroundWindow` - active window lookup: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow

- `WindowFromPoint` - window under cursor lookup: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-windowfrompoint

- `GetAncestor` - top-level/root window discovery: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getancestor

- `GetWindowRect` - window bounds: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect

- `SetWindowPos` - window move/resize operation: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos

- `GetGUIThreadInfo` - native move/size loop state such as `GUI_INMOVESIZE`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getguithreadinfo

- `SendMessage` - message sending for drag interruption behavior: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessage

- `ReleaseCapture` - release mouse capture: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasecapture

- `DwmGetWindowAttribute` - extended frame bounds and DWM attributes: https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmgetwindowattribute

- `GetDpiForWindow` - per-window DPI: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow

- `GetDpiForMonitor` - monitor DPI: https://learn.microsoft.com/en-us/windows/win32/api/shellscalingapi/nf-shellscalingapi-getdpiformonitor

- `RegisterHotKey` - global hotkey registration if Snapdragin' adds a fallback mode: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey

## macOS APIs and Permissions

- Apple user guide for Input Monitoring permission: https://support.apple.com/guide/mac-help/control-access-to-input-monitoring-on-mac-mchl4cedafb6/mac

- `AXUIElementCreateApplication` - create Accessibility element for another app: https://developer.apple.com/documentation/applicationservices/1459134-axuielementcreateapplication

- `AXUIElementSetAttributeValue` - set Accessibility attributes such as window position/size where permitted: https://developer.apple.com/documentation/applicationservices/1462091-axuielementsetattributevalue

- `AXIsProcessTrustedWithOptions` - check/request Accessibility trust: https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions

- `NSStatusItem` - menu-bar/status-item style app surface: https://developer.apple.com/documentation/appkit/nsstatusitem

## Linux and Wayland

- Wayland book, `xdg_toplevel` interactive move/resize. Useful for understanding compositor-owned interactive movement: https://wayland-book.com/xdg-shell-in-depth/interactive.html

- Wayland protocol repository. Useful for confirming protocol-specific capabilities and limits: https://gitlab.freedesktop.org/wayland/wayland-protocols

- X11 protocol Rust crate `x11rb`. Candidate library for X11 integration: https://docs.rs/x11rb/latest/x11rb/

- XDG base directory specification. Useful for Linux config path decisions: https://specifications.freedesktop.org/basedir-spec/latest/

- StatusNotifierItem specification. Relevant to Linux tray/menu support: https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/

## Rust Crates to Evaluate

- `windows` - generated Rust bindings for Win32 and other Windows APIs: https://docs.rs/windows/latest/windows/

- `tray-icon` - cross-platform tray icon crate candidate: https://docs.rs/tray-icon/latest/tray_icon/

- `muda` - menu crate commonly used with tray/windowing crates: https://docs.rs/muda/latest/muda/

- `global-hotkey` - global hotkey crate candidate. Its Linux backend should be evaluated carefully, especially around X11 versus Wayland: https://docs.rs/global-hotkey/latest/global_hotkey/

- `rdev` - global input listen/simulate crate candidate. Must be prototyped before depending on it for exact gesture parity: https://docs.rs/rdev/latest/rdev/

- `x11rb` - Rust X11 protocol implementation: https://docs.rs/x11rb/latest/x11rb/

- `egui` / `eframe` - possible lightweight settings UI and overlay tooling: https://github.com/emilk/egui

- Slint - possible declarative settings UI toolkit with Rust support: https://slint.dev/

## Important Source Interpretation

- Windows exposes the required primitives directly enough for parity to be a realistic goal.
- macOS exposes window movement through Accessibility, but that implies user permission and possible app-specific limitations.
- X11 exposes global window/input concepts, but behavior varies by window manager.
- Wayland does not provide a compositor-independent way for a normal client to manage arbitrary windows owned by other apps. Snapdragin' should not promise generic Wayland support without compositor-specific work.
