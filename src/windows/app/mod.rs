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

use super::ffi::*;

mod desktop;
mod drag;
mod overlay;
mod runtime;
mod settings;
mod snap;
mod state;
mod tray;

pub(crate) use runtime::run;

use state::{
    AppState, MonitorConfig, MonitorEdit, PendingSnap, RgbaColor, SettingsData,
    StoredMonitorConfig, StoredSettings,
};

const APP_NAME: &str = "Snapdragin";
const APP_DIR_NAME: &str = "Snapdragin";
const STARTUP_SHORTCUT_NAME: &str = "Snapdragin.lnk";
const MAIN_CLASS: &str = "SnapdraginMainWindow";
const OVERLAY_CLASS: &str = "SnapdraginOverlayWindow";
const SETTINGS_CLASS: &str = "SnapdraginSettingsWindow";
const APP_ICON_ID: usize = 101;

const ID_TRAY: u32 = 1;
const WM_TRAYICON: u32 = WM_USER + 1;
const WM_APPLY_SNAP: u32 = WM_USER + 2;

const ID_SETTINGS: usize = 1080;
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
const SETTINGS_WIDTH: i32 = 700;
const SETTINGS_HEIGHT: i32 = 500;

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

unsafe fn set_window_text(hwnd: Hwnd, text: &str) {
    let text = wide(text);
    SetWindowTextW(hwnd, text.as_ptr());
}

fn default_grid() -> GridSpec {
    GridSpec::new(2, 2).expect("default grid is valid")
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
