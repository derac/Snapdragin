use super::*;

#[derive(Debug, Clone)]
pub(super) struct MonitorConfig {
    pub(super) device_name: String,
    pub(super) display_name: String,
    pub(super) monitor_rect: ScreenRect,
    pub(super) work_rect: ScreenRect,
    pub(super) columns: u16,
    pub(super) rows: u16,
}

#[derive(Debug, Clone)]
pub(super) struct SettingsData {
    pub(super) monitors: Vec<MonitorConfig>,
    pub(super) grid_color: String,
    pub(super) selection_color: String,
    pub(super) selection_border_color: String,
    pub(super) run_on_startup: bool,
    pub(super) is_dark_mode: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MonitorEdit {
    pub(super) columns_edit: Hwnd,
    pub(super) rows_edit: Hwnd,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingSnap {
    pub(super) target: Hwnd,
    pub(super) rect: ScreenRect,
}

#[derive(Debug, Clone)]
pub(super) struct StoredMonitorConfig {
    pub(super) device_name: String,
    pub(super) display_name: String,
    pub(super) columns: u16,
    pub(super) rows: u16,
}

#[derive(Debug, Clone)]
pub(super) struct StoredSettings {
    pub(super) monitors: Vec<StoredMonitorConfig>,
    pub(super) grid_color: String,
    pub(super) selection_color: String,
    pub(super) selection_border_color: String,
    pub(super) run_on_startup: Option<bool>,
    pub(super) is_dark_mode: Option<bool>,
}

#[derive(Debug)]
pub(super) struct AppState {
    pub(super) main_hwnd: Hwnd,
    pub(super) settings_hwnd: Hwnd,
    pub(super) monitor_edits: Vec<MonitorEdit>,
    pub(super) grid_color_edit: Hwnd,
    pub(super) selection_color_edit: Hwnd,
    pub(super) selection_border_color_edit: Hwnd,
    pub(super) run_startup_checkbox: Hwnd,
    pub(super) syncing_settings_ui: bool,
    pub(super) settings_dpi: u32,
    pub(super) overlay_hwnd: Hwnd,
    pub(super) hook: Hhook,
    pub(super) app_icon: Hicon,
    pub(super) left_button_down: bool,
    pub(super) dragging: bool,
    pub(super) suppress_right_up: bool,
    pub(super) target_hwnd: Hwnd,
    pub(super) grid: GridSpec,
    pub(super) settings: SettingsData,
    pub(super) monitor_device_name: String,
    pub(super) monitor_rect: ScreenRect,
    pub(super) tracker: Option<SelectionTracker>,
    pub(super) selection: Option<GridSelection>,
    pub(super) queued_snap: Option<PendingSnap>,
    pub(super) snap_apply_pending: bool,
    pub(super) settle_snap: Option<PendingSnap>,
}

impl AppState {
    pub(super) fn new(main_hwnd: Hwnd, app_icon: Hicon) -> Self {
        let settings = settings::load_settings();
        let grid = settings
            .monitors
            .first()
            .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
            .unwrap_or_else(default_grid);

        Self {
            main_hwnd,
            settings_hwnd: 0,
            monitor_edits: Vec::new(),
            grid_color_edit: 0,
            selection_color_edit: 0,
            selection_border_color_edit: 0,
            run_startup_checkbox: 0,
            syncing_settings_ui: false,
            settings_dpi: 96,
            overlay_hwnd: 0,
            hook: 0,
            app_icon,
            left_button_down: desktop::left_button_is_down(),
            dragging: false,
            suppress_right_up: false,
            target_hwnd: 0,
            grid,
            settings,
            monitor_device_name: String::new(),
            monitor_rect: ScreenRect::new(0, 0, 1, 1),
            tracker: None,
            selection: None,
            queued_snap: None,
            snap_apply_pending: false,
            settle_snap: None,
        }
    }

    pub(super) fn clear_drag(&mut self) {
        self.dragging = false;
        self.target_hwnd = 0;
        self.tracker = None;
        self.selection = None;
    }
}

#[derive(Clone, Copy)]
pub(super) struct RgbaColor {
    pub(super) red: u8,
    pub(super) green: u8,
    pub(super) blue: u8,
    pub(super) alpha: u8,
}
