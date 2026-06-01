# Snapdragin'

Rust rewrite workspace for Snapdragin', a possible successor to The Griddler.

Status: Windows MVP implemented. The shared core and Windows desktop app compile; macOS and Linux backends are still design/prototype work.

## Build

Requires a Rust toolchain with Cargo.

```powershell
cargo test
cargo build --release --workspace
```

The Windows executable is produced at:

```text
target/release/snapdragin.exe
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
- [SOURCES.md](SOURCES.md) - source links used to ground platform assumptions
- [crates/bettersnap-core](crates/bettersnap-core) - shared grid, geometry, and selection logic
- [crates/bettersnap-platform](crates/bettersnap-platform) - shared traits for OS-specific backends
- [crates/bettersnap-windows](crates/bettersnap-windows) - Windows desktop app
