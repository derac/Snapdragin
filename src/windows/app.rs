use std::{
    env,
    ffi::c_void,
    fs,
    mem::{size_of, zeroed},
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};

use crate::core::{GridSelection, GridSpec, ScreenPoint, ScreenRect, SelectionTracker};

use super::win32::*;

mod desktop;
mod overlay;
mod settings_store;
mod settings_window;
mod tray;

const APP_NAME: &str = "Snapdragin";
const APP_DIR_NAME: &str = "Snapdragin";
const STARTUP_SHORTCUT_NAME: &str = "Snapdragin.lnk";
const LEGACY_STARTUP_SCRIPT_NAME: &str = "Snapdragin.cmd";
const MAIN_CLASS: &str = "SnapdraginMainWindow";
const OVERLAY_CLASS: &str = "SnapdraginOverlayWindow";
const SETTINGS_CLASS: &str = "SnapdraginSettingsWindow";
const APP_ICON_ID: usize = 101;

const ID_TRAY: u32 = 1;
const WM_TRAYICON: u32 = WM_USER + 1;
const WM_APPLY_SNAP: u32 = WM_USER + 2;

const ID_SETTINGS: usize = 1080;
const ID_ABOUT: usize = 1090;
const ID_EXIT: usize = 1099;

const ID_RESET_COLORS: usize = 2005;
const ID_RUN_STARTUP: usize = 2006;
const ID_GRID_COLOR_EDIT: usize = 2007;
const ID_SELECTION_COLOR_EDIT: usize = 2008;
const ID_SELECTION_BORDER_EDIT: usize = 2009;
const ID_MONITOR_EDIT_BASE: usize = 3000;

const WH_MOUSE_LL: i32 = 14;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_PAINT: u32 = 0x000F;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_COMMAND: u32 = 0x0111;
const WM_TIMER: u32 = 0x0113;
const WM_CONTEXTMENU: u32 = 0x007B;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_DISPLAYCHANGE: u32 = 0x007E;
const WM_DPICHANGED: u32 = 0x02E0;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_CANCELMODE: u32 = 0x001F;
const WM_USER: u32 = 0x0400;

const VK_LBUTTON: i32 = 0x01;

const GA_ROOT: u32 = 2;
const GUI_INMOVESIZE: u32 = 0x0000_0002;

const MONITOR_DEFAULTTONEAREST: u32 = 2;

const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
const WS_POPUP: u32 = 0x8000_0000;
const WS_CHILD: u32 = 0x4000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_BORDER: u32 = 0x0080_0000;
const WS_TABSTOP: u32 = 0x0001_0000;
const ES_AUTOHSCROLL: u32 = 0x0000_0080;
const WS_EX_LAYERED: u32 = 0x0008_0000;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_TOPMOST: u32 = 0x0000_0008;

const CS_VREDRAW: u32 = 0x0001;
const CS_HREDRAW: u32 = 0x0002;
const CS_DBLCLKS: u32 = 0x0008;

const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;
const SW_SHOWNOACTIVATE: i32 = 4;
const CW_USEDEFAULT: i32 = 0x8000_0000_u32 as i32;

const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_SHOWWINDOW: u32 = 0x0040;

const HWND_TOPMOST: Hwnd = -1;

const ULW_ALPHA: u32 = 0x0000_0002;
const AC_SRC_OVER: u8 = 0;
const AC_SRC_ALPHA: u8 = 1;
const BI_RGB: u32 = 0;
const DIB_RGB_COLORS: u32 = 0;

const SMTO_ABORTIFHUNG: u32 = 0x0002;
const SMTO_ERRORONEXIT: u32 = 0x0020;
const RDW_INVALIDATE: u32 = 0x0001;
const RDW_ERASE: u32 = 0x0004;
const RDW_ALLCHILDREN: u32 = 0x0080;
const RDW_UPDATENOW: u32 = 0x0100;
const RDW_FRAME: u32 = 0x0400;
const DRAG_CANCEL_TIMEOUT_MS: u32 = 20;
const DRAG_CANCEL_SETTLE_ATTEMPTS: usize = 8;
const DRAG_CANCEL_SETTLE_DELAY_MS: u64 = 1;
const SNAP_SETTLE_ATTEMPTS: usize = 3;
const SNAP_SETTLE_DELAY_MS: u64 = 2;
const SNAP_SETTLE_TIMER_ID: usize = 20_101;
const SNAP_SETTLE_TIMER_MS: u32 = 50;

const NIM_ADD: u32 = 0x0000_0000;
const NIM_DELETE: u32 = 0x0000_0002;
const NIF_MESSAGE: u32 = 0x0000_0001;
const NIF_ICON: u32 = 0x0000_0002;
const NIF_TIP: u32 = 0x0000_0004;

const MF_STRING: u32 = 0x0000_0000;
const MF_SEPARATOR: u32 = 0x0000_0800;
const MF_CHECKED: u32 = 0x0000_0008;

const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_RETURNCMD: u32 = 0x0100;
const TPM_NONOTIFY: u32 = 0x0080;

const PS_SOLID: i32 = 0;
const COLOR_WINDOW: i32 = 5;
const ES_NUMBER: u32 = 0x0000_2000;
const BS_PUSHBUTTON: u32 = 0x0000_0000;
const BS_AUTOCHECKBOX: u32 = 0x0000_0003;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BST_CHECKED: usize = 1;
const BN_CLICKED: usize = 0;
const EN_CHANGE: usize = 0x0300;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;
const WM_NCLBUTTONUP: u32 = 0x00A2;
const HTCAPTION: usize = 2;

const MB_OK: u32 = 0x0000_0000;
const MB_ICONINFORMATION: u32 = 0x0000_0040;
const DT_LEFT: u32 = 0x0000_0000;
const DT_TOP: u32 = 0x0000_0000;
const DT_WORDBREAK: u32 = 0x0000_0010;
const TRANSPARENT_BK: i32 = 1;
const DI_NORMAL: u32 = 0x0003;
const CC_RGBINIT: u32 = 0x0000_0001;
const CC_FULLOPEN: u32 = 0x0000_0002;

const IDI_APPLICATION: usize = 32_512;
const IDC_ARROW: usize = 32_512;

const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;
const MIN_GRID_DIMENSION: u16 = 1;
const MAX_GRID_DIMENSION: u16 = 20;
const DEFAULT_GRID_COLOR: &str = "#FFFFFF11";
const DEFAULT_SELECTION_COLOR: &str = "#00FFFF11";
const DEFAULT_SELECTION_BORDER_COLOR: &str = "#00FFFF44";
const PREVIOUS_DEFAULT_GRID_COLOR: &str = "#FFFFFF22";
const PREVIOUS_DEFAULT_SELECTION_COLOR: &str = "#00FFFF22";
const PREVIOUS_DEFAULT_SELECTION_BORDER_COLOR: &str = "#00FFFF88";
const LEGACY_DEFAULT_GRID_COLOR: &str = "#22FFFFFF";
const LEGACY_DEFAULT_SELECTION_COLOR: &str = "#2200FFFF";
const LEGACY_DEFAULT_SELECTION_BORDER_COLOR: &str = "#8800FFFF";
const SETTINGS_WIDTH: i32 = 700;
const SETTINGS_HEIGHT: i32 = 500;

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

#[derive(Debug, Clone)]
struct MonitorConfig {
    device_name: String,
    display_name: String,
    monitor_rect: ScreenRect,
    work_rect: ScreenRect,
    columns: u16,
    rows: u16,
}

#[derive(Debug, Clone)]
struct SettingsData {
    monitors: Vec<MonitorConfig>,
    grid_color: String,
    selection_color: String,
    selection_border_color: String,
    run_on_startup: bool,
    is_dark_mode: bool,
}

#[derive(Debug, Clone, Copy)]
struct MonitorEdit {
    columns_edit: Hwnd,
    rows_edit: Hwnd,
}

#[derive(Debug, Clone, Copy)]
struct PendingSnap {
    target: Hwnd,
    rect: ScreenRect,
}

#[derive(Debug, Clone)]
struct StoredMonitorConfig {
    device_name: String,
    display_name: String,
    columns: u16,
    rows: u16,
}

#[derive(Debug, Clone)]
struct StoredSettings {
    monitors: Vec<StoredMonitorConfig>,
    grid_color: String,
    selection_color: String,
    selection_border_color: String,
    run_on_startup: Option<bool>,
    is_dark_mode: Option<bool>,
}

#[derive(Debug)]
struct AppState {
    main_hwnd: Hwnd,
    settings_hwnd: Hwnd,
    monitor_edits: Vec<MonitorEdit>,
    grid_color_edit: Hwnd,
    selection_color_edit: Hwnd,
    selection_border_color_edit: Hwnd,
    run_startup_checkbox: Hwnd,
    syncing_settings_ui: bool,
    settings_dpi: u32,
    overlay_hwnd: Hwnd,
    hook: Hhook,
    app_icon: Hicon,
    left_button_down: bool,
    dragging: bool,
    suppress_right_up: bool,
    target_hwnd: Hwnd,
    grid: GridSpec,
    settings: SettingsData,
    monitor_device_name: String,
    monitor_rect: ScreenRect,
    tracker: Option<SelectionTracker>,
    selection: Option<GridSelection>,
    queued_snap: Option<PendingSnap>,
    snap_apply_pending: bool,
    settle_snap: Option<PendingSnap>,
}

impl AppState {
    fn new(main_hwnd: Hwnd, app_icon: Hicon) -> Self {
        let settings = settings_store::load_settings();
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

    fn clear_drag(&mut self) {
        self.dragging = false;
        self.target_hwnd = 0;
        self.tracker = None;
        self.selection = None;
    }
}

#[derive(Clone, Copy)]
struct RgbaColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

pub fn run() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let hinstance = GetModuleHandleW(null());
        let app_icon = load_app_icon(hinstance);
        register_window_classes(hinstance, app_icon);

        let main_hwnd = create_main_window(hinstance);
        if main_hwnd == 0 {
            show_message(0, "Snapdragin failed to create its main window.");
            return;
        }

        STATE
            .set(Mutex::new(AppState::new(main_hwnd, app_icon)))
            .expect("app state should only be initialized once");

        tray::add_tray_icon(main_hwnd);

        let hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(mouse_hook_proc),
            GetModuleHandleW(null()),
            0,
        );

        if hook == 0 {
            show_message(
                main_hwnd,
                "Snapdragin failed to install the global mouse hook.",
            );
            tray::remove_tray_icon(main_hwnd);
            DestroyWindow(main_hwnd);
            return;
        }

        with_state(|state| state.hook = hook);

        ShowWindow(main_hwnd, SW_HIDE);
        message_loop();
    }
}

unsafe fn register_window_classes(hinstance: Hinstance, app_icon: Hicon) {
    let main_class = wide(MAIN_CLASS);
    let overlay_class = wide(OVERLAY_CLASS);
    let settings_class = wide(SETTINGS_CLASS);

    let main_wc = Wndclassexw {
        cb_size: size_of::<Wndclassexw>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        lpfn_wnd_proc: Some(main_wnd_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinstance,
        h_icon: app_icon,
        h_cursor: LoadCursorW(0, IDC_ARROW as Pcwstr),
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: main_class.as_ptr(),
        h_icon_sm: app_icon,
    };
    RegisterClassExW(&main_wc);

    let overlay_wc = Wndclassexw {
        cb_size: size_of::<Wndclassexw>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfn_wnd_proc: Some(overlay::wnd_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinstance,
        h_icon: 0,
        h_cursor: LoadCursorW(0, IDC_ARROW as Pcwstr),
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: overlay_class.as_ptr(),
        h_icon_sm: 0,
    };
    RegisterClassExW(&overlay_wc);

    let settings_wc = Wndclassexw {
        cb_size: size_of::<Wndclassexw>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfn_wnd_proc: Some(settings_window::wnd_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinstance,
        h_icon: app_icon,
        h_cursor: LoadCursorW(0, IDC_ARROW as Pcwstr),
        hbr_background: (COLOR_WINDOW + 1) as Hbrush,
        lpsz_menu_name: null(),
        lpsz_class_name: settings_class.as_ptr(),
        h_icon_sm: app_icon,
    };
    RegisterClassExW(&settings_wc);
}

unsafe fn create_main_window(hinstance: Hinstance) -> Hwnd {
    let class_name = wide(MAIN_CLASS);
    let title = wide(APP_NAME);
    CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_OVERLAPPEDWINDOW,
        0,
        0,
        0,
        0,
        0,
        0,
        hinstance,
        null_mut(),
    )
}

unsafe fn message_loop() {
    let mut msg: Msg = zeroed();
    while GetMessageW(&mut msg, 0, 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: Hwnd,
    msg: u32,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_TRAYICON => {
            let event = tray_event(lparam);
            match event {
                WM_CONTEXTMENU | WM_RBUTTONUP => tray::show_context_menu(hwnd),
                WM_LBUTTONDBLCLK => settings_window::show_settings_window(hwnd),
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            tray::handle_menu_command(wparam & 0xFFFF);
            0
        }
        WM_APPLY_SNAP => {
            apply_queued_snap();
            0
        }
        WM_TIMER => {
            if wparam == SNAP_SETTLE_TIMER_ID {
                apply_snap_settle_timer(hwnd);
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_DISPLAYCHANGE => {
            settings_window::refresh_monitors();
            settings_window::rebuild_settings_window_if_open();
            0
        }
        WM_DESTROY => {
            cleanup(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn mouse_hook_proc(n_code: i32, wparam: Wparam, lparam: Lparam) -> Lresult {
    if n_code >= 0 && lparam != 0 {
        let hook = *(lparam as *const Msllhookstruct);
        if handle_mouse_message(wparam as u32, ScreenPoint::new(hook.pt.x, hook.pt.y)) {
            return 1;
        }
    }

    let hook = STATE
        .get()
        .and_then(|state| state.lock().ok().map(|state| state.hook))
        .unwrap_or_default();

    CallNextHookEx(hook, n_code, wparam, lparam)
}

fn handle_mouse_message(message: u32, point: ScreenPoint) -> bool {
    match message {
        WM_MOUSEMOVE => {
            if with_state(|state| state.dragging).unwrap_or(false)
                && !desktop::left_button_is_down()
            {
                finish_drag_at(point);
            } else {
                update_drag(point);
            }
            false
        }
        WM_LBUTTONDOWN => {
            with_state(|state| state.left_button_down = true);
            false
        }
        WM_LBUTTONUP => {
            let should_finish = with_state(|state| {
                state.left_button_down = false;
                state.dragging
            })
            .unwrap_or(false);

            if should_finish {
                finish_drag_at(point);
            }
            false
        }
        WM_RBUTTONDOWN => handle_right_button_down(point),
        WM_RBUTTONUP => with_state(|state| {
            if state.suppress_right_up {
                state.suppress_right_up = false;
                true
            } else {
                state.dragging
            }
        })
        .unwrap_or(false),
        _ => false,
    }
}

fn handle_right_button_down(point: ScreenPoint) -> bool {
    let physical_left_down = desktop::left_button_is_down();
    let already_dragging = with_state(|state| {
        state.left_button_down = physical_left_down;
        state.dragging
    })
    .unwrap_or(false);

    if already_dragging {
        finish_drag_at(point);
        with_state(|state| state.suppress_right_up = true);
        return true;
    }

    if !physical_left_down {
        return false;
    }

    let Some(target) = desktop::target_window(point) else {
        return false;
    };

    if !desktop::window_is_in_move_size_loop(target) {
        return false;
    }

    with_state(|state| state.suppress_right_up = true);

    if unsafe { !desktop::break_drag_loop(target, point) } {
        return true;
    }

    begin_drag(target, point);
    true
}

fn begin_drag(target: Hwnd, point: ScreenPoint) {
    settings_window::refresh_monitors();
    let (monitor_info, grid) = with_state(|state| {
        let monitor_info = active_monitor_for_point(state, point);
        let grid = grid_for_monitor(state, &monitor_info);
        (monitor_info, grid)
    })
    .unwrap_or_else(|| {
        let monitor_info = unsafe { desktop::monitor_info_from_point(point) };
        let grid = default_grid();
        (monitor_info, grid)
    });
    let monitor_rect = monitor_info.work_rect;
    let mut tracker = SelectionTracker::new(grid, monitor_rect);
    let selection = tracker.begin(point);

    with_state(|state| {
        state.dragging = true;
        state.target_hwnd = target;
        state.grid = grid;
        state.monitor_device_name = monitor_info.device_name.clone();
        state.monitor_rect = monitor_rect;
        state.tracker = Some(tracker);
        state.selection = selection;
    });

    unsafe {
        overlay::show_overlay(monitor_rect);
    }
    queue_current_snap();
}

fn grid_for_monitor(state: &AppState, monitor_info: &MonitorConfig) -> GridSpec {
    state
        .settings
        .monitors
        .iter()
        .find(|monitor| monitor.device_name == monitor_info.device_name)
        .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
        .unwrap_or_else(|| GridSpec::clamped(monitor_info.columns, monitor_info.rows))
}

fn monitor_for_point(state: &AppState, point: ScreenPoint) -> Option<MonitorConfig> {
    state
        .settings
        .monitors
        .iter()
        .find(|monitor| monitor.monitor_rect.contains(point))
        .cloned()
        .or_else(|| {
            state
                .settings
                .monitors
                .iter()
                .min_by_key(|monitor| point_distance_to_rect_squared(point, monitor.monitor_rect))
                .cloned()
        })
}

fn active_monitor_for_point(state: &AppState, point: ScreenPoint) -> MonitorConfig {
    let mut native = unsafe { desktop::monitor_info_from_point(point) };

    if let Some(monitor) = state
        .settings
        .monitors
        .iter()
        .find(|monitor| monitor.device_name == native.device_name)
        .cloned()
    {
        return monitor;
    }

    if let Some(saved) = monitor_for_point(state, point) {
        native.columns = saved.columns;
        native.rows = saved.rows;
    }

    native
}

fn point_distance_to_rect_squared(point: ScreenPoint, rect: ScreenRect) -> i64 {
    let x = i64::from(point.x);
    let y = i64::from(point.y);
    let left = i64::from(rect.x);
    let top = i64::from(rect.y);
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);

    let dx = if x < left {
        left - x
    } else if x > right {
        x - right
    } else {
        0
    };
    let dy = if y < top {
        top - y
    } else if y > bottom {
        y - bottom
    } else {
        0
    };

    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
}

fn update_drag(point: ScreenPoint) {
    let (changed, new_overlay_monitor) = update_drag_monitor_and_selection(point);

    if changed {
        unsafe {
            if let Some(monitor) = new_overlay_monitor {
                overlay::show_overlay(monitor);
            } else {
                overlay::invalidate_overlay();
            }
        }
        queue_current_snap();
    }
}

fn update_drag_monitor_and_selection(point: ScreenPoint) -> (bool, Option<ScreenRect>) {
    with_state(|state| {
        if !state.dragging {
            return (false, None);
        }

        let monitor_info = active_monitor_for_point(state, point);
        let monitor_rect = monitor_info.work_rect;

        if state.monitor_device_name == monitor_info.device_name {
            return (update_drag_selection_in_state(state, point), None);
        }

        let grid = grid_for_monitor(state, &monitor_info);
        let mut tracker = SelectionTracker::new(grid, monitor_rect);
        let selection = tracker.begin(point);

        state.grid = grid;
        state.monitor_device_name = monitor_info.device_name.clone();
        state.monitor_rect = monitor_rect;
        state.tracker = Some(tracker);
        state.selection = selection;

        (selection.is_some(), Some(monitor_rect))
    })
    .unwrap_or((false, None))
}

fn update_drag_selection_in_state(state: &mut AppState, point: ScreenPoint) -> bool {
    if !state.dragging {
        return false;
    }

    let Some(mut tracker) = state.tracker else {
        return false;
    };

    let selection = tracker.update(point);
    state.tracker = Some(tracker);

    if selection != state.selection {
        state.selection = selection;
        return true;
    }

    false
}

fn finish_drag() {
    let (overlay, snap) = with_state(|state| {
        let overlay = state.overlay_hwnd;
        let snap = current_snap(state);
        state.clear_drag();
        (overlay, snap)
    })
    .unwrap_or_default();

    unsafe {
        if overlay != 0 {
            ShowWindow(overlay, SW_HIDE);
        }
        ReleaseCapture();
    }

    if let Some(snap) = snap {
        queue_snap(snap);
    }
}

fn finish_drag_at(point: ScreenPoint) {
    update_drag(point);
    finish_drag();
}

fn queue_current_snap() {
    let snap = with_state(|state| current_snap(state)).flatten();

    if let Some(snap) = snap {
        queue_snap(snap);
    }
}

fn current_snap(state: &AppState) -> Option<PendingSnap> {
    let selection = state.selection?;
    let rect = selection.screen_rect(state.grid, state.monitor_rect)?;
    Some(PendingSnap {
        target: state.target_hwnd,
        rect,
    })
}

fn queue_snap(snap: PendingSnap) {
    let main_hwnd = with_state(|state| {
        state.queued_snap = Some(snap);

        if state.snap_apply_pending {
            return 0;
        }

        state.snap_apply_pending = true;
        state.main_hwnd
    })
    .unwrap_or_default();

    if main_hwnd != 0 {
        unsafe {
            PostMessageW(main_hwnd, WM_APPLY_SNAP, 0, 0);
        }
    }
}

fn apply_queued_snap() {
    let snap = with_state(|state| {
        state.snap_apply_pending = false;
        state.queued_snap.take()
    })
    .flatten();

    if let Some(snap) = snap {
        unsafe {
            apply_snap_to_target(snap);
        }
        queue_snap_settle(snap);
    }
}

unsafe fn apply_snap_to_target(snap: PendingSnap) {
    for attempt in 0..SNAP_SETTLE_ATTEMPTS {
        if desktop::window_is_in_move_size_loop(snap.target) {
            let _ = desktop::break_drag_loop(snap.target, desktop::cursor_position());
        }

        set_target_window_rect(snap.target, snap.rect);

        if attempt + 1 < SNAP_SETTLE_ATTEMPTS {
            thread::sleep(Duration::from_millis(SNAP_SETTLE_DELAY_MS));
        }
    }

    redraw_target_window(snap.target);
}

unsafe fn set_target_window_rect(target: Hwnd, rect: ScreenRect) {
    SetWindowPos(
        target,
        0,
        rect.x,
        rect.y,
        i32::try_from(rect.width).unwrap_or(i32::MAX),
        i32::try_from(rect.height).unwrap_or(i32::MAX),
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
    );
}

unsafe fn redraw_target_window(target: Hwnd) {
    InvalidateRect(target, null(), 1);
    RedrawWindow(
        target,
        null(),
        0,
        RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW | RDW_FRAME,
    );
}

fn queue_snap_settle(snap: PendingSnap) {
    let main_hwnd = with_state(|state| {
        state.settle_snap = Some(snap);
        state.main_hwnd
    })
    .unwrap_or_default();

    if main_hwnd != 0 {
        unsafe {
            SetTimer(main_hwnd, SNAP_SETTLE_TIMER_ID, SNAP_SETTLE_TIMER_MS, None);
        }
    }
}

fn apply_snap_settle_timer(hwnd: Hwnd) {
    unsafe {
        KillTimer(hwnd, SNAP_SETTLE_TIMER_ID);
    }

    let snap = with_state(|state| state.settle_snap.take()).flatten();
    if let Some(snap) = snap {
        unsafe {
            if desktop::window_is_in_move_size_loop(snap.target) {
                let _ = desktop::break_drag_loop(snap.target, desktop::cursor_position());
            }

            set_target_window_rect(snap.target, snap.rect);
            redraw_target_window(snap.target);
        }
    }
}

unsafe fn set_window_text(hwnd: Hwnd, text: &str) {
    let text = wide(text);
    SetWindowTextW(hwnd, text.as_ptr());
}

fn default_grid() -> GridSpec {
    GridSpec::new(2, 2).expect("default grid is valid")
}

unsafe fn show_about(hwnd: Hwnd) {
    show_message(
        hwnd,
        "Snapdragin is running.\n\nDrag a window with the left mouse button, right-click to open the grid, move across the cells, then right-click again or release left click to snap.",
    );
}

unsafe fn cleanup(hwnd: Hwnd) {
    KillTimer(hwnd, SNAP_SETTLE_TIMER_ID);
    tray::remove_tray_icon(hwnd);

    let (hook, overlay, settings) = with_state(|state| {
        let hook = state.hook;
        let overlay = state.overlay_hwnd;
        let settings = state.settings_hwnd;
        state.hook = 0;
        state.overlay_hwnd = 0;
        state.settings_hwnd = 0;
        (hook, overlay, settings)
    })
    .unwrap_or_default();

    if hook != 0 {
        UnhookWindowsHookEx(hook);
    }

    if overlay != 0 {
        DestroyWindow(overlay);
    }

    if settings != 0 {
        DestroyWindow(settings);
    }
}

fn tray_event(lparam: Lparam) -> u32 {
    let raw = lparam as u32;
    let low_word = raw & 0xFFFF;

    match low_word {
        WM_CONTEXTMENU | WM_LBUTTONDBLCLK | WM_RBUTTONUP => low_word,
        _ => raw,
    }
}

unsafe fn load_app_icon(hinstance: Hinstance) -> Hicon {
    let icon = LoadIconW(hinstance, APP_ICON_ID as Pcwstr);
    if icon != 0 {
        icon
    } else {
        LoadIconW(0, IDI_APPLICATION as Pcwstr)
    }
}

fn loword_signed(value: Lparam) -> i32 {
    (value as u32 & 0xFFFF) as i16 as i32
}

fn hiword_signed(value: Lparam) -> i32 {
    ((value as u32 >> 16) & 0xFFFF) as i16 as i32
}

fn hiword(value: Wparam) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}

fn point_lparam(point: ScreenPoint) -> Lparam {
    let x = u32::from(point.x as i16 as u16);
    let y = u32::from(point.y as i16 as u16);
    (x | (y << 16)) as Lparam
}

fn scale_value(dpi: u32, value: i32) -> i32 {
    let scaled = i64::from(value) * i64::from(dpi.max(1));
    ((scaled + 48) / 96).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn unscale_value(dpi: u32, value: i32) -> i32 {
    let dpi = i64::from(dpi.max(1));
    ((i64::from(value) * 96 + dpi / 2) / dpi).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn scaled_rect(dpi: u32, left: i32, top: i32, right: i32, bottom: i32) -> Rect {
    Rect::new(
        scale_value(dpi, left),
        scale_value(dpi, top),
        scale_value(dpi, right),
        scale_value(dpi, bottom),
    )
}

fn with_state<R>(f: impl FnOnce(&mut AppState) -> R) -> Option<R> {
    let state = STATE.get()?;
    let mut state = state.lock().ok()?;
    Some(f(&mut state))
}

unsafe fn show_message(hwnd: Hwnd, message: &str) {
    let message = wide(message);
    let title = wide(APP_NAME);
    MessageBoxW(
        hwnd,
        message.as_ptr(),
        title.as_ptr(),
        MB_OK | MB_ICONINFORMATION,
    );
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

fn string_from_wide_z(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

fn copy_wide(buffer: &mut [u16], value: &str) {
    let encoded: Vec<u16> = value.encode_utf16().collect();
    let count = encoded.len().min(buffer.len().saturating_sub(1));
    buffer[..count].copy_from_slice(&encoded[..count]);
    buffer[count] = 0;
}

const fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    (red as u32) | ((green as u32) << 8) | ((blue as u32) << 16)
}
