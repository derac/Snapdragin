# bettersnap-platform

Shared traits and types for Snapdragin' platform backends.

This crate defines contracts for:

- external window discovery and movement
- monitor discovery
- global input observation
- overlay display
- tray/menu integration
- permission checks
- startup integration

Backend crates should implement these traits with platform-specific APIs such as Win32, macOS Accessibility/AppKit, or Linux X11/EWMH.
