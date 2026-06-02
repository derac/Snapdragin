use super::*;

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
        lpfn_wnd_proc: Some(settings::wnd_proc),
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
                WM_LBUTTONDBLCLK => settings::show_settings_window(hwnd),
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            tray::handle_menu_command(wparam & 0xFFFF);
            0
        }
        WM_APPLY_SNAP => {
            snap::apply_queued_snap();
            0
        }
        WM_TIMER => {
            if wparam == SNAP_SETTLE_TIMER_ID {
                snap::apply_snap_settle_timer(hwnd);
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_DISPLAYCHANGE => {
            settings::refresh_monitors();
            settings::rebuild_settings_window_if_open();
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
        if drag::handle_mouse_message(wparam as u32, ScreenPoint::new(hook.pt.x, hook.pt.y)) {
            return 1;
        }
    }

    let hook = STATE
        .get()
        .and_then(|state| state.lock().ok().map(|state| state.hook))
        .unwrap_or_default();

    CallNextHookEx(hook, n_code, wparam, lparam)
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
