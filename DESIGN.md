# Snapdragin' Rust Rewrite Design

Status: approved for initial implementation
Date: 2026-05-28
Source project: The Griddler

## 1. Purpose

Snapdragin' is a proposed Rust rewrite of The Griddler. The current app is a lightweight Windows tray utility that lets the user resize and reposition another app's window by dragging the target window, activating a grid overlay, selecting grid cells, and applying the resulting bounds.

This document is not an implementation plan approval. It is a design proposal to decide whether a Rust rewrite is worth pursuing, what platform support means in practice, and where the OS-specific risks are.

## 2. Current App Summary

The current project is Windows-specific:

- Target framework: `net10.0-windows`
- UI frameworks: WPF and Windows Forms
- App type: tray/taskbar utility with a small settings dialog
- Core behavior: global mouse hook, target window discovery, drag interruption, transparent grid overlay, and Win32 window move/resize

The current README describes the user flow:

1. User drags a window with the left mouse button.
2. User right-clicks the starting grid cell.
3. App interrupts the native drag and displays the grid overlay.
4. User moves to the ending grid cell.
5. User right-clicks again or releases left click.
6. App resizes and moves the target window to the selected grid area.

## 3. Goals

- Preserve the core interaction model where feasible.
- Keep the app lightweight and tray/menu-bar focused.
- Share grid, monitor, settings, and state-machine logic across platforms.
- Support Windows as the first-class platform.
- Support Linux X11 if prototypes confirm reliable window control.
- Support macOS if Accessibility and input permissions are acceptable.
- Clearly document Wayland limitations instead of hiding them behind unreliable behavior.
- Avoid a heavy rendering or game engine; only simple overlay drawing is required.

## 4. Non-Goals

- No full desktop environment replacement.
- No tiling window manager.
- No general Wayland compositor plugin in the first version.
- No single abstraction pretending all OSes expose the same window-control model.
- No web stack unless a future UI decision makes that explicitly worthwhile.

## 5. Key Product Decisions Needed

Before implementation, these decisions need sign-off:

1. Is Windows parity the first milestone?
2. Is Linux X11 support enough for "Linux support" in the first release?
3. Should Wayland be documented as unsupported/limited unless compositor-specific integrations are added?
4. Should macOS use a slightly different workflow if exact mouse-drag interruption is unreliable?
5. Should the app keep the exact current gesture, or also add a global-hotkey mode as a more portable fallback?

Recommended initial answer:

- Windows parity: yes.
- Linux first target: X11.
- Wayland: documented limitation, no generic support claim.
- macOS: supported after prototype, with Accessibility permission requirement.
- Add a hotkey fallback: yes, because it gives a less fragile interaction path across platforms.

## 6. Proposed Architecture

Use a Cargo workspace with a strict split between shared logic and OS backends.

```text
BetterSnap/
  crates/
    bettersnap-core/
      grid math
      selection state machine
      monitor model
      settings model
      command/event types

    bettersnap-app/
      app orchestration
      tray/menu commands
      settings persistence
      platform-independent workflow

    bettersnap-platform/
      platform traits
      shared platform types

    bettersnap-windows/
      Win32 input hooks
      active window discovery
      move/resize
      overlay implementation
      tray integration if not shared

    bettersnap-macos/
      Accessibility API window control
      CGEvent tap/global input
      menu bar integration
      overlay implementation

    bettersnap-linux-x11/
      X11/XCB/EWMH window control
      global mouse/input handling
      overlay implementation
      tray integration

    bettersnap-desktop/
      binary entry point
      dependency wiring
```

The core crate should have no OS calls. It should be testable with normal unit tests.

## 7. Core Interfaces

The OS-specific code should sit behind explicit traits.

```rust
pub trait WindowController {
    fn active_window(&self) -> Result<Option<ExternalWindowId>>;
    fn window_at(&self, point: ScreenPoint) -> Result<Option<ExternalWindowId>>;
    fn window_frame(&self, window: ExternalWindowId) -> Result<ScreenRect>;
    fn move_resize(&self, window: ExternalWindowId, rect: ScreenRect) -> Result<()>;
    fn cancel_native_drag(&self, window: ExternalWindowId) -> Result<()>;
    fn is_own_window(&self, window: ExternalWindowId) -> bool;
}

pub trait InputMonitor {
    fn start(&mut self, sink: InputEventSink) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

pub trait Overlay {
    fn show(&mut self, monitor: MonitorInfo, grid: GridSpec) -> Result<()>;
    fn update_selection(&mut self, selection: GridSelection) -> Result<()>;
    fn hide(&mut self) -> Result<()>;
}

pub trait TrayMenu {
    fn set_enabled(&mut self, enabled: bool) -> Result<()>;
    fn show_settings(&mut self) -> Result<()>;
    fn quit(&mut self) -> Result<()>;
}
```

The trait names are illustrative. The important design point is that global input, window movement, overlay behavior, and tray/menu behavior are not treated as portable by default.

## 8. Interaction Model

### 8.1 Exact Current Gesture

Target behavior:

1. Observe global mouse input.
2. Track left button down and cursor movement.
3. On right-click while left is held, identify the target window.
4. If the target is in a native move/size loop, interrupt that loop.
5. Display a transparent grid overlay on the relevant monitor.
6. Convert pointer position to grid cells.
7. Preview the selected rectangle.
8. On second right-click or left-button release, compute final bounds.
9. Move/resize the target window.

This is the highest-parity workflow, but it depends on deep OS behavior. It is most realistic on Windows and X11.

### 8.2 Hotkey Fallback

Add an alternate mode:

1. User focuses a window.
2. User presses a global hotkey.
3. Grid overlay appears for the active window's monitor.
4. User selects cells with normal mouse input inside the overlay.
5. App moves/resizes the active window.

This mode avoids interrupting a native mouse drag. It still requires permission to move another app's window, but it is less fragile than the current gesture on macOS and Linux.

Recommendation: implement the hotkey mode even if exact gesture parity remains the default.

## 9. Overlay Design

The grid overlay is the only real rendering requirement. It should be a borderless, transparent, always-on-top window that draws:

- grid lines
- selected cells
- optional target preview rectangle
- optional monitor label or dimensions if needed for debugging

It should not own the main product flow. It is a temporary visual surface.

Platform notes:

- Windows: topmost layered window, optionally click-through when appropriate.
- macOS: borderless transparent `NSWindow` or `NSPanel`, with mouse ignoring when appropriate.
- Linux X11: ARGB visual plus window-manager hints; input shape can make it click-through if needed.
- Linux Wayland: overlay creation may work, but global positioning and layering are compositor-dependent.

## 10. Platform Feasibility

### 10.1 Windows

Feasibility: high.

Expected implementation:

- Global mouse hook: `SetWindowsHookEx` with `WH_MOUSE_LL`
- Mouse state: `GetAsyncKeyState` as needed
- Target window: `WindowFromPoint`, `GetAncestor`, `GetForegroundWindow`
- Native drag detection: `GetGUIThreadInfo`
- Drag interruption: `ReleaseCapture`, `SendMessage` with `WM_CANCELMODE` and/or button-up behavior
- Move/resize: `SetWindowPos`
- Accurate bounds: `GetWindowRect` plus DWM extended frame bounds where needed
- DPI: `GetDpiForWindow`, monitor DPI APIs, per-monitor awareness
- Tray: Rust tray crate or direct `Shell_NotifyIcon`

Risks:

- Elevated/admin windows may reject control from a non-elevated Snapdragin' process.
- Some apps with custom title bars may behave differently during drag interruption.
- Multi-monitor DPI handling must be tested carefully.

### 10.2 macOS

Feasibility: medium.

Expected implementation:

- Window move/resize through Accessibility APIs.
- The user must grant Accessibility permission.
- Global input monitoring may require Input Monitoring permission.
- Menu-bar app via native status item behavior.
- Overlay via transparent borderless AppKit window.

Risks:

- Exact drag interruption may not map cleanly to the current Windows behavior.
- Permission prompts and failure states must be clear.
- Some apps may not expose movable/resizable Accessibility windows.
- Distribution likely requires signing and notarization for a smooth user experience.

Recommendation:

- Prototype active-window move/resize first.
- Prototype exact current gesture second.
- Keep hotkey mode as the reliable macOS workflow if drag interruption is not robust.

### 10.3 Linux X11

Feasibility: medium to high, depending on window manager.

Expected implementation:

- Global input through X11/XInput2 or a Rust wrapper.
- Target window discovery through X11 tree queries and EWMH properties.
- Move/resize through EWMH requests or direct X11 configure calls depending on the window manager.
- Overlay through an ARGB top-level window with window-manager hints.
- Tray through StatusNotifier/AppIndicator where available.

Risks:

- Behavior differs across GNOME Xorg, KDE Plasma X11, XFCE, i3, Openbox, and other window managers.
- Some window managers may ignore or reinterpret move/resize requests.
- Interrupting native drag behavior may be less consistent than on Windows.
- Linux tray support is fragmented.

Recommendation:

- Treat Linux X11 as a supported target only after testing a small matrix of window managers.
- Prefer a hotkey fallback even if drag gesture works on common desktops.

### 10.4 Linux Wayland

Feasibility: low for generic support.

Wayland intentionally avoids a global window-management model where normal clients can enumerate, position, and move arbitrary windows owned by other apps. The compositor owns that authority.

Possible limited paths:

- Support XWayland windows through X11 APIs where available.
- Add compositor-specific integrations:
  - KDE KWin scripts or D-Bus integration
  - GNOME Shell extension
  - Sway IPC
  - Hyprland IPC
  - other compositor-specific APIs
- Document Wayland as unsupported for full Snapdragin' behavior in the first release.

Recommendation:

- Do not claim general Wayland support in v1.
- If Linux is a priority, state "Linux X11 supported; Wayland limited/unsupported unless listed compositor integration is installed."

## 11. Dependency Direction

The final dependency list should be decided after prototypes, but this is the likely direction.

Shared:

- `serde` for settings serialization.
- `directories` or similar for config paths.
- `thiserror` or `anyhow` for error handling.
- `tracing` for diagnostics.

UI/tray:

- `tray-icon` for tray/menu support where it fits.
- `muda` if native menu construction is needed.
- A small UI toolkit for settings, likely `egui/eframe` or Slint.

Platform:

- Windows: `windows` crate for Win32 APIs.
- macOS: AppKit, CoreGraphics, and ApplicationServices bindings.
- Linux X11: `x11rb`, `xcb`, or focused X11/EWMH wrappers after prototype.

Overlay:

- Native transparent windows plus simple drawing.
- Avoid `wgpu`, Bevy, or a game engine unless the overlay requirements grow substantially.

## 12. Settings

Initial settings should map closely to The Griddler:

- grid rows per monitor
- grid columns per monitor
- overlay colors
- selection colors
- trigger mode:
  - current drag/right-click gesture
  - hotkey fallback
  - both
- startup behavior
- per-platform permission/status page

Use a portable config format such as JSON or TOML.

Recommended config locations should use OS conventions:

- Windows: `%APPDATA%\Snapdragin`
- macOS: `~/Library/Application Support/Snapdragin`
- Linux: `$XDG_CONFIG_HOME/bettersnap` or `~/.config/bettersnap`

## 13. Packaging

Windows:

- `.exe` for development.
- MSI or installer later if startup integration and auto-update are added.

macOS:

- `.app` bundle.
- Code signing and notarization for public distribution.
- Permission explanations for Accessibility and Input Monitoring.

Linux:

- AppImage for broad manual distribution.
- `.deb`/`.rpm` later if desired.
- Document X11 requirement for full functionality.
- Document any tray dependencies, such as AppIndicator/Ayatana libraries, if the chosen tray crate requires them.

## 14. Testing Strategy

Core tests:

- grid coordinate mapping
- monitor bounds and negative-coordinate monitors
- DPI-independent selection math
- selection normalization from any drag direction
- settings validation and defaults

Windows integration tests/manual tests:

- normal DPI
- mixed DPI multi-monitor
- maximized window restore/snap behavior
- elevated target windows
- custom title bar apps
- own-window exclusion
- tray lifecycle

macOS manual tests:

- permission denied flow
- permission granted flow
- active-window move/resize
- menu-bar lifecycle
- overlay on multiple monitors
- apps that do not expose normal Accessibility windows

Linux X11 manual tests:

- KDE Plasma X11
- GNOME on Xorg
- XFCE
- i3 or another non-floating-first window manager
- AppIndicator/tray behavior
- XWayland distinction if testing under Wayland sessions

Wayland tests:

- confirm unsupported behavior is detected cleanly
- verify the app does not silently appear to work while failing to move windows
- document compositor-specific experiments separately

## 15. Risks

Highest-risk items:

1. Linux Wayland generic support is not realistic for the core feature.
2. macOS requires Accessibility and possibly Input Monitoring permissions.
3. Exact drag interruption may not be portable.
4. Linux X11 behavior varies by window manager.
5. Tray behavior is not uniform across Linux desktops.
6. Transparent overlay behavior differs per platform.
7. Mixed-DPI multi-monitor snapping needs careful math and testing.

Risk-reduction order:

1. Prototype Windows parity.
2. Prototype macOS active-window move/resize with permissions.
3. Prototype Linux X11 move/resize on KDE/GNOME/XFCE.
4. Decide whether exact gesture parity is required outside Windows.
5. Only then choose final UI/tray crates.

## 16. Suggested Milestones

### Milestone 0: Feasibility Spikes

- Windows: prove Rust can reproduce current gesture and snapping.
- macOS: prove active-window move/resize with Accessibility.
- Linux X11: prove active-window move/resize and overlay behavior.
- Wayland: prove limitation detection and messaging.

Exit criteria:

- We know which OSes can support exact gesture parity.
- We know which fallback workflows are needed.
- We have a dependency shortlist based on working prototypes.

### Milestone 1: Shared Core

- Implement grid math.
- Implement selection state machine.
- Implement settings model.
- Implement monitor and rectangle types.
- Add unit tests.

Exit criteria:

- Core logic works without platform code.
- Backend APIs are defined from prototype evidence.

### Milestone 2: Windows MVP

- Tray/menu app.
- Settings dialog.
- Exact drag/right-click gesture.
- Overlay preview.
- Move/resize target window.
- Startup option.

Exit criteria:

- Feature parity with the current Windows app.

### Milestone 3: Linux X11 MVP

- X11 active-window and window-at-point support.
- X11 move/resize.
- Overlay.
- Tray/menu.
- Hotkey fallback if exact gesture is unreliable.

Exit criteria:

- Works on the approved X11 desktop test matrix.

### Milestone 4: macOS MVP

- Menu-bar app.
- Accessibility permission detection.
- Active-window move/resize.
- Overlay.
- Hotkey fallback.
- Exact gesture only if prototype proves it reliable.

Exit criteria:

- Works with documented permissions and clear failure states.

## 17. Recommended Decision

Rust is a reasonable rewrite target if the goal is a lean native utility with explicit platform backends.

The important expectation is that Snapdragin' would not be a single generic cross-platform app internally. It would be a shared Rust core plus separate OS integrations. Windows is the best first target. Linux support should initially mean X11. macOS can work if the permission model is acceptable. Wayland should be treated as limited unless the project commits to compositor-specific integrations.

Recommended sign-off scope:

- Approve a design/prototype phase, not a full rewrite.
- Prototype Windows parity first.
- Prototype macOS and Linux X11 before promising release support.
- Add hotkey mode as a portable fallback.
- Do not promise generic Wayland support in v1.
