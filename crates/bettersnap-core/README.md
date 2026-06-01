# bettersnap-core

Shared, platform-independent logic for Snapdragin'.

This crate owns:

- physical screen geometry types
- grid validation
- point-to-cell mapping
- cell-to-rectangle mapping
- selection normalization
- selection tracking for overlay previews

This crate must not call OS APIs. Windows, macOS, X11, and future Wayland-specific code should translate native data into these types.
