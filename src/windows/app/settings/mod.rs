mod color;
mod controls;
mod paint;
mod startup;
mod storage;
mod window;

pub(super) use color::{colorref_from_hex, is_valid_hex_color, normalize_hex_color, rgba_from_hex};
pub(super) use startup::{set_startup_enabled, startup_is_enabled};
pub(super) use storage::{
    clamp_grid_dimension, load_settings, merge_monitors, save_settings, stored_settings_from_data,
};
pub(super) use window::{
    rebuild_settings_window_if_open, refresh_monitors, show_settings_window, wnd_proc,
};
