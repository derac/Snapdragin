[demo](https://github.com/user-attachments/assets/ce02675b-2678-4f79-bab1-e725f6bfb9ff)

<img width="700" height="500" alt="settings" src="https://github.com/user-attachments/assets/396befcb-8bd0-4513-b36b-7eadd3e6acf2" />


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
target/release/BetterSnap.exe
```

Files:

- [DESIGN.md](DESIGN.md) - proposed product and technical design
- [SOURCES.md](SOURCES.md) - source links used to ground Windows API decisions
- [src/core](src/core) - OS-free grid, geometry, and selection logic
- [src/windows](src/windows) - Windows desktop app and Win32 FFI
