use super::*;

pub(super) unsafe fn add_tray_icon(hwnd: Hwnd) {
    let mut nid = notify_icon(hwnd);
    Shell_NotifyIconW(NIM_ADD, &mut nid);
}

pub(super) unsafe fn remove_tray_icon(hwnd: Hwnd) {
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
    nid.h_icon = with_state(|state| state.app_icon).unwrap_or_default();
    copy_wide(&mut nid.sz_tip, APP_NAME);
    nid
}

pub(super) unsafe fn show_context_menu(hwnd: Hwnd) {
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

pub(super) fn handle_menu_command(command: usize) {
    match command {
        ID_SETTINGS => {
            let hwnd = with_state(|state| state.main_hwnd).unwrap_or_default();
            unsafe {
                settings::show_settings_window(hwnd);
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
