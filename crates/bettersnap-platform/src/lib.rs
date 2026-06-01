//! Shared interfaces for Snapdragin' platform backends.
//!
//! Each operating system backend owns the native handles and unsafe calls needed
//! to observe input, show tray/menu UI, draw overlays, and move other windows.
//! This crate defines only the portable contract used by the app layer.

use std::{error::Error, fmt};

use bettersnap_core::{GridSelection, GridSpec, ScreenPoint, ScreenRect};

pub type PlatformResult<T> = Result<T, PlatformError>;
pub type InputEventHandler = Box<dyn FnMut(InputEvent) + Send + 'static>;

/// Opaque identifier for a native window controlled by a backend.
///
/// Backends may map this value to `HWND`, `X11` `Window`, `CGWindowID`, or another
/// stable native identifier. The value is not meaningful outside the backend
/// that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalWindowId(u64);

impl ExternalWindowId {
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonitorId(u64);

impl MonitorId {
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub id: MonitorId,
    pub name: Option<String>,
    pub bounds: ScreenRect,
    /// Scale factor relative to 96 DPI / 1.0 backing scale.
    pub scale_factor: f64,
}

impl MonitorInfo {
    #[must_use]
    pub fn new(id: MonitorId, name: Option<String>, bounds: ScreenRect, scale_factor: f64) -> Self {
        Self {
            id,
            name,
            bounds,
            scale_factor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub id: ExternalWindowId,
    pub title: Option<String>,
    pub frame: ScreenRect,
    pub monitor: Option<MonitorId>,
}

impl WindowInfo {
    #[must_use]
    pub fn new(
        id: ExternalWindowId,
        title: Option<String>,
        frame: ScreenRect,
        monitor: Option<MonitorId>,
    ) -> Self {
        Self {
            id,
            title,
            frame,
            monitor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    PointerMoved {
        position: ScreenPoint,
    },
    PointerButton {
        button: PointerButton,
        state: ButtonState,
        position: ScreenPoint,
    },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKind {
    Accessibility,
    InputMonitoring,
    ScreenRecording,
    Automation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Unknown,
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupState {
    Enabled,
    Disabled,
    Unsupported,
}

pub trait WindowController {
    fn active_window(&self) -> PlatformResult<Option<WindowInfo>>;

    fn window_at(&self, point: ScreenPoint) -> PlatformResult<Option<WindowInfo>>;

    fn window_frame(&self, window: ExternalWindowId) -> PlatformResult<ScreenRect>;

    fn move_resize(&self, window: ExternalWindowId, rect: ScreenRect) -> PlatformResult<()>;

    fn cancel_native_drag(&self, window: ExternalWindowId) -> PlatformResult<()> {
        let _ = window;
        Err(PlatformError::unsupported("cancel_native_drag"))
    }

    fn is_own_window(&self, window: ExternalWindowId) -> bool;
}

pub trait MonitorProvider {
    fn monitors(&self) -> PlatformResult<Vec<MonitorInfo>>;

    fn monitor_at(&self, point: ScreenPoint) -> PlatformResult<Option<MonitorInfo>>;

    fn monitor_for_window(&self, window: ExternalWindowId) -> PlatformResult<Option<MonitorInfo>>;
}

pub trait InputMonitor {
    fn start(&mut self, handler: InputEventHandler) -> PlatformResult<()>;

    fn stop(&mut self) -> PlatformResult<()>;
}

pub trait OverlayController {
    fn show_grid(&mut self, monitor: MonitorInfo, grid: GridSpec) -> PlatformResult<()>;

    fn update_selection(&mut self, selection: GridSelection) -> PlatformResult<()>;

    fn hide(&mut self) -> PlatformResult<()>;
}

pub trait TrayMenu {
    fn set_enabled(&mut self, enabled: bool) -> PlatformResult<()>;

    fn show_settings(&mut self) -> PlatformResult<()>;

    fn quit(&mut self) -> PlatformResult<()>;
}

pub trait PermissionManager {
    fn status(&self, permission: PermissionKind) -> PermissionStatus;

    fn request(&self, permission: PermissionKind) -> PlatformResult<PermissionStatus>;
}

pub trait StartupIntegration {
    fn state(&self) -> PlatformResult<StartupState>;

    fn set_enabled(&self, enabled: bool) -> PlatformResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    Unsupported { feature: &'static str },
    PermissionDenied { permission: &'static str },
    WindowNotFound,
    MonitorNotFound,
    AlreadyRunning { service: &'static str },
    NotRunning { service: &'static str },
    Backend { message: String },
}

impl PlatformError {
    #[must_use]
    pub const fn unsupported(feature: &'static str) -> Self {
        Self::Unsupported { feature }
    }

    #[must_use]
    pub const fn permission_denied(permission: &'static str) -> Self {
        Self::PermissionDenied { permission }
    }

    #[must_use]
    pub fn backend(message: impl Into<String>) -> Self {
        Self::Backend {
            message: message.into(),
        }
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported { feature } => {
                write!(f, "platform feature is unsupported: {feature}")
            }
            Self::PermissionDenied { permission } => {
                write!(f, "platform permission denied: {permission}")
            }
            Self::WindowNotFound => write!(f, "window not found"),
            Self::MonitorNotFound => write!(f, "monitor not found"),
            Self::AlreadyRunning { service } => write!(f, "{service} is already running"),
            Self::NotRunning { service } => write!(f, "{service} is not running"),
            Self::Backend { message } => write!(f, "{message}"),
        }
    }
}

impl Error for PlatformError {}

#[cfg(test)]
mod tests {
    use super::{ExternalWindowId, PlatformError};

    #[test]
    fn window_ids_are_opaque_raw_values() {
        let id = ExternalWindowId::new(42);

        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn platform_errors_are_displayable() {
        let error = PlatformError::unsupported("wayland_global_window_move");

        assert_eq!(
            error.to_string(),
            "platform feature is unsupported: wayland_global_window_move"
        );
    }
}
