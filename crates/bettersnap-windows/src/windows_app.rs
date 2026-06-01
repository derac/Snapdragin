use std::{
    env,
    ffi::c_void,
    fs,
    mem::{size_of, zeroed},
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{Mutex, OnceLock},
};

use bettersnap_core::{GridSelection, GridSpec, ScreenPoint, ScreenRect, SelectionTracker};

type Bool = i32;
type Dword = u32;
type Hbitmap = isize;
type Hbrush = isize;
type Hcursor = isize;
type Hdc = isize;
type Hgdiobj = isize;
type Hhook = isize;
type Hicon = isize;
type Hinstance = isize;
type Hmenu = isize;
type Hmonitor = isize;
type Hpen = isize;
type Hwnd = isize;
type Lparam = isize;
type Lresult = isize;
type Pcwstr = *const u16;
type Uint = u32;
type Wparam = usize;

const APP_NAME: &str = "Snapdragin'";
const APP_DIR_NAME: &str = "Snapdragin";
const STARTUP_SCRIPT_NAME: &str = "Snapdragin.cmd";
const MAIN_CLASS: &str = "SnapdraginMainWindow";
const OVERLAY_CLASS: &str = "SnapdraginOverlayWindow";
const SETTINGS_CLASS: &str = "SnapdraginSettingsWindow";
const APP_ICON_ID: usize = 101;

const ID_TRAY: u32 = 1;
const WM_TRAYICON: u32 = WM_USER + 1;

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
const SWP_NOCOPYBITS: u32 = 0x0100;

const HWND_TOPMOST: Hwnd = -1;

const ULW_ALPHA: u32 = 0x0000_0002;
const AC_SRC_OVER: u8 = 0;
const AC_SRC_ALPHA: u8 = 1;
const BI_RGB: u32 = 0;
const DIB_RGB_COLORS: u32 = 0;

const SMTO_ABORTIFHUNG: u32 = 0x0002;
const SMTO_ERRORONEXIT: u32 = 0x0020;
const DRAG_CANCEL_TIMEOUT_MS: u32 = 50;

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
const DPI_AWARENESS_PER_MONITOR_AWARE: i32 = 2;
const MDT_EFFECTIVE_DPI: i32 = 0;
const MIN_GRID_DIMENSION: u16 = 1;
const MAX_GRID_DIMENSION: u16 = 20;
const DEFAULT_GRID_COLOR: &str = "#FFFFFF22";
const DEFAULT_SELECTION_COLOR: &str = "#00FFFF22";
const DEFAULT_SELECTION_BORDER_COLOR: &str = "#00FFFF88";
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
    monitor_rect: ScreenRect,
    tracker: Option<SelectionTracker>,
    selection: Option<GridSelection>,
}

impl AppState {
    fn new(main_hwnd: Hwnd, app_icon: Hicon) -> Self {
        let settings = load_settings();
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
            left_button_down: left_button_is_down(),
            dragging: false,
            suppress_right_up: false,
            target_hwnd: 0,
            grid,
            settings,
            monitor_rect: ScreenRect::new(0, 0, 1, 1),
            tracker: None,
            selection: None,
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
            show_message(0, "Snapdragin' failed to create its main window.");
            return;
        }

        STATE
            .set(Mutex::new(AppState::new(main_hwnd, app_icon)))
            .expect("app state should only be initialized once");

        add_tray_icon(main_hwnd);

        let hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(mouse_hook_proc),
            GetModuleHandleW(null()),
            0,
        );

        if hook == 0 {
            show_message(
                main_hwnd,
                "Snapdragin' failed to install the global mouse hook.",
            );
            remove_tray_icon(main_hwnd);
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
        lpfn_wnd_proc: Some(overlay_wnd_proc),
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
        lpfn_wnd_proc: Some(settings_wnd_proc),
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
                WM_CONTEXTMENU | WM_RBUTTONUP => show_context_menu(hwnd),
                WM_LBUTTONDBLCLK => show_settings_window(hwnd),
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            handle_menu_command(wparam & 0xFFFF);
            0
        }
        WM_DISPLAYCHANGE => {
            refresh_monitors();
            rebuild_settings_window_if_open();
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

unsafe extern "system" fn settings_wnd_proc(
    hwnd: Hwnd,
    msg: u32,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_COMMAND => {
            let control_id = wparam & 0xFFFF;
            let notification = (wparam >> 16) & 0xFFFF;
            let syncing = with_state(|state| state.syncing_settings_ui).unwrap_or(false);

            if control_id == ID_RESET_COLORS {
                reset_colors_from_window(hwnd);
            } else if !syncing && live_setting_changed(control_id, notification) {
                apply_settings_from_window(hwnd);
            }
            0
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT => {
            let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
            SetTextColor(
                wparam as Hdc,
                if dark {
                    rgb(255, 255, 255)
                } else {
                    rgb(0, 0, 0)
                },
            );
            SetBkColor(
                wparam as Hdc,
                if dark {
                    rgb(45, 45, 45)
                } else {
                    rgb(255, 255, 255)
                },
            );
            settings_control_brush()
        }
        WM_PAINT => {
            paint_settings(hwnd);
            0
        }
        WM_LBUTTONDOWN => {
            let dpi = with_state(|state| state.settings_dpi).unwrap_or(96);
            let x = unscale_value(dpi, loword_signed(lparam));
            let y = unscale_value(dpi, hiword_signed(lparam));
            if (662..=694).contains(&x) && (4..=36).contains(&y) {
                apply_settings_from_window(hwnd);
                DestroyWindow(hwnd);
            } else if (632..=662).contains(&x) && (4..=36).contains(&y) {
                toggle_theme(hwnd);
            } else if y < 40 {
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0);
            } else if swatch_hit(x, y, 0) {
                pick_color(hwnd, ColorTarget::Grid);
            } else if swatch_hit(x, y, 1) {
                pick_color(hwnd, ColorTarget::Selection);
            } else if swatch_hit(x, y, 2) {
                pick_color(hwnd, ColorTarget::Border);
            }
            0
        }
        WM_CLOSE => {
            apply_settings_from_window(hwnd);
            DestroyWindow(hwnd);
            0
        }
        WM_DISPLAYCHANGE => {
            refresh_monitors();
            rebuild_settings_window_if_open();
            0
        }
        WM_DPICHANGED => {
            let _dpi = hiword(wparam) as u32;
            with_state(|state| state.settings_dpi = 96);
            let suggested = lparam as *const Rect;
            if !suggested.is_null() {
                let suggested = *suggested;
                SetWindowPos(
                    hwnd,
                    0,
                    suggested.left,
                    suggested.top,
                    SETTINGS_WIDTH,
                    SETTINGS_HEIGHT,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            layout_settings_controls(hwnd);
            InvalidateRect(hwnd, null(), 1);
            0
        }
        WM_DESTROY => {
            with_state(|state| {
                if state.settings_hwnd == hwnd {
                    state.settings_hwnd = 0;
                    state.monitor_edits.clear();
                    state.grid_color_edit = 0;
                    state.selection_color_edit = 0;
                    state.selection_border_color_edit = 0;
                    state.run_startup_checkbox = 0;
                }
            });
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: Hwnd,
    msg: u32,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            paint_overlay(hwnd);
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
            if with_state(|state| state.dragging).unwrap_or(false) && !left_button_is_down() {
                finish_drag();
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
                finish_drag();
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
    let physical_left_down = left_button_is_down();
    let already_dragging = with_state(|state| {
        state.left_button_down = physical_left_down;
        state.dragging
    })
    .unwrap_or(false);

    if already_dragging {
        finish_drag();
        with_state(|state| state.suppress_right_up = true);
        return true;
    }

    if !physical_left_down {
        return false;
    }

    let Some(target) = target_window(point) else {
        return false;
    };

    if !window_is_in_move_size_loop(target) {
        return false;
    }

    unsafe {
        break_drag_loop(target);
    }

    begin_drag(target, point);
    with_state(|state| state.suppress_right_up = true);
    true
}

fn begin_drag(target: Hwnd, point: ScreenPoint) {
    refresh_monitors();
    let monitor_info = unsafe { monitor_info_from_point(point) };
    let monitor_rect = monitor_info.work_rect;
    let grid = with_state(|state| {
        state
            .settings
            .monitors
            .iter()
            .find(|monitor| monitor.device_name == monitor_info.device_name)
            .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
            .unwrap_or(state.grid)
    })
    .unwrap_or_else(default_grid);
    let mut tracker = SelectionTracker::new(grid, monitor_rect);
    let selection = tracker.begin(point);

    with_state(|state| {
        state.dragging = true;
        state.target_hwnd = target;
        state.grid = grid;
        state.monitor_rect = monitor_rect;
        state.tracker = Some(tracker);
        state.selection = selection;
    });

    unsafe {
        show_overlay(monitor_rect);
    }
    apply_snap();
}

fn update_drag(point: ScreenPoint) {
    let changed = with_state(|state| {
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
    })
    .unwrap_or(false);

    if changed {
        unsafe {
            invalidate_overlay();
        }
        apply_snap();
    }
}

fn finish_drag() {
    apply_snap();

    let overlay = with_state(|state| {
        let overlay = state.overlay_hwnd;
        state.clear_drag();
        overlay
    })
    .unwrap_or_default();

    unsafe {
        if overlay != 0 {
            ShowWindow(overlay, SW_HIDE);
        }
        ReleaseCapture();
    }
}

fn apply_snap() {
    let snap = with_state(|state| {
        let selection = state.selection?;
        let rect = selection.screen_rect(state.grid, state.monitor_rect)?;
        Some((state.target_hwnd, rect))
    })
    .flatten();

    if let Some((target, rect)) = snap {
        unsafe {
            let first_rect = dpi_compensated_target_rect(target, rect);
            set_target_window_rect(target, first_rect);

            let settled_rect = dpi_compensated_target_rect(target, rect);
            if settled_rect != first_rect {
                set_target_window_rect(target, settled_rect);
            }
        }
    }
}

unsafe fn set_target_window_rect(target: Hwnd, rect: ScreenRect) {
    SetWindowPos(
        target,
        0,
        rect.x,
        rect.y,
        i32::try_from(rect.width).unwrap_or(i32::MAX),
        i32::try_from(rect.height).unwrap_or(i32::MAX),
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOCOPYBITS | SWP_SHOWWINDOW,
    );
}

unsafe fn dpi_compensated_target_rect(target: Hwnd, rect: ScreenRect) -> ScreenRect {
    let dest_monitor = MonitorFromPoint(
        Point {
            x: rect.x + i32::try_from(rect.width / 2).unwrap_or_default(),
            y: rect.y + i32::try_from(rect.height / 2).unwrap_or_default(),
        },
        MONITOR_DEFAULTTONEAREST,
    );

    let mut dest_dpi_x = 96;
    let mut dest_dpi_y = 96;
    let dpi_result = GetDpiForMonitor(
        dest_monitor,
        MDT_EFFECTIVE_DPI,
        &mut dest_dpi_x,
        &mut dest_dpi_y,
    );

    let awareness = GetAwarenessFromDpiAwarenessContext(GetWindowDpiAwarenessContext(target));
    let window_dpi = GetDpiForWindow(target);

    if dpi_result != 0
        || awareness != DPI_AWARENESS_PER_MONITOR_AWARE
        || window_dpi == 0
        || dest_dpi_x == 0
        || window_dpi == dest_dpi_x
    {
        return rect;
    }

    let width = ((u64::from(rect.width) * u64::from(window_dpi)) / u64::from(dest_dpi_x))
        .try_into()
        .unwrap_or(u32::MAX);
    let height = ((u64::from(rect.height) * u64::from(window_dpi)) / u64::from(dest_dpi_x))
        .try_into()
        .unwrap_or(u32::MAX);

    ScreenRect::new(rect.x, rect.y, width, height)
}

unsafe fn show_overlay(monitor: ScreenRect) {
    let overlay = with_state(|state| state.overlay_hwnd).unwrap_or_default();
    let hwnd = if overlay == 0 {
        let class_name = wide(OVERLAY_CLASS);
        let title = wide("Snapdragin' Overlay");
        let created = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            monitor.x,
            monitor.y,
            i32::try_from(monitor.width).unwrap_or(i32::MAX),
            i32::try_from(monitor.height).unwrap_or(i32::MAX),
            0,
            0,
            GetModuleHandleW(null()),
            null_mut(),
        );
        with_state(|state| state.overlay_hwnd = created);
        created
    } else {
        overlay
    };

    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        monitor.x,
        monitor.y,
        i32::try_from(monitor.width).unwrap_or(i32::MAX),
        i32::try_from(monitor.height).unwrap_or(i32::MAX),
        SWP_NOACTIVATE,
    );
    render_overlay(hwnd);
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
}

unsafe fn invalidate_overlay() {
    let overlay = with_state(|state| state.overlay_hwnd).unwrap_or_default();
    if overlay != 0 {
        render_overlay(overlay);
    }
}

unsafe fn paint_overlay(hwnd: Hwnd) {
    let mut paint: Paintstruct = zeroed();
    let hdc = BeginPaint(hwnd, &mut paint);
    if hdc != 0 {
        EndPaint(hwnd, &paint);
    }

    render_overlay(hwnd);
}

unsafe fn render_overlay(hwnd: Hwnd) {
    let snapshot = with_state(|state| {
        (
            state.grid,
            state.monitor_rect,
            state.selection,
            state.dragging,
            state.settings.grid_color.clone(),
            state.settings.selection_color.clone(),
            state.settings.selection_border_color.clone(),
        )
    });

    let Some((
        grid,
        monitor,
        selection,
        dragging,
        grid_color,
        selection_color,
        selection_border_color,
    )) = snapshot
    else {
        return;
    };

    if monitor.is_empty() {
        return;
    }

    let width = usize::try_from(monitor.width).unwrap_or_default();
    let height = usize::try_from(monitor.height).unwrap_or_default();
    let Some(pixel_count) = width.checked_mul(height) else {
        return;
    };
    if pixel_count == 0 {
        return;
    }

    let mut pixels = vec![0_u32; pixel_count];
    draw_overlay_grid(
        &mut pixels,
        width,
        height,
        grid,
        rgba_from_hex(&grid_color, DEFAULT_GRID_COLOR),
    );

    if dragging {
        if let Some(selection) = selection {
            if let Some(rect) = selection.screen_rect(grid, monitor) {
                draw_overlay_selection(
                    &mut pixels,
                    width,
                    height,
                    monitor,
                    rect,
                    rgba_from_hex(&selection_color, DEFAULT_SELECTION_COLOR),
                    rgba_from_hex(&selection_border_color, DEFAULT_SELECTION_BORDER_COLOR),
                );
            }
        }
    }

    update_layered_overlay(hwnd, monitor, width, height, &pixels);
}

fn draw_overlay_grid(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    grid: GridSpec,
    color: RgbaColor,
) {
    if width == 0 || height == 0 {
        return;
    }

    for column in 0..=grid.columns() {
        let x = (usize::from(column) * width / usize::from(grid.columns())).min(width - 1);
        for y in 0..height {
            blend_overlay_pixel(pixels, width, x, y, color);
        }
    }

    for row in 0..=grid.rows() {
        let y = (usize::from(row) * height / usize::from(grid.rows())).min(height - 1);
        for x in 0..width {
            blend_overlay_pixel(pixels, width, x, y, color);
        }
    }
}

fn draw_overlay_selection(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    monitor: ScreenRect,
    rect: ScreenRect,
    fill_color: RgbaColor,
    border_color: RgbaColor,
) {
    let left = (rect.x - monitor.x).max(0);
    let top = (rect.y - monitor.y).max(0);
    let right = (rect.x - monitor.x + i32::try_from(rect.width).unwrap_or(i32::MAX))
        .clamp(0, i32::try_from(width).unwrap_or(i32::MAX));
    let bottom = (rect.y - monitor.y + i32::try_from(rect.height).unwrap_or(i32::MAX))
        .clamp(0, i32::try_from(height).unwrap_or(i32::MAX));

    let left = usize::try_from(left).unwrap_or_default();
    let top = usize::try_from(top).unwrap_or_default();
    let right = usize::try_from(right).unwrap_or(width);
    let bottom = usize::try_from(bottom).unwrap_or(height);

    if left >= right || top >= bottom {
        return;
    }

    for y in top..bottom {
        for x in left..right {
            blend_overlay_pixel(pixels, width, x, y, fill_color);
        }
    }

    let thickness = 3_usize.min(right - left).min(bottom - top);
    for offset in 0..thickness {
        let top_y = top + offset;
        let bottom_y = bottom - 1 - offset;
        for x in left..right {
            blend_overlay_pixel(pixels, width, x, top_y, border_color);
            blend_overlay_pixel(pixels, width, x, bottom_y, border_color);
        }

        let left_x = left + offset;
        let right_x = right - 1 - offset;
        for y in top..bottom {
            blend_overlay_pixel(pixels, width, left_x, y, border_color);
            blend_overlay_pixel(pixels, width, right_x, y, border_color);
        }
    }
}

fn blend_overlay_pixel(pixels: &mut [u32], width: usize, x: usize, y: usize, color: RgbaColor) {
    let index = y.saturating_mul(width).saturating_add(x);
    if index >= pixels.len() || color.alpha == 0 {
        return;
    }

    let destination = pixels[index];
    let source_alpha = u32::from(color.alpha);
    let inverse_alpha = 255 - source_alpha;

    let source_red = u32::from(color.red) * source_alpha / 255;
    let source_green = u32::from(color.green) * source_alpha / 255;
    let source_blue = u32::from(color.blue) * source_alpha / 255;

    let destination_blue = destination & 0xFF;
    let destination_green = (destination >> 8) & 0xFF;
    let destination_red = (destination >> 16) & 0xFF;
    let destination_alpha = (destination >> 24) & 0xFF;

    let out_blue = source_blue + destination_blue * inverse_alpha / 255;
    let out_green = source_green + destination_green * inverse_alpha / 255;
    let out_red = source_red + destination_red * inverse_alpha / 255;
    let out_alpha = source_alpha + destination_alpha * inverse_alpha / 255;

    pixels[index] = out_blue | (out_green << 8) | (out_red << 16) | (out_alpha << 24);
}

unsafe fn update_layered_overlay(
    hwnd: Hwnd,
    monitor: ScreenRect,
    width: usize,
    height: usize,
    pixels: &[u32],
) {
    let screen_dc = GetDC(0);
    if screen_dc == 0 {
        return;
    }

    let memory_dc = CreateCompatibleDC(screen_dc);
    if memory_dc == 0 {
        ReleaseDC(0, screen_dc);
        return;
    }

    let bitmap_info = Bitmapinfo {
        bmi_header: Bitmapinfoheader {
            bi_size: size_of::<Bitmapinfoheader>() as u32,
            bi_width: i32::try_from(width).unwrap_or(i32::MAX),
            bi_height: -i32::try_from(height).unwrap_or(i32::MAX),
            bi_planes: 1,
            bi_bit_count: 32,
            bi_compression: BI_RGB,
            bi_size_image: 0,
            bi_x_pels_per_meter: 0,
            bi_y_pels_per_meter: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        },
        bmi_colors: [Rgbquad::default()],
    };
    let mut bits: *mut c_void = null_mut();
    let bitmap = CreateDIBSection(screen_dc, &bitmap_info, DIB_RGB_COLORS, &mut bits, 0, 0);
    if bitmap == 0 || bits.is_null() {
        if bitmap != 0 {
            DeleteObject(bitmap);
        }
        DeleteDC(memory_dc);
        ReleaseDC(0, screen_dc);
        return;
    }

    std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits.cast::<u32>(), pixels.len());

    let old_bitmap = SelectObject(memory_dc, bitmap);
    let destination = Point {
        x: monitor.x,
        y: monitor.y,
    };
    let size = Size {
        cx: i32::try_from(width).unwrap_or(i32::MAX),
        cy: i32::try_from(height).unwrap_or(i32::MAX),
    };
    let source = Point { x: 0, y: 0 };
    let blend = Blendfunction {
        blend_op: AC_SRC_OVER,
        blend_flags: 0,
        source_constant_alpha: 255,
        alpha_format: AC_SRC_ALPHA,
    };
    UpdateLayeredWindow(
        hwnd,
        screen_dc,
        &destination,
        &size,
        memory_dc,
        &source,
        0,
        &blend,
        ULW_ALPHA,
    );
    SelectObject(memory_dc, old_bitmap);
    DeleteObject(bitmap);
    DeleteDC(memory_dc);
    ReleaseDC(0, screen_dc);
}

unsafe fn add_tray_icon(hwnd: Hwnd) {
    let mut nid = notify_icon(hwnd);
    Shell_NotifyIconW(NIM_ADD, &mut nid);
}

unsafe fn remove_tray_icon(hwnd: Hwnd) {
    let mut nid = notify_icon(hwnd);
    Shell_NotifyIconW(NIM_DELETE, &mut nid);
}

unsafe fn notify_icon(hwnd: Hwnd) -> Notifyicondataw {
    let mut nid: Notifyicondataw = zeroed();
    nid.cb_size = size_of::<Notifyicondataw>() as u32;
    nid.h_wnd = hwnd;
    nid.u_id = ID_TRAY;
    nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.u_callback_message = WM_TRAYICON;
    nid.h_icon = with_state(|state| state.app_icon)
        .unwrap_or_else(|| load_app_icon(GetModuleHandleW(null())));
    copy_wide(&mut nid.sz_tip, APP_NAME);
    nid
}

unsafe fn show_context_menu(hwnd: Hwnd) {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return;
    }

    append_menu_string(menu, ID_SETTINGS, "Settings", false);
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_menu_string(menu, ID_EXIT, "Exit", false);

    let mut point: Point = zeroed();
    GetCursorPos(&mut point);
    SetForegroundWindow(hwnd);

    let command = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
        point.x,
        point.y,
        0,
        hwnd,
        null(),
    );
    DestroyMenu(menu);

    if command > 0 {
        handle_menu_command(command as usize);
    }
}

unsafe fn append_menu_string(menu: Hmenu, id: usize, text: &str, checked: bool) {
    let text = wide(text);
    let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
    AppendMenuW(menu, flags, id, text.as_ptr());
}

fn handle_menu_command(command: usize) {
    match command {
        ID_SETTINGS => {
            let hwnd = with_state(|state| state.main_hwnd).unwrap_or_default();
            unsafe {
                show_settings_window(hwnd);
            }
        }
        ID_ABOUT => {
            let hwnd = with_state(|state| state.main_hwnd).unwrap_or_default();
            unsafe {
                show_about(hwnd);
            }
        }
        ID_EXIT => {
            let hwnd = with_state(|state| state.main_hwnd).unwrap_or_default();
            unsafe {
                DestroyWindow(hwnd);
            }
        }
        _ => {}
    }
}

unsafe fn show_settings_window(owner: Hwnd) {
    refresh_monitors();

    let existing = with_state(|state| state.settings_hwnd).unwrap_or_default();
    if existing != 0 {
        sync_settings_window(with_state(|state| state.grid).unwrap_or_else(default_grid));
        InvalidateRect(existing, null(), 1);
        ShowWindow(existing, SW_SHOW);
        SetForegroundWindow(existing);
        return;
    }

    let dpi = 96;
    let class_name = wide(SETTINGS_CLASS);
    let title = wide(APP_NAME);
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_POPUP,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        SETTINGS_WIDTH,
        SETTINGS_HEIGHT,
        owner,
        0,
        GetModuleHandleW(null()),
        null_mut(),
    );

    if hwnd == 0 {
        return;
    }

    let settings = with_state(|state| state.settings.clone()).unwrap_or_else(load_settings);
    let mut monitor_edits = Vec::new();
    for (index, monitor) in settings.monitors.iter().enumerate() {
        if index >= 7 {
            break;
        }

        let y = 82 + i32::try_from(index).unwrap_or_default() * 56;
        let columns_edit =
            create_edit(hwnd, ID_MONITOR_EDIT_BASE + index * 2, 277, y, 40, 22, true);
        let rows_edit = create_edit(
            hwnd,
            ID_MONITOR_EDIT_BASE + index * 2 + 1,
            363,
            y,
            40,
            22,
            true,
        );
        set_window_text(columns_edit, &monitor.columns.to_string());
        set_window_text(rows_edit, &monitor.rows.to_string());
        monitor_edits.push(MonitorEdit {
            columns_edit,
            rows_edit,
        });
    }

    let grid_color_edit = create_edit(hwnd, ID_GRID_COLOR_EDIT, 445, 86, 199, 22, false);
    let selection_color_edit = create_edit(hwnd, ID_SELECTION_COLOR_EDIT, 445, 136, 199, 22, false);
    let selection_border_color_edit =
        create_edit(hwnd, ID_SELECTION_BORDER_EDIT, 445, 187, 199, 22, false);
    create_button(
        hwnd,
        ID_RESET_COLORS,
        "Reset to Defaults",
        445,
        223,
        228,
        28,
    );
    let run_startup_checkbox =
        create_checkbox(hwnd, ID_RUN_STARTUP, "Run on Startup", 445, 301, 160, 24);

    with_state(|state| {
        state.settings_hwnd = hwnd;
        state.settings_dpi = dpi;
        state.monitor_edits = monitor_edits;
        state.grid_color_edit = grid_color_edit;
        state.selection_color_edit = selection_color_edit;
        state.selection_border_color_edit = selection_border_color_edit;
        state.run_startup_checkbox = run_startup_checkbox;
    });

    layout_settings_controls(hwnd);
    sync_settings_window(with_state(|state| state.grid).unwrap_or_else(default_grid));
    ShowWindow(hwnd, SW_SHOW);
    SetForegroundWindow(hwnd);
}

unsafe fn create_edit(
    parent: Hwnd,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    numeric: bool,
) -> Hwnd {
    create_control(
        "EDIT",
        "",
        WS_CHILD
            | WS_VISIBLE
            | WS_BORDER
            | WS_TABSTOP
            | ES_AUTOHSCROLL
            | if numeric { ES_NUMBER } else { 0 },
        0,
        parent,
        id,
        x,
        y,
        width,
        height,
    )
}

unsafe fn create_button(
    parent: Hwnd,
    id: usize,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Hwnd {
    create_control(
        "BUTTON",
        text,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
        0,
        parent,
        id,
        x,
        y,
        width,
        height,
    )
}

unsafe fn create_checkbox(
    parent: Hwnd,
    id: usize,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Hwnd {
    create_control(
        "BUTTON",
        text,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX,
        0,
        parent,
        id,
        x,
        y,
        width,
        height,
    )
}

unsafe fn layout_settings_controls(hwnd: Hwnd) {
    let snapshot = with_state(|state| {
        (
            state.settings_dpi,
            state.monitor_edits.clone(),
            state.grid_color_edit,
            state.selection_color_edit,
            state.selection_border_color_edit,
            state.run_startup_checkbox,
        )
    });

    let Some((
        dpi,
        monitor_edits,
        grid_color_edit,
        selection_color_edit,
        selection_border_color_edit,
        run_startup_checkbox,
    )) = snapshot
    else {
        return;
    };

    SetWindowPos(
        hwnd,
        0,
        0,
        0,
        SETTINGS_WIDTH,
        SETTINGS_HEIGHT,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    );

    for (index, edit) in monitor_edits.iter().enumerate() {
        let y = 82 + i32::try_from(index).unwrap_or_default() * 56;
        move_control(edit.columns_edit, dpi, 277, y, 40, 22);
        move_control(edit.rows_edit, dpi, 363, y, 40, 22);
    }

    move_control(grid_color_edit, dpi, 445, 86, 199, 22);
    move_control(selection_color_edit, dpi, 445, 136, 199, 22);
    move_control(selection_border_color_edit, dpi, 445, 187, 199, 22);
    move_control(
        GetDlgItem(hwnd, ID_RESET_COLORS as i32),
        dpi,
        445,
        223,
        228,
        28,
    );
    move_control(run_startup_checkbox, dpi, 445, 301, 160, 24);
}

unsafe fn move_control(hwnd: Hwnd, dpi: u32, x: i32, y: i32, width: i32, height: i32) {
    if hwnd == 0 {
        return;
    }

    SetWindowPos(
        hwnd,
        0,
        scale_value(dpi, x),
        scale_value(dpi, y),
        scale_value(dpi, width),
        scale_value(dpi, height),
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
}

#[allow(clippy::too_many_arguments)]
unsafe fn create_control(
    class_name: &str,
    text: &str,
    style: u32,
    ex_style: u32,
    parent: Hwnd,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Hwnd {
    let class_name = wide(class_name);
    let text = wide(text);
    CreateWindowExW(
        ex_style,
        class_name.as_ptr(),
        text.as_ptr(),
        style,
        x,
        y,
        width,
        height,
        parent,
        id as Hmenu,
        GetModuleHandleW(null()),
        null_mut(),
    )
}

fn apply_settings_from_window(hwnd: Hwnd) {
    let updated = with_state(|state| {
        for (index, edit) in state.monitor_edits.iter().copied().enumerate() {
            if let Some(monitor) = state.settings.monitors.get_mut(index) {
                monitor.columns =
                    clamp_grid_dimension(read_number(edit.columns_edit).unwrap_or(monitor.columns));
                monitor.rows =
                    clamp_grid_dimension(read_number(edit.rows_edit).unwrap_or(monitor.rows));
            }
        }

        if let Some(color) = read_valid_hex_color(state.grid_color_edit, DEFAULT_GRID_COLOR) {
            state.settings.grid_color = color;
        }
        if let Some(color) =
            read_valid_hex_color(state.selection_color_edit, DEFAULT_SELECTION_COLOR)
        {
            state.settings.selection_color = color;
        }
        if let Some(color) = read_valid_hex_color(
            state.selection_border_color_edit,
            DEFAULT_SELECTION_BORDER_COLOR,
        ) {
            state.settings.selection_border_color = color;
        }
        state.settings.run_on_startup = unsafe {
            SendMessageW(state.run_startup_checkbox, BM_GETCHECK, 0, 0) as usize == BST_CHECKED
        };

        let first_grid = state
            .settings
            .monitors
            .first()
            .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
            .unwrap_or_else(default_grid);
        state.grid = first_grid;
        state.settings.clone()
    });

    if let Some(settings) = updated {
        set_startup_enabled(settings.run_on_startup);
        save_settings(&settings);
    }

    unsafe {
        InvalidateRect(hwnd, null(), 1);
        SetForegroundWindow(hwnd);
    }
}

fn sync_settings_window(grid: GridSpec) {
    with_state(|state| state.syncing_settings_ui = true);

    let snapshot = with_state(|state| {
        (
            state.monitor_edits.clone(),
            state.settings.clone(),
            state.grid_color_edit,
            state.selection_color_edit,
            state.selection_border_color_edit,
            state.run_startup_checkbox,
        )
    });

    let Some((
        monitor_edits,
        settings,
        grid_color_edit,
        selection_color_edit,
        selection_border_color_edit,
        run_startup_checkbox,
    )) = snapshot
    else {
        with_state(|state| state.syncing_settings_ui = false);
        return;
    };

    unsafe {
        for (index, edit) in monitor_edits.iter().enumerate() {
            if let Some(monitor) = settings.monitors.get(index) {
                set_window_text(edit.columns_edit, &monitor.columns.to_string());
                set_window_text(edit.rows_edit, &monitor.rows.to_string());
            }
        }

        if grid_color_edit != 0 {
            set_window_text(grid_color_edit, &settings.grid_color);
        }
        if selection_color_edit != 0 {
            set_window_text(selection_color_edit, &settings.selection_color);
        }
        if selection_border_color_edit != 0 {
            set_window_text(
                selection_border_color_edit,
                &settings.selection_border_color,
            );
        }
        if run_startup_checkbox != 0 {
            SendMessageW(
                run_startup_checkbox,
                BM_SETCHECK,
                if settings.run_on_startup {
                    BST_CHECKED
                } else {
                    0
                },
                0,
            );
        }
    }

    with_state(|state| state.grid = grid);
    with_state(|state| state.syncing_settings_ui = false);
}

fn read_number(hwnd: Hwnd) -> Option<u16> {
    if hwnd == 0 {
        return None;
    }

    let mut buffer = [0_u16; 16];
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if len <= 0 {
        return None;
    }

    String::from_utf16_lossy(&buffer[..usize::try_from(len).ok()?])
        .trim()
        .parse()
        .ok()
}

fn read_text(hwnd: Hwnd) -> Option<String> {
    if hwnd == 0 {
        return None;
    }

    let mut buffer = [0_u16; 128];
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if len <= 0 {
        return None;
    }

    Some(
        String::from_utf16_lossy(&buffer[..usize::try_from(len).ok()?])
            .trim()
            .to_string(),
    )
}

fn read_valid_hex_color(hwnd: Hwnd, default: &str) -> Option<String> {
    let text = read_text(hwnd)?;
    if is_valid_hex_color(&text) {
        Some(normalize_hex_color(Some(&text), default))
    } else {
        None
    }
}

fn live_setting_changed(control_id: usize, notification: usize) -> bool {
    let is_edit_change = notification == EN_CHANGE
        && (control_id == ID_GRID_COLOR_EDIT
            || control_id == ID_SELECTION_COLOR_EDIT
            || control_id == ID_SELECTION_BORDER_EDIT
            || control_id >= ID_MONITOR_EDIT_BASE);
    let is_checkbox_click = notification == BN_CLICKED && control_id == ID_RUN_STARTUP;

    is_edit_change || is_checkbox_click
}

fn refresh_monitors() {
    let stored = with_state(|state| state.settings.clone()).unwrap_or_else(load_settings);
    let mut settings = merge_monitors(stored_settings_from_data(&stored), unsafe {
        enumerate_monitors()
    });
    settings.grid_color = stored.grid_color;
    settings.selection_color = stored.selection_color;
    settings.selection_border_color = stored.selection_border_color;
    settings.run_on_startup = startup_is_enabled();
    settings.is_dark_mode = stored.is_dark_mode;

    with_state(|state| {
        state.settings = settings.clone();
        state.grid = settings
            .monitors
            .first()
            .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
            .unwrap_or_else(default_grid);
    });

    let settings_hwnd = with_state(|state| state.settings_hwnd).unwrap_or_default();
    if settings_hwnd != 0 {
        sync_settings_window(
            settings
                .monitors
                .first()
                .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
                .unwrap_or_else(default_grid),
        );
        unsafe {
            InvalidateRect(settings_hwnd, null(), 1);
        }
    }
}

fn rebuild_settings_window_if_open() {
    let handles = with_state(|state| (state.main_hwnd, state.settings_hwnd)).unwrap_or_default();
    if handles.1 == 0 {
        return;
    }

    unsafe {
        DestroyWindow(handles.1);
        show_settings_window(handles.0);
    }
}

fn load_settings() -> SettingsData {
    let stored = load_stored_settings()
        .or_else(load_original_settings)
        .unwrap_or_else(default_stored_settings);
    merge_monitors(stored, unsafe { enumerate_monitors() })
}

fn default_stored_settings() -> StoredSettings {
    StoredSettings {
        monitors: Vec::new(),
        grid_color: DEFAULT_GRID_COLOR.to_string(),
        selection_color: DEFAULT_SELECTION_COLOR.to_string(),
        selection_border_color: DEFAULT_SELECTION_BORDER_COLOR.to_string(),
        run_on_startup: None,
        is_dark_mode: Some(true),
    }
}

fn stored_settings_from_data(settings: &SettingsData) -> StoredSettings {
    StoredSettings {
        monitors: settings
            .monitors
            .iter()
            .map(|monitor| StoredMonitorConfig {
                device_name: monitor.device_name.clone(),
                display_name: monitor.display_name.clone(),
                columns: monitor.columns,
                rows: monitor.rows,
            })
            .collect(),
        grid_color: settings.grid_color.clone(),
        selection_color: settings.selection_color.clone(),
        selection_border_color: settings.selection_border_color.clone(),
        run_on_startup: Some(settings.run_on_startup),
        is_dark_mode: Some(settings.is_dark_mode),
    }
}

fn merge_monitors(stored: StoredSettings, mut monitors: Vec<MonitorConfig>) -> SettingsData {
    if monitors.is_empty() {
        monitors.push(MonitorConfig {
            device_name: "DISPLAY".to_string(),
            display_name: "Primary Monitor".to_string(),
            work_rect: ScreenRect::new(0, 0, 1, 1),
            columns: 2,
            rows: 2,
        });
    }

    for monitor in &mut monitors {
        if let Some(stored_monitor) = stored
            .monitors
            .iter()
            .find(|candidate| candidate.device_name == monitor.device_name)
        {
            monitor.columns = clamp_grid_dimension(stored_monitor.columns);
            monitor.rows = clamp_grid_dimension(stored_monitor.rows);
            if monitor.display_name == monitor.device_name
                && !stored_monitor.display_name.is_empty()
            {
                monitor.display_name = stored_monitor.display_name.clone();
            }
        }
    }

    SettingsData {
        monitors,
        grid_color: normalize_hex_color(Some(&stored.grid_color), DEFAULT_GRID_COLOR),
        selection_color: normalize_hex_color(
            Some(&stored.selection_color),
            DEFAULT_SELECTION_COLOR,
        ),
        selection_border_color: normalize_hex_color(
            Some(&stored.selection_border_color),
            DEFAULT_SELECTION_BORDER_COLOR,
        ),
        run_on_startup: stored.run_on_startup.unwrap_or_else(startup_is_enabled),
        is_dark_mode: stored.is_dark_mode.unwrap_or(true),
    }
}

fn load_stored_settings() -> Option<StoredSettings> {
    let path = settings_path()
        .filter(|path| path.exists())
        .or_else(|| legacy_settings_path().filter(|path| path.exists()))?;
    let contents = fs::read_to_string(path).ok()?;
    let mut settings = default_stored_settings();

    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key.trim() {
            "grid_color" => settings.grid_color = value.trim().to_string(),
            "selection_color" => settings.selection_color = value.trim().to_string(),
            "selection_border_color" => settings.selection_border_color = value.trim().to_string(),
            "run_on_startup" => {
                settings.run_on_startup = Some(value.trim().eq_ignore_ascii_case("true"))
            }
            "is_dark_mode" => {
                settings.is_dark_mode = Some(value.trim().eq_ignore_ascii_case("true"))
            }
            "monitor" => {
                let parts: Vec<&str> = value.split('|').collect();
                if parts.len() >= 4 {
                    settings.monitors.push(StoredMonitorConfig {
                        device_name: parts[0].to_string(),
                        display_name: parts[1].to_string(),
                        columns: parts[2].parse().unwrap_or(2),
                        rows: parts[3].parse().unwrap_or(2),
                    });
                }
            }
            _ => {}
        }
    }

    Some(settings)
}

fn load_original_settings() -> Option<StoredSettings> {
    let contents = fs::read_to_string(original_settings_path()?).ok()?;
    let mut settings = default_stored_settings();

    if let Some(color) = extract_json_string(&contents, "GridColor") {
        settings.grid_color = argb_to_rgba_hex(&color, DEFAULT_GRID_COLOR);
    }
    if let Some(color) = extract_json_string(&contents, "SelectionColor") {
        settings.selection_color = argb_to_rgba_hex(&color, DEFAULT_SELECTION_COLOR);
    }
    if let Some(color) = extract_json_string(&contents, "SelectionBorderColor") {
        settings.selection_border_color = argb_to_rgba_hex(&color, DEFAULT_SELECTION_BORDER_COLOR);
    }
    settings.run_on_startup = extract_json_bool(&contents, "RunOnStartup");
    settings.is_dark_mode = extract_json_bool(&contents, "IsDarkMode").or(settings.is_dark_mode);

    let mut cursor = contents.as_str();
    while let Some(index) = cursor.find("\"DeviceName\"") {
        cursor = &cursor[index..];
        let block_end = cursor.find('}').unwrap_or(cursor.len());
        let block = &cursor[..block_end];
        if let Some(device_name) = extract_json_string(block, "DeviceName") {
            settings.monitors.push(StoredMonitorConfig {
                device_name,
                display_name: extract_json_string(block, "FriendlyName").unwrap_or_default(),
                columns: extract_json_number(block, "Columns").unwrap_or(2),
                rows: extract_json_number(block, "Rows").unwrap_or(2),
            });
        }
        cursor = &cursor[block_end..];
    }

    Some(settings)
}

fn save_settings(settings: &SettingsData) {
    let Some(path) = settings_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut contents = String::new();
    contents.push_str(&format!("grid_color={}\n", settings.grid_color));
    contents.push_str(&format!("selection_color={}\n", settings.selection_color));
    contents.push_str(&format!(
        "selection_border_color={}\n",
        settings.selection_border_color
    ));
    contents.push_str(&format!("run_on_startup={}\n", settings.run_on_startup));
    contents.push_str(&format!("is_dark_mode={}\n", settings.is_dark_mode));

    for monitor in &settings.monitors {
        contents.push_str(&format!(
            "monitor={}|{}|{}|{}\n",
            monitor.device_name, monitor.display_name, monitor.columns, monitor.rows
        ));
    }

    let _ = fs::write(path, contents);
}

fn extract_json_string(contents: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = contents.find(&needle)?;
    let after_key = &contents[start + needle.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    let after_quote = after_colon.strip_prefix('"')?;
    let end = after_quote.find('"')?;
    Some(after_quote[..end].replace("\\\\", "\\"))
}

fn extract_json_bool(contents: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let start = contents.find(&needle)?;
    let after_key = &contents[start + needle.len()..];
    let colon = after_key.find(':')?;
    let value = after_key[colon + 1..].trim_start();
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn extract_json_number(contents: &str, key: &str) -> Option<u16> {
    let needle = format!("\"{key}\"");
    let start = contents.find(&needle)?;
    let after_key = &contents[start + needle.len()..];
    let colon = after_key.find(':')?;
    let value = after_key[colon + 1..].trim_start();
    let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .map(|base| base.join(APP_DIR_NAME).join("settings.ini"))
}

fn legacy_settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .map(|base| base.join("BetterSnap").join("settings.ini"))
}

fn original_settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|base| base.join("TheGriddler").join("settings.json"))
}

fn startup_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from).map(|base| {
        base.join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join(STARTUP_SCRIPT_NAME)
    })
}

fn legacy_startup_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from).map(|base| {
        base.join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join("BetterSnap.cmd")
    })
}

fn startup_is_enabled() -> bool {
    startup_path().is_some_and(|path| path.exists())
        || legacy_startup_path().is_some_and(|path| path.exists())
}

fn set_startup_enabled(enabled: bool) {
    let Some(path) = startup_path() else {
        return;
    };

    if enabled {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(exe) = env::current_exe() {
            let contents = format!("@echo off\r\nstart \"\" \"{}\"\r\n", exe.display());
            let _ = fs::write(path, contents);
        }
        if let Some(legacy_path) = legacy_startup_path() {
            let _ = fs::remove_file(legacy_path);
        }
    } else {
        let _ = fs::remove_file(path);
        if let Some(legacy_path) = legacy_startup_path() {
            let _ = fs::remove_file(legacy_path);
        }
    }
}

fn clamp_grid_dimension(value: u16) -> u16 {
    value.clamp(MIN_GRID_DIMENSION, MAX_GRID_DIMENSION)
}

fn normalize_hex_color(value: Option<&str>, default: &str) -> String {
    let Some(value) = value else {
        return default.to_string();
    };

    let value = value.trim().to_ascii_uppercase();
    if !is_valid_hex_color(&value) {
        return default.to_string();
    }

    match value.as_str() {
        LEGACY_DEFAULT_GRID_COLOR => DEFAULT_GRID_COLOR.to_string(),
        LEGACY_DEFAULT_SELECTION_COLOR => DEFAULT_SELECTION_COLOR.to_string(),
        LEGACY_DEFAULT_SELECTION_BORDER_COLOR => DEFAULT_SELECTION_BORDER_COLOR.to_string(),
        _ => value,
    }
}

fn argb_to_rgba_hex(value: &str, default: &str) -> String {
    let value = value.trim().to_ascii_uppercase();
    if !is_valid_hex_color(&value) {
        return default.to_string();
    }

    if value.len() == 9 {
        format!("#{}{}", &value[3..9], &value[1..3],)
    } else {
        value
    }
}

fn is_valid_hex_color(value: &str) -> bool {
    let value = value.trim();
    let valid_len = value.len() == 7 || value.len() == 9;
    valid_len && value.starts_with('#') && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

fn colorref_from_hex(value: &str) -> Option<u32> {
    let normalized = normalize_hex_color(Some(value), DEFAULT_GRID_COLOR);
    let hex = normalized.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(rgb(red, green, blue))
}

fn rgba_from_hex(value: &str, default: &str) -> RgbaColor {
    let normalized = normalize_hex_color(Some(value), default);
    let hex = normalized.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let green = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let blue = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    let alpha = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };

    RgbaColor {
        red,
        green,
        blue,
        alpha,
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
        "Snapdragin' is running.\n\nDrag a window with the left mouse button, right-click to open the grid, move across the cells, then right-click again or release left click to snap.",
    );
}

unsafe fn cleanup(hwnd: Hwnd) {
    remove_tray_icon(hwnd);

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

#[derive(Debug, Clone, Copy)]
enum ColorTarget {
    Grid,
    Selection,
    Border,
}

unsafe fn paint_settings(hwnd: Hwnd) {
    let mut paint: Paintstruct = zeroed();
    let hdc = BeginPaint(hwnd, &mut paint);
    if hdc == 0 {
        return;
    }

    let mut client: Rect = zeroed();
    GetClientRect(hwnd, &mut client);
    let dpi = with_state(|state| state.settings_dpi).unwrap_or(96);
    let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
    let background = if dark {
        rgb(30, 30, 30)
    } else {
        rgb(243, 243, 243)
    };
    let card = if dark {
        rgb(45, 45, 45)
    } else {
        rgb(255, 255, 255)
    };

    draw_filled_rect(hdc, client, background);
    SetBkMode(hdc, TRANSPARENT_BK);

    let icon = with_state(|state| state.app_icon).unwrap_or_default();
    if icon != 0 {
        DrawIconEx(
            hdc,
            scale_value(dpi, 16),
            scale_value(dpi, 11),
            icon,
            scale_value(dpi, 20),
            scale_value(dpi, 20),
            0,
            0,
            DI_NORMAL,
        );
    }

    draw_text_line(hdc, APP_NAME, dpi, 47, 16, 150, 18, true);
    draw_text_line(hdc, "X", dpi, 675, 15, 16, 18, true);
    draw_text_line(hdc, "\u{1F4A1}", dpi, 645, 13, 22, 22, false);

    let settings = with_state(|state| state.settings.clone()).unwrap_or_else(load_settings);

    draw_group(
        hdc,
        scaled_rect(dpi, 16, 50, 424, 484),
        "Monitor Grid Configuration",
        dpi,
    );
    for (index, monitor) in settings.monitors.iter().enumerate() {
        if index >= 7 {
            break;
        }

        let y = 67 + i32::try_from(index).unwrap_or_default() * 56;
        draw_filled_rect(hdc, scaled_rect(dpi, 28, y, 412, y + 41), card);
        draw_text_line(hdc, &monitor.display_name, dpi, 37, y + 14, 205, 16, true);
        draw_text_line(hdc, "Cols:", dpi, 247, y + 15, 40, 16, false);
        draw_text_line(hdc, "Rows:", dpi, 326, y + 15, 40, 16, false);
    }

    draw_group(
        hdc,
        scaled_rect(dpi, 434, 50, 684, 261),
        "Visual Settings",
        dpi,
    );
    draw_text_line(hdc, "Grid Lines", dpi, 445, 73, 120, 16, false);
    draw_text_line(hdc, "Selection Area", dpi, 445, 123, 120, 16, false);
    draw_text_line(hdc, "Selection Border", dpi, 445, 174, 130, 16, false);
    draw_swatch(
        hdc,
        dpi,
        649,
        86,
        colorref_from_hex(&settings.grid_color).unwrap_or(rgb(255, 255, 255)),
    );
    draw_swatch(
        hdc,
        dpi,
        649,
        136,
        colorref_from_hex(&settings.selection_color).unwrap_or(rgb(0, 255, 255)),
    );
    draw_swatch(
        hdc,
        dpi,
        649,
        187,
        colorref_from_hex(&settings.selection_border_color).unwrap_or(rgb(0, 255, 255)),
    );

    draw_group(
        hdc,
        scaled_rect(dpi, 434, 279, 684, 329),
        "App Settings",
        dpi,
    );
    draw_group(hdc, scaled_rect(dpi, 434, 348, 684, 484), "Info", dpi);
    draw_wrapped_text(
        hdc,
        "While dragging a window by holding left click, right click in the grid section you want to resize from, continue dragging left click to the grid section you want to resize to and press right click again or let go of left click.",
        scaled_rect(dpi, 440, 360, 678, 478),
    );

    EndPaint(hwnd, &paint);
}

unsafe fn draw_group(hdc: Hdc, rect: Rect, title: &str, dpi: u32) {
    let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
    let border = if dark {
        rgb(220, 220, 220)
    } else {
        rgb(90, 90, 90)
    };
    let background = if dark {
        rgb(30, 30, 30)
    } else {
        rgb(243, 243, 243)
    };
    let pen = CreatePen(PS_SOLID, scale_value(dpi, 1).max(1), border);
    let previous = SelectObject(hdc, pen);
    let hollow = GetStockObject(5);
    let previous_brush = SelectObject(hdc, hollow);
    Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
    SelectObject(hdc, previous_brush);
    SelectObject(hdc, previous);
    DeleteObject(pen);

    let title_width = scale_value(
        dpi,
        i32::try_from(title.chars().count()).unwrap_or_default() * 7 + 16,
    );
    draw_filled_rect(
        hdc,
        Rect::new(
            rect.left + scale_value(dpi, 7),
            rect.top - scale_value(dpi, 8),
            rect.left + scale_value(dpi, 7) + title_width,
            rect.top + scale_value(dpi, 8),
        ),
        background,
    );
    draw_text_line(
        hdc,
        title,
        dpi,
        unscale_value(dpi, rect.left) + 10,
        unscale_value(dpi, rect.top) - 8,
        unscale_value(dpi, title_width),
        16,
        false,
    );
}

unsafe fn draw_filled_rect(hdc: Hdc, rect: Rect, color: u32) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, &rect, brush);
    DeleteObject(brush);
}

unsafe fn draw_swatch(hdc: Hdc, dpi: u32, x: i32, y: i32, color: u32) {
    draw_filled_rect(hdc, scaled_rect(dpi, x, y, x + 24, y + 22), color);
}

#[allow(clippy::too_many_arguments)]
unsafe fn draw_text_line(
    hdc: Hdc,
    text: &str,
    dpi: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    bright: bool,
) {
    let mut rect = scaled_rect(dpi, x, y, x + width, y + height);
    let is_close_button = text == "X";
    let text = wide(text);
    let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
    let color = if is_close_button {
        rgb(255, 64, 64)
    } else if dark && bright {
        rgb(255, 255, 255)
    } else if dark {
        rgb(238, 238, 238)
    } else if bright {
        rgb(0, 0, 0)
    } else {
        rgb(35, 35, 35)
    };
    SetTextColor(hdc, color);
    DrawTextW(hdc, text.as_ptr(), -1, &mut rect, DT_LEFT | DT_TOP);
}

unsafe fn draw_wrapped_text(hdc: Hdc, text: &str, mut rect: Rect) {
    let text = wide(text);
    let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
    SetTextColor(
        hdc,
        if dark {
            rgb(255, 255, 255)
        } else {
            rgb(35, 35, 35)
        },
    );
    DrawTextW(
        hdc,
        text.as_ptr(),
        -1,
        &mut rect,
        DT_LEFT | DT_TOP | DT_WORDBREAK,
    );
}

fn settings_control_brush() -> Lresult {
    static DARK_BRUSH: OnceLock<Hbrush> = OnceLock::new();
    static LIGHT_BRUSH: OnceLock<Hbrush> = OnceLock::new();
    let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
    let brush = if dark {
        DARK_BRUSH.get_or_init(|| unsafe { CreateSolidBrush(rgb(45, 45, 45)) })
    } else {
        LIGHT_BRUSH.get_or_init(|| unsafe { CreateSolidBrush(rgb(255, 255, 255)) })
    };

    *brush as Lresult
}

fn reset_colors_from_window(hwnd: Hwnd) {
    let handles = with_state(|state| {
        (
            state.grid_color_edit,
            state.selection_color_edit,
            state.selection_border_color_edit,
        )
    });

    if let Some((grid, selection, border)) = handles {
        unsafe {
            set_window_text(grid, DEFAULT_GRID_COLOR);
            set_window_text(selection, DEFAULT_SELECTION_COLOR);
            set_window_text(border, DEFAULT_SELECTION_BORDER_COLOR);
        }
        apply_settings_from_window(hwnd);
    }
}

fn toggle_theme(hwnd: Hwnd) {
    let settings = with_state(|state| {
        state.settings.is_dark_mode = !state.settings.is_dark_mode;
        state.settings.clone()
    });

    if let Some(settings) = settings {
        save_settings(&settings);
    }

    unsafe {
        InvalidateRect(hwnd, null(), 1);
        if let Some(handles) = with_state(|state| {
            let mut handles = Vec::with_capacity(state.monitor_edits.len() * 2 + 4);
            for edit in &state.monitor_edits {
                handles.push(edit.columns_edit);
                handles.push(edit.rows_edit);
            }
            handles.push(state.grid_color_edit);
            handles.push(state.selection_color_edit);
            handles.push(state.selection_border_color_edit);
            handles.push(state.run_startup_checkbox);
            handles
        }) {
            for control in handles {
                if control != 0 {
                    InvalidateRect(control, null(), 1);
                }
            }
        }
    }
}

fn swatch_hit(x: i32, y: i32, index: usize) -> bool {
    let top = match index {
        0 => 86,
        1 => 136,
        2 => 187,
        _ => return false,
    };

    (649..=673).contains(&x) && (top..=top + 22).contains(&y)
}

fn pick_color(hwnd: Hwnd, target: ColorTarget) {
    let edit = with_state(|state| match target {
        ColorTarget::Grid => state.grid_color_edit,
        ColorTarget::Selection => state.selection_color_edit,
        ColorTarget::Border => state.selection_border_color_edit,
    })
    .unwrap_or_default();

    if edit == 0 {
        return;
    }

    let default = match target {
        ColorTarget::Grid => DEFAULT_GRID_COLOR,
        ColorTarget::Selection => DEFAULT_SELECTION_COLOR,
        ColorTarget::Border => DEFAULT_SELECTION_BORDER_COLOR,
    };
    let current = read_text(edit).unwrap_or_else(|| match target {
        ColorTarget::Grid => DEFAULT_GRID_COLOR.to_string(),
        ColorTarget::Selection => DEFAULT_SELECTION_COLOR.to_string(),
        ColorTarget::Border => DEFAULT_SELECTION_BORDER_COLOR.to_string(),
    });
    let normalized = normalize_hex_color(Some(&current), default);
    let alpha = if normalized.len() == 9 {
        normalized[7..9].to_string()
    } else {
        "FF".to_string()
    };

    let mut custom_colors = [0_u32; 16];
    let mut choose = Choosecolorw {
        l_struct_size: size_of::<Choosecolorw>() as u32,
        hwnd_owner: hwnd,
        h_instance: 0,
        rgb_result: colorref_from_hex(&normalized).unwrap_or_default(),
        lp_cust_colors: custom_colors.as_mut_ptr(),
        flags: CC_RGBINIT | CC_FULLOPEN,
        l_cust_data: 0,
        lpfn_hook: None,
        lp_template_name: null(),
    };

    if unsafe { ChooseColorW(&mut choose) == 0 } {
        return;
    }

    let red = choose.rgb_result & 0xFF;
    let green = (choose.rgb_result >> 8) & 0xFF;
    let blue = (choose.rgb_result >> 16) & 0xFF;
    let value = format!("#{red:02X}{green:02X}{blue:02X}{alpha}");

    unsafe {
        set_window_text(edit, &value);
    }
    apply_settings_from_window(hwnd);
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

fn target_window(point: ScreenPoint) -> Option<Hwnd> {
    unsafe {
        let current_pid = GetCurrentProcessId();
        let foreground = GetForegroundWindow();
        if foreground != 0 && window_process_id(foreground) != current_pid {
            let root = GetAncestor(foreground, GA_ROOT);
            if root != 0 {
                return Some(root);
            }
        }

        let hwnd = WindowFromPoint(Point {
            x: point.x,
            y: point.y,
        });
        if hwnd == 0 {
            return None;
        }

        let root = GetAncestor(hwnd, GA_ROOT);
        if root == 0 || window_process_id(root) == current_pid {
            return None;
        }

        Some(root)
    }
}

fn window_is_in_move_size_loop(hwnd: Hwnd) -> bool {
    unsafe {
        let thread_id = GetWindowThreadProcessId(hwnd, null_mut());
        let mut info: Guithreadinfo = zeroed();
        info.cb_size = size_of::<Guithreadinfo>() as u32;

        if GetGUIThreadInfo(thread_id, &mut info) == 0 {
            return true;
        }

        (info.flags & GUI_INMOVESIZE) != 0 || info.hwnd_move_size != 0
    }
}

unsafe fn break_drag_loop(hwnd: Hwnd) {
    send_drag_cancel_message(hwnd, WM_CANCELMODE);
    send_drag_cancel_message(hwnd, WM_LBUTTONUP);
    ReleaseCapture();
}

unsafe fn send_drag_cancel_message(hwnd: Hwnd, message: u32) {
    let mut result = 0;
    let sent = SendMessageTimeoutW(
        hwnd,
        message,
        0,
        0,
        SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
        DRAG_CANCEL_TIMEOUT_MS,
        &mut result,
    );

    if sent == 0 {
        PostMessageW(hwnd, message, 0, 0);
    }
}

unsafe fn monitor_info_from_point(point: ScreenPoint) -> MonitorConfig {
    let monitor = MonitorFromPoint(
        Point {
            x: point.x,
            y: point.y,
        },
        MONITOR_DEFAULTTONEAREST,
    );

    monitor_config_from_handle(monitor).unwrap_or_else(|| MonitorConfig {
        device_name: "DISPLAY".to_string(),
        display_name: "Primary Monitor".to_string(),
        work_rect: ScreenRect::new(point.x, point.y, 1, 1),
        columns: 2,
        rows: 2,
    })
}

unsafe fn enumerate_monitors() -> Vec<MonitorConfig> {
    let mut monitors = Vec::new();
    EnumDisplayMonitors(
        0,
        null(),
        Some(enum_monitor_proc),
        &mut monitors as *mut _ as Lparam,
    );

    if monitors.is_empty() {
        let point = ScreenPoint::new(0, 0);
        monitors.push(monitor_info_from_point(point));
    }

    monitors
}

unsafe extern "system" fn enum_monitor_proc(
    monitor: Hmonitor,
    _hdc: Hdc,
    _rect: *mut Rect,
    data: Lparam,
) -> Bool {
    let monitors = &mut *(data as *mut Vec<MonitorConfig>);
    if let Some(config) = monitor_config_from_handle(monitor) {
        monitors.push(config);
    }
    1
}

unsafe fn monitor_config_from_handle(monitor: Hmonitor) -> Option<MonitorConfig> {
    if monitor == 0 {
        return None;
    }

    let mut info: Monitorinfoexw = zeroed();
    info.cb_size = size_of::<Monitorinfoexw>() as u32;
    if GetMonitorInfoW(
        monitor,
        &mut info as *mut Monitorinfoexw as *mut Monitorinfo,
    ) == 0
    {
        return None;
    }

    let device_name = string_from_wide_z(&info.sz_device);
    let display_name = friendly_monitor_name(&device_name).unwrap_or_else(|| device_name.clone());

    Some(MonitorConfig {
        device_name,
        display_name,
        work_rect: rect_to_screen_rect(info.rc_work),
        columns: 2,
        rows: 2,
    })
}

unsafe fn friendly_monitor_name(device_name: &str) -> Option<String> {
    for index in 0..32 {
        let mut adapter: DisplayDeviceW = zeroed();
        adapter.cb = size_of::<DisplayDeviceW>() as u32;
        if EnumDisplayDevicesW(null(), index, &mut adapter, 0) == 0 {
            break;
        }

        let adapter_name = string_from_wide_z(&adapter.device_name);
        if adapter_name != device_name {
            continue;
        }

        let mut monitor: DisplayDeviceW = zeroed();
        monitor.cb = size_of::<DisplayDeviceW>() as u32;
        let adapter_name_w = wide(&adapter_name);
        if EnumDisplayDevicesW(adapter_name_w.as_ptr(), 0, &mut monitor, 0) != 0 {
            let friendly = string_from_wide_z(&monitor.device_string);
            if !friendly.is_empty() {
                return Some(friendly);
            }
        }
    }

    None
}

fn rect_to_screen_rect(rect: Rect) -> ScreenRect {
    ScreenRect::new(
        rect.left,
        rect.top,
        u32::try_from(rect.right - rect.left).unwrap_or(1),
        u32::try_from(rect.bottom - rect.top).unwrap_or(1),
    )
}

fn window_process_id(hwnd: Hwnd) -> u32 {
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        pid
    }
}

fn left_button_is_down() -> bool {
    unsafe { (GetAsyncKeyState(VK_LBUTTON) & i16::MIN) != 0 }
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Size {
    cx: i32,
    cy: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
    time: Dword,
    pt: Point,
    l_private: Dword,
}

#[repr(C)]
struct Wndclassexw {
    cb_size: Uint,
    style: Uint,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Hinstance,
    h_icon: Hicon,
    h_cursor: Hcursor,
    hbr_background: Hbrush,
    lpsz_menu_name: Pcwstr,
    lpsz_class_name: Pcwstr,
    h_icon_sm: Hicon,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Msllhookstruct {
    pt: Point,
    mouse_data: Dword,
    flags: Dword,
    time: Dword,
    dw_extra_info: usize,
}

#[repr(C)]
struct Guithreadinfo {
    cb_size: Dword,
    flags: Dword,
    hwnd_active: Hwnd,
    hwnd_focus: Hwnd,
    hwnd_capture: Hwnd,
    hwnd_menu_owner: Hwnd,
    hwnd_move_size: Hwnd,
    hwnd_caret: Hwnd,
    rc_caret: Rect,
}

#[repr(C)]
struct Monitorinfo {
    cb_size: Dword,
    rc_monitor: Rect,
    rc_work: Rect,
    flags: Dword,
}

#[repr(C)]
struct Monitorinfoexw {
    cb_size: Dword,
    rc_monitor: Rect,
    rc_work: Rect,
    flags: Dword,
    sz_device: [u16; 32],
}

#[repr(C)]
struct DisplayDeviceW {
    cb: Dword,
    device_name: [u16; 32],
    device_string: [u16; 128],
    state_flags: Dword,
    device_id: [u16; 128],
    device_key: [u16; 128],
}

#[repr(C)]
struct Paintstruct {
    hdc: Hdc,
    f_erase: Bool,
    rc_paint: Rect,
    f_restore: Bool,
    f_inc_update: Bool,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct Notifyicondataw {
    cb_size: Dword,
    h_wnd: Hwnd,
    u_id: Uint,
    u_flags: Uint,
    u_callback_message: Uint,
    h_icon: Hicon,
    sz_tip: [u16; 128],
    dw_state: Dword,
    dw_state_mask: Dword,
    sz_info: [u16; 256],
    u_version: Uint,
    sz_info_title: [u16; 64],
    dw_info_flags: Dword,
    guid_item: Guid,
    h_balloon_icon: Hicon,
}

#[repr(C)]
struct Choosecolorw {
    l_struct_size: Dword,
    hwnd_owner: Hwnd,
    h_instance: Hwnd,
    rgb_result: Dword,
    lp_cust_colors: *mut Dword,
    flags: Dword,
    l_cust_data: Lparam,
    lpfn_hook: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Uint>,
    lp_template_name: Pcwstr,
}

#[repr(C)]
struct Bitmapinfoheader {
    bi_size: Dword,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: Dword,
    bi_size_image: Dword,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: Dword,
    bi_clr_important: Dword,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rgbquad {
    rgb_blue: u8,
    rgb_green: u8,
    rgb_red: u8,
    rgb_reserved: u8,
}

#[repr(C)]
struct Bitmapinfo {
    bmi_header: Bitmapinfoheader,
    bmi_colors: [Rgbquad; 1],
}

#[repr(C)]
struct Blendfunction {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

#[link(name = "user32")]
#[link(name = "gdi32")]
#[link(name = "shell32")]
#[link(name = "comdlg32")]
#[link(name = "shcore")]
unsafe extern "system" {
    fn AppendMenuW(h_menu: Hmenu, u_flags: Uint, u_id_new_item: usize, lp_new_item: Pcwstr)
        -> Bool;
    fn BeginPaint(hwnd: Hwnd, lp_paint: *mut Paintstruct) -> Hdc;
    fn CallNextHookEx(hhk: Hhook, n_code: i32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn CreateDIBSection(
        hdc: Hdc,
        pbmi: *const Bitmapinfo,
        usage: Uint,
        ppv_bits: *mut *mut c_void,
        h_section: isize,
        offset: Dword,
    ) -> Hbitmap;
    fn CreatePen(style: i32, width: i32, color: u32) -> Hpen;
    fn CreatePopupMenu() -> Hmenu;
    fn CreateSolidBrush(color: u32) -> Hbrush;
    fn CreateWindowExW(
        dw_ex_style: Dword,
        lp_class_name: Pcwstr,
        lp_window_name: Pcwstr,
        dw_style: Dword,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hwnd_parent: Hwnd,
        h_menu: Hmenu,
        h_instance: Hinstance,
        lp_param: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn DeleteDC(hdc: Hdc) -> Bool;
    fn DeleteObject(ho: Hgdiobj) -> Bool;
    fn DestroyMenu(h_menu: Hmenu) -> Bool;
    fn DestroyWindow(hwnd: Hwnd) -> Bool;
    fn DispatchMessageW(lp_msg: *const Msg) -> Lresult;
    fn DrawIconEx(
        hdc: Hdc,
        x_left: i32,
        y_top: i32,
        hicon: Hicon,
        cx_width: i32,
        cy_width: i32,
        istep_if_ani_cur: Uint,
        hbr_flicker_free_draw: Hbrush,
        di_flags: Uint,
    ) -> Bool;
    fn DrawTextW(hdc: Hdc, text: Pcwstr, text_len: i32, rect: *mut Rect, format: Uint) -> i32;
    fn EnumDisplayDevicesW(
        lp_device: Pcwstr,
        i_dev_num: Dword,
        lp_display_device: *mut DisplayDeviceW,
        dw_flags: Dword,
    ) -> Bool;
    fn EnumDisplayMonitors(
        hdc: Hdc,
        clip_rect: *const Rect,
        callback: Option<unsafe extern "system" fn(Hmonitor, Hdc, *mut Rect, Lparam) -> Bool>,
        data: Lparam,
    ) -> Bool;
    fn EndPaint(hwnd: Hwnd, lp_paint: *const Paintstruct) -> Bool;
    fn FillRect(hdc: Hdc, rect: *const Rect, hbr: Hbrush) -> i32;
    fn GetAncestor(hwnd: Hwnd, ga_flags: Uint) -> Hwnd;
    fn GetAsyncKeyState(vkey: i32) -> i16;
    fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    fn GetCursorPos(point: *mut Point) -> Bool;
    fn GetDC(hwnd: Hwnd) -> Hdc;
    fn GetDlgItem(hwnd: Hwnd, id: i32) -> Hwnd;
    fn GetDpiForWindow(hwnd: Hwnd) -> Uint;
    fn GetForegroundWindow() -> Hwnd;
    fn GetGUIThreadInfo(id_thread: Dword, gui: *mut Guithreadinfo) -> Bool;
    fn GetMessageW(
        lp_msg: *mut Msg,
        hwnd: Hwnd,
        msg_filter_min: Uint,
        msg_filter_max: Uint,
    ) -> Bool;
    fn GetModuleHandleW(lp_module_name: Pcwstr) -> Hinstance;
    fn GetMonitorInfoW(hmonitor: Hmonitor, info: *mut Monitorinfo) -> Bool;
    fn GetStockObject(index: i32) -> Hgdiobj;
    fn GetWindowDpiAwarenessContext(hwnd: Hwnd) -> isize;
    fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: i32) -> i32;
    fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut Dword) -> Dword;
    fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
    fn LoadCursorW(hinstance: Hinstance, cursor_name: Pcwstr) -> Hcursor;
    fn LoadIconW(hinstance: Hinstance, icon_name: Pcwstr) -> Hicon;
    fn MessageBoxW(hwnd: Hwnd, text: Pcwstr, caption: Pcwstr, flags: Uint) -> i32;
    fn MonitorFromPoint(point: Point, flags: Dword) -> Hmonitor;
    fn PostMessageW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Bool;
    fn PostQuitMessage(exit_code: i32);
    fn Rectangle(hdc: Hdc, left: i32, top: i32, right: i32, bottom: i32) -> Bool;
    fn RegisterClassExW(wnd_class: *const Wndclassexw) -> u16;
    fn ReleaseCapture() -> Bool;
    fn ReleaseDC(hwnd: Hwnd, hdc: Hdc) -> i32;
    fn SelectObject(hdc: Hdc, object: Hgdiobj) -> Hgdiobj;
    fn SendMessageTimeoutW(
        hwnd: Hwnd,
        msg: Uint,
        wparam: Wparam,
        lparam: Lparam,
        flags: Uint,
        timeout: Uint,
        result: *mut Lparam,
    ) -> Lresult;
    fn SendMessageW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn SetBkColor(hdc: Hdc, color: u32) -> u32;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetForegroundWindow(hwnd: Hwnd) -> Bool;
    fn SetProcessDpiAwarenessContext(value: isize) -> Bool;
    fn SetWindowTextW(hwnd: Hwnd, text: Pcwstr) -> Bool;
    fn SetTextColor(hdc: Hdc, color: u32) -> u32;
    fn SetWindowPos(
        hwnd: Hwnd,
        hwnd_insert_after: Hwnd,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: Uint,
    ) -> Bool;
    fn SetWindowsHookExW(
        hook: i32,
        proc: Option<unsafe extern "system" fn(i32, Wparam, Lparam) -> Lresult>,
        hmod: Hinstance,
        thread_id: Dword,
    ) -> Hhook;
    fn Shell_NotifyIconW(message: Dword, data: *mut Notifyicondataw) -> Bool;
    fn ShowWindow(hwnd: Hwnd, cmd_show: i32) -> Bool;
    fn TrackPopupMenu(
        menu: Hmenu,
        flags: Uint,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: Hwnd,
        rect: *const Rect,
    ) -> i32;
    fn TranslateMessage(lp_msg: *const Msg) -> Bool;
    fn UnhookWindowsHookEx(hhk: Hhook) -> Bool;
    fn UpdateLayeredWindow(
        hwnd: Hwnd,
        hdc_dst: Hdc,
        ppt_dst: *const Point,
        psize: *const Size,
        hdc_src: Hdc,
        ppt_src: *const Point,
        cr_key: Dword,
        pblend: *const Blendfunction,
        dw_flags: Dword,
    ) -> Bool;
    fn WindowFromPoint(point: Point) -> Hwnd;
    fn GetCurrentProcessId() -> Dword;
    fn ChooseColorW(choose_color: *mut Choosecolorw) -> Bool;
    fn GetAwarenessFromDpiAwarenessContext(value: isize) -> i32;
    fn GetDpiForMonitor(
        hmonitor: Hmonitor,
        dpi_type: i32,
        dpi_x: *mut Uint,
        dpi_y: *mut Uint,
    ) -> i32;
}
