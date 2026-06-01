# Snapdragin' Windows Design

Status: Windows-only implementation
Date: 2026-06-01
Source project: The Griddler

## 1. Scope

Snapdragin' is a lightweight Windows tray utility. It lets the user drag another app's window, activate a grid overlay with right-click, select cells, and snap the target window to the selected screen area.

Linux and macOS are intentionally out of scope for this repository. Any future Linux app should be built separately.

## 2. Project Shape

The crate is intentionally small:

```text
BetterSnap/
  src/
    core/
      OS-free geometry, grid, and selection math

    windows/
      app logic, overlay, settings UI, and Win32 FFI
```

`src/core` exists because the grid math is easy to test without Win32. It is not a separate package or cross-platform backend layer.

## 3. Windows Workflow

1. User drags a window with the left mouse button.
2. User right-clicks while still holding left click.
3. Snapdragin' verifies the target is in the native Windows move/size loop.
4. Snapdragin' cancels that native loop and waits briefly for it to settle.
5. Snapdragin' shows a click-through layered overlay on the active monitor.
6. Pointer movement updates the selected grid cells.
7. Window move/resize requests are queued to the app message loop instead of running inside the low-level mouse hook.
8. Right-click again or release left click to finish the snap.

During a snap, the active monitor is resolved from the current pointer position with `MonitorFromPoint`. The cached monitor list supplies per-monitor grid settings and friendly names, but it does not decide monitor transitions by stale boundary state.

## 4. Win32 Responsibilities

The Windows module owns all OS integration:

- global mouse observation with `SetWindowsHookExW(WH_MOUSE_LL)`
- target-window discovery with `GetForegroundWindow`, `WindowFromPoint`, and `GetAncestor`
- native drag-loop detection with `GetGUIThreadInfo`
- drag cancellation with `WM_CANCELMODE`, non-client/client button-up messages, and capture cleanup
- overlay rendering with a topmost layered window
- target move/resize with `SetWindowPos`
- per-monitor settings stored in `%APPDATA%\Snapdragin\settings.ini`
- tray lifecycle with `Shell_NotifyIconW`

## 5. Current Risk Areas

- Elevated/admin target windows may reject messages from a non-elevated Snapdragin' process.
- Some custom title-bar apps may not behave like normal Win32 windows during drag cancellation.
- Mixed-DPI monitor transitions need manual testing because target apps can handle `WM_DPICHANGED` differently.
- Low-level mouse hooks must stay fast; expensive or blocking work should be posted back to the app window.

## 6. Testing

Automated tests cover the OS-free grid and selection logic. Manual Windows testing should cover:

- normal DPI single monitor
- mixed-DPI multi-monitor snapping
- dragging from one monitor to another
- snapping custom-title-bar apps
- snapping maximized/restored windows
- tray menu and settings persistence
- startup shortcut enable/disable
