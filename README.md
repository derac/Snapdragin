[demo](https://github.com/user-attachments/assets/ce02675b-2678-4f79-bab1-e725f6bfb9ff)

<img width="700" height="500" alt="Settings menu" src="https://github.com/user-attachments/assets/0b0d2f1f-cd09-40b9-b9bc-04b47c521cec" />

# Snapdragin

Windows-only Rust app. Resize windows into tiles with multi monitor and dpi support. Use the mouse to drag and snap windows into tiles.

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

Files:

- [DESIGN.md](DESIGN.md) - proposed product and technical design
- [SOURCES.md](SOURCES.md) - source links used to ground Windows API decisions
- [src/core](src/core) - OS-free grid, geometry, and selection logic
- [src/windows](src/windows) - Windows-only integration boundary
- [src/windows/app](src/windows/app) - tray app runtime, drag tracking, snapping, overlay, and settings UI
- [src/windows/app/settings](src/windows/app/settings) - settings persistence, controls, painting, and startup shortcut handling
- [src/windows/ffi](src/windows/ffi) - raw Win32 and COM declarations used by the app
