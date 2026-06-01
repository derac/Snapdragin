# Snapdragin

Windows-only Rust app for Snapdragin, a successor to The Griddler.

Status: Windows MVP implemented. Linux and macOS are intentionally out of scope for this repository.

## Build

Requires a Rust toolchain with Cargo.

```powershell
cargo test
cargo build --release
```

The Windows executable is produced at:

```text
target/release/Snapdragin.exe
```

## Current Windows MVP

The Rust Windows app currently supports:

- original The Griddler icon embedded into the executable and tray icon
- tray icon right-click menu with Settings and Exit
- tray icon double-click settings window
- original-style settings UI with monitor grid configuration, visual colors, startup, and usage info
- settings changes apply immediately and persist to `%APPDATA%\Snapdragin\settings.ini`
- global mouse hook
- current drag/right-click activation gesture
- transparent topmost grid overlay
- live window move/resize preview
- final snap on second right-click or left-button release

Files:

- [DESIGN.md](DESIGN.md) - proposed product and technical design
- [SOURCES.md](SOURCES.md) - source links used to ground Windows API decisions
- [src/core](src/core) - OS-free grid, geometry, and selection logic
- [src/windows](src/windows) - Windows desktop app and Win32 FFI
