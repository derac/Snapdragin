use std::ffi::c_void;

pub(crate) type Bool = i32;
pub(crate) type Dword = u32;
pub(crate) type Hbitmap = isize;
pub(crate) type Hbrush = isize;
pub(crate) type Hcursor = isize;
pub(crate) type Hdc = isize;
pub(crate) type Hgdiobj = isize;
pub(crate) type Hhook = isize;
pub(crate) type Hicon = isize;
pub(crate) type Hinstance = isize;
pub(crate) type Hmenu = isize;
pub(crate) type Hmonitor = isize;
pub(crate) type Hpen = isize;
pub(crate) type Hrgn = isize;
pub(crate) type Hwnd = isize;
pub(crate) type Lparam = isize;
pub(crate) type Lresult = isize;
pub(crate) type Pcwstr = *const u16;
pub(crate) type Uint = u32;
pub(crate) type Wparam = usize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Rect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

impl Rect {
    pub(crate) const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
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
pub(crate) struct Size {
    pub(crate) cx: i32,
    pub(crate) cy: i32,
}

#[repr(C)]
pub(crate) struct Msg {
    pub(crate) hwnd: Hwnd,
    pub(crate) message: Uint,
    pub(crate) w_param: Wparam,
    pub(crate) l_param: Lparam,
    pub(crate) time: Dword,
    pub(crate) pt: Point,
    pub(crate) l_private: Dword,
}

#[repr(C)]
pub(crate) struct Wndclassexw {
    pub(crate) cb_size: Uint,
    pub(crate) style: Uint,
    pub(crate) lpfn_wnd_proc:
        Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
    pub(crate) cb_cls_extra: i32,
    pub(crate) cb_wnd_extra: i32,
    pub(crate) h_instance: Hinstance,
    pub(crate) h_icon: Hicon,
    pub(crate) h_cursor: Hcursor,
    pub(crate) hbr_background: Hbrush,
    pub(crate) lpsz_menu_name: Pcwstr,
    pub(crate) lpsz_class_name: Pcwstr,
    pub(crate) h_icon_sm: Hicon,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Msllhookstruct {
    pub(crate) pt: Point,
    pub(crate) mouse_data: Dword,
    pub(crate) flags: Dword,
    pub(crate) time: Dword,
    pub(crate) dw_extra_info: usize,
}

#[repr(C)]
pub(crate) struct Guithreadinfo {
    pub(crate) cb_size: Dword,
    pub(crate) flags: Dword,
    pub(crate) hwnd_active: Hwnd,
    pub(crate) hwnd_focus: Hwnd,
    pub(crate) hwnd_capture: Hwnd,
    pub(crate) hwnd_menu_owner: Hwnd,
    pub(crate) hwnd_move_size: Hwnd,
    pub(crate) hwnd_caret: Hwnd,
    pub(crate) rc_caret: Rect,
}

#[repr(C)]
pub(crate) struct Monitorinfo {
    pub(crate) cb_size: Dword,
    pub(crate) rc_monitor: Rect,
    pub(crate) rc_work: Rect,
    pub(crate) flags: Dword,
}

#[repr(C)]
pub(crate) struct Monitorinfoexw {
    pub(crate) cb_size: Dword,
    pub(crate) rc_monitor: Rect,
    pub(crate) rc_work: Rect,
    pub(crate) flags: Dword,
    pub(crate) sz_device: [u16; 32],
}

#[repr(C)]
pub(crate) struct DisplayDeviceW {
    pub(crate) cb: Dword,
    pub(crate) device_name: [u16; 32],
    pub(crate) device_string: [u16; 128],
    pub(crate) state_flags: Dword,
    pub(crate) device_id: [u16; 128],
    pub(crate) device_key: [u16; 128],
}

#[repr(C)]
pub(crate) struct Paintstruct {
    pub(crate) hdc: Hdc,
    pub(crate) f_erase: Bool,
    pub(crate) rc_paint: Rect,
    pub(crate) f_restore: Bool,
    pub(crate) f_inc_update: Bool,
    pub(crate) rgb_reserved: [u8; 32],
}

#[repr(C)]
pub(crate) struct Guid {
    pub(crate) data1: u32,
    pub(crate) data2: u16,
    pub(crate) data3: u16,
    pub(crate) data4: [u8; 8],
}

#[repr(C)]
pub(crate) struct Notifyicondataw {
    pub(crate) cb_size: Dword,
    pub(crate) h_wnd: Hwnd,
    pub(crate) u_id: Uint,
    pub(crate) u_flags: Uint,
    pub(crate) u_callback_message: Uint,
    pub(crate) h_icon: Hicon,
    pub(crate) sz_tip: [u16; 128],
    pub(crate) dw_state: Dword,
    pub(crate) dw_state_mask: Dword,
    pub(crate) sz_info: [u16; 256],
    pub(crate) u_version: Uint,
    pub(crate) sz_info_title: [u16; 64],
    pub(crate) dw_info_flags: Dword,
    pub(crate) guid_item: Guid,
    pub(crate) h_balloon_icon: Hicon,
}

#[repr(C)]
pub(crate) struct Choosecolorw {
    pub(crate) l_struct_size: Dword,
    pub(crate) hwnd_owner: Hwnd,
    pub(crate) h_instance: Hwnd,
    pub(crate) rgb_result: Dword,
    pub(crate) lp_cust_colors: *mut Dword,
    pub(crate) flags: Dword,
    pub(crate) l_cust_data: Lparam,
    pub(crate) lpfn_hook: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Uint>,
    pub(crate) lp_template_name: Pcwstr,
}

#[repr(C)]
pub(crate) struct Bitmapinfoheader {
    pub(crate) bi_size: Dword,
    pub(crate) bi_width: i32,
    pub(crate) bi_height: i32,
    pub(crate) bi_planes: u16,
    pub(crate) bi_bit_count: u16,
    pub(crate) bi_compression: Dword,
    pub(crate) bi_size_image: Dword,
    pub(crate) bi_x_pels_per_meter: i32,
    pub(crate) bi_y_pels_per_meter: i32,
    pub(crate) bi_clr_used: Dword,
    pub(crate) bi_clr_important: Dword,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct Rgbquad {
    pub(crate) rgb_blue: u8,
    pub(crate) rgb_green: u8,
    pub(crate) rgb_red: u8,
    pub(crate) rgb_reserved: u8,
}

#[repr(C)]
pub(crate) struct Bitmapinfo {
    pub(crate) bmi_header: Bitmapinfoheader,
    pub(crate) bmi_colors: [Rgbquad; 1],
}

#[repr(C)]
pub(crate) struct Blendfunction {
    pub(crate) blend_op: u8,
    pub(crate) blend_flags: u8,
    pub(crate) source_constant_alpha: u8,
    pub(crate) alpha_format: u8,
}

#[link(name = "user32")]
#[link(name = "gdi32")]
#[link(name = "shell32")]
#[link(name = "comdlg32")]
#[link(name = "shcore")]
unsafe extern "system" {
    pub(crate) fn AppendMenuW(
        h_menu: Hmenu,
        u_flags: Uint,
        u_id_new_item: usize,
        lp_new_item: Pcwstr,
    ) -> Bool;
    pub(crate) fn BeginPaint(hwnd: Hwnd, lp_paint: *mut Paintstruct) -> Hdc;
    pub(crate) fn CallNextHookEx(
        hhk: Hhook,
        n_code: i32,
        wparam: Wparam,
        lparam: Lparam,
    ) -> Lresult;
    pub(crate) fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    pub(crate) fn CreateDIBSection(
        hdc: Hdc,
        pbmi: *const Bitmapinfo,
        usage: Uint,
        ppv_bits: *mut *mut c_void,
        h_section: isize,
        offset: Dword,
    ) -> Hbitmap;
    pub(crate) fn CreatePen(style: i32, width: i32, color: u32) -> Hpen;
    pub(crate) fn CreatePopupMenu() -> Hmenu;
    pub(crate) fn CreateSolidBrush(color: u32) -> Hbrush;
    pub(crate) fn CreateWindowExW(
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
    pub(crate) fn DefWindowProcW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    pub(crate) fn DeleteDC(hdc: Hdc) -> Bool;
    pub(crate) fn DeleteObject(ho: Hgdiobj) -> Bool;
    pub(crate) fn DestroyMenu(h_menu: Hmenu) -> Bool;
    pub(crate) fn DestroyWindow(hwnd: Hwnd) -> Bool;
    pub(crate) fn DispatchMessageW(lp_msg: *const Msg) -> Lresult;
    pub(crate) fn DrawIconEx(
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
    pub(crate) fn DrawTextW(
        hdc: Hdc,
        text: Pcwstr,
        text_len: i32,
        rect: *mut Rect,
        format: Uint,
    ) -> i32;
    pub(crate) fn EnumDisplayDevicesW(
        lp_device: Pcwstr,
        i_dev_num: Dword,
        lp_display_device: *mut DisplayDeviceW,
        dw_flags: Dword,
    ) -> Bool;
    pub(crate) fn EnumDisplayMonitors(
        hdc: Hdc,
        clip_rect: *const Rect,
        callback: Option<unsafe extern "system" fn(Hmonitor, Hdc, *mut Rect, Lparam) -> Bool>,
        data: Lparam,
    ) -> Bool;
    pub(crate) fn EndPaint(hwnd: Hwnd, lp_paint: *const Paintstruct) -> Bool;
    pub(crate) fn FillRect(hdc: Hdc, rect: *const Rect, hbr: Hbrush) -> i32;
    pub(crate) fn GetAncestor(hwnd: Hwnd, ga_flags: Uint) -> Hwnd;
    pub(crate) fn GetAsyncKeyState(vkey: i32) -> i16;
    pub(crate) fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
    pub(crate) fn GetCursorPos(point: *mut Point) -> Bool;
    pub(crate) fn GetDC(hwnd: Hwnd) -> Hdc;
    pub(crate) fn GetDlgItem(hwnd: Hwnd, id: i32) -> Hwnd;
    pub(crate) fn GetForegroundWindow() -> Hwnd;
    pub(crate) fn GetGUIThreadInfo(id_thread: Dword, gui: *mut Guithreadinfo) -> Bool;
    pub(crate) fn GetMessageW(
        lp_msg: *mut Msg,
        hwnd: Hwnd,
        msg_filter_min: Uint,
        msg_filter_max: Uint,
    ) -> Bool;
    pub(crate) fn GetModuleHandleW(lp_module_name: Pcwstr) -> Hinstance;
    pub(crate) fn GetMonitorInfoW(hmonitor: Hmonitor, info: *mut Monitorinfo) -> Bool;
    pub(crate) fn GetStockObject(index: i32) -> Hgdiobj;
    pub(crate) fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: i32) -> i32;
    pub(crate) fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut Dword) -> Dword;
    pub(crate) fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
    pub(crate) fn LoadCursorW(hinstance: Hinstance, cursor_name: Pcwstr) -> Hcursor;
    pub(crate) fn LoadIconW(hinstance: Hinstance, icon_name: Pcwstr) -> Hicon;
    pub(crate) fn MessageBoxW(hwnd: Hwnd, text: Pcwstr, caption: Pcwstr, flags: Uint) -> i32;
    pub(crate) fn MonitorFromPoint(point: Point, flags: Dword) -> Hmonitor;
    pub(crate) fn PostMessageW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Bool;
    pub(crate) fn PostQuitMessage(exit_code: i32);
    pub(crate) fn Rectangle(hdc: Hdc, left: i32, top: i32, right: i32, bottom: i32) -> Bool;
    pub(crate) fn RedrawWindow(hwnd: Hwnd, rect: *const Rect, region: Hrgn, flags: Uint) -> Bool;
    pub(crate) fn RegisterClassExW(wnd_class: *const Wndclassexw) -> u16;
    pub(crate) fn ReleaseCapture() -> Bool;
    pub(crate) fn ReleaseDC(hwnd: Hwnd, hdc: Hdc) -> i32;
    pub(crate) fn SelectObject(hdc: Hdc, object: Hgdiobj) -> Hgdiobj;
    pub(crate) fn ScreenToClient(hwnd: Hwnd, point: *mut Point) -> Bool;
    pub(crate) fn SendMessageTimeoutW(
        hwnd: Hwnd,
        msg: Uint,
        wparam: Wparam,
        lparam: Lparam,
        flags: Uint,
        timeout: Uint,
        result: *mut Lparam,
    ) -> Lresult;
    pub(crate) fn SendMessageW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    pub(crate) fn SetBkColor(hdc: Hdc, color: u32) -> u32;
    pub(crate) fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    pub(crate) fn SetForegroundWindow(hwnd: Hwnd) -> Bool;
    pub(crate) fn SetProcessDpiAwarenessContext(value: isize) -> Bool;
    pub(crate) fn SetTimer(
        hwnd: Hwnd,
        id_event: usize,
        elapse: Uint,
        timer_func: Option<unsafe extern "system" fn(Hwnd, Uint, usize, Dword)>,
    ) -> usize;
    pub(crate) fn SetWindowTextW(hwnd: Hwnd, text: Pcwstr) -> Bool;
    pub(crate) fn SetTextColor(hdc: Hdc, color: u32) -> u32;
    pub(crate) fn SetWindowPos(
        hwnd: Hwnd,
        hwnd_insert_after: Hwnd,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: Uint,
    ) -> Bool;
    pub(crate) fn SetWindowsHookExW(
        hook: i32,
        proc: Option<unsafe extern "system" fn(i32, Wparam, Lparam) -> Lresult>,
        hmod: Hinstance,
        thread_id: Dword,
    ) -> Hhook;
    pub(crate) fn Shell_NotifyIconW(message: Dword, data: *mut Notifyicondataw) -> Bool;
    pub(crate) fn ShowWindow(hwnd: Hwnd, cmd_show: i32) -> Bool;
    pub(crate) fn TrackPopupMenu(
        menu: Hmenu,
        flags: Uint,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: Hwnd,
        rect: *const Rect,
    ) -> i32;
    pub(crate) fn TranslateMessage(lp_msg: *const Msg) -> Bool;
    pub(crate) fn UnhookWindowsHookEx(hhk: Hhook) -> Bool;
    pub(crate) fn UpdateLayeredWindow(
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
    pub(crate) fn WindowFromPoint(point: Point) -> Hwnd;
    pub(crate) fn GetCurrentProcessId() -> Dword;
    pub(crate) fn ChooseColorW(choose_color: *mut Choosecolorw) -> Bool;
    pub(crate) fn KillTimer(hwnd: Hwnd, id_event: usize) -> Bool;
}
