# Snapdragin' Source Notes

Date reviewed: 2026-06-01

These sources ground the Windows-only implementation decisions in `DESIGN.md`.

## Current Project

- `TheGriddler.csproj` - confirms the source app is Windows-only (`net10.0-windows`) and uses WPF/Windows Forms.
- `README.md` - describes the tray utility behavior, grid workflow, Windows APIs, and project structure.

## Microsoft Windows APIs

- `SetWindowsHookExW` - low-level hooks for global mouse observation on Windows: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw

- `CallNextHookEx` - hook chain behavior: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-callnexthookex

- `UnhookWindowsHookEx` - hook cleanup: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex

- `GetAsyncKeyState` - physical key/button state checks: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate

- `GetForegroundWindow` - active window lookup: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow

- `WindowFromPoint` - window under cursor lookup: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-windowfrompoint

- `GetAncestor` - top-level/root window discovery: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getancestor

- `GetGUIThreadInfo` - native move/size loop state such as `GUI_INMOVESIZE`, `hwndMoveSize`, and `hwndCapture`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getguithreadinfo

- `SendMessageTimeoutW` - bounded synchronous messages used while cancelling native drag loops: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw

- `PostMessageW` - async app-window work queue for snap application: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew

- `ReleaseCapture` - release mouse capture owned by the calling thread; target windows are still cancelled through messages: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-releasecapture

- `SetWindowPos` - target window move/resize operation: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos

- `GetDpiForWindow` - per-window DPI: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow

- `GetDpiForMonitor` - monitor DPI: https://learn.microsoft.com/en-us/windows/win32/api/shellscalingapi/nf-shellscalingapi-getdpiformonitor

- `UpdateLayeredWindow` - transparent overlay rendering: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-updatelayeredwindow

- `Shell_NotifyIconW` - tray icon lifecycle: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw

## Important Source Interpretation

- Windows exposes the required global input, foreign-window movement, layered overlay, and tray primitives directly enough for a focused native utility.
- Snapdragin' should keep expensive work out of the low-level mouse hook and post it to the app window instead.
- Native drag cancellation should address the active move/size window and capture window, not only the root target window.
