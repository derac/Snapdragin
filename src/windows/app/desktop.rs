use super::*;

pub(super) fn target_window(point: ScreenPoint) -> Option<Hwnd> {
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

pub(super) fn window_is_in_move_size_loop(hwnd: Hwnd) -> bool {
    unsafe {
        gui_thread_info(hwnd)
            .map(|info| (info.flags & GUI_INMOVESIZE) != 0 || info.hwnd_move_size != 0)
            .unwrap_or(true)
    }
}

pub(super) unsafe fn break_drag_loop(hwnd: Hwnd, point: ScreenPoint) -> bool {
    cancel_drag_loop_once(hwnd, point);

    for _ in 0..DRAG_CANCEL_SETTLE_ATTEMPTS {
        if !window_is_in_move_size_loop(hwnd) {
            return true;
        }

        thread::sleep(Duration::from_millis(DRAG_CANCEL_SETTLE_DELAY_MS));
        cancel_drag_loop_once(hwnd, cursor_position());
    }

    !window_is_in_move_size_loop(hwnd)
}

unsafe fn cancel_drag_loop_once(hwnd: Hwnd, point: ScreenPoint) -> bool {
    ReleaseCapture();

    let mut sent = true;
    for target in drag_cancel_targets(hwnd) {
        if target == 0 {
            continue;
        }

        let client_point = client_point_for_window(target, point);

        sent &= send_drag_cancel_message(target, WM_CANCELMODE, 0, 0);
        sent &= send_drag_cancel_message(target, WM_NCLBUTTONUP, HTCAPTION, point_lparam(point));
        sent &= send_drag_cancel_message(target, WM_LBUTTONUP, 0, point_lparam(client_point));
    }

    ReleaseCapture();
    sent
}

pub(super) unsafe fn cursor_position() -> ScreenPoint {
    let mut point: Point = zeroed();
    if GetCursorPos(&mut point) == 0 {
        return ScreenPoint::new(0, 0);
    }

    ScreenPoint::new(point.x, point.y)
}

unsafe fn client_point_for_window(hwnd: Hwnd, point: ScreenPoint) -> ScreenPoint {
    let mut client = Point {
        x: point.x,
        y: point.y,
    };

    if ScreenToClient(hwnd, &mut client) == 0 {
        return point;
    }

    ScreenPoint::new(client.x, client.y)
}

unsafe fn drag_cancel_targets(hwnd: Hwnd) -> [Hwnd; 3] {
    let mut targets = [hwnd, 0, 0];

    if let Some(info) = gui_thread_info(hwnd) {
        push_unique_hwnd(&mut targets, info.hwnd_move_size);
        push_unique_hwnd(&mut targets, info.hwnd_capture);
    }

    targets
}

fn push_unique_hwnd(targets: &mut [Hwnd; 3], hwnd: Hwnd) {
    if hwnd == 0 || targets.contains(&hwnd) {
        return;
    }

    if let Some(slot) = targets.iter_mut().find(|target| **target == 0) {
        *slot = hwnd;
    }
}

unsafe fn gui_thread_info(hwnd: Hwnd) -> Option<Guithreadinfo> {
    let thread_id = GetWindowThreadProcessId(hwnd, null_mut());
    if thread_id == 0 {
        return None;
    }

    let mut info: Guithreadinfo = zeroed();
    info.cb_size = size_of::<Guithreadinfo>() as u32;

    if GetGUIThreadInfo(thread_id, &mut info) == 0 {
        return None;
    }

    Some(info)
}

unsafe fn send_drag_cancel_message(
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
) -> bool {
    let mut result = 0;
    let sent = SendMessageTimeoutW(
        hwnd,
        message,
        wparam,
        lparam,
        SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
        DRAG_CANCEL_TIMEOUT_MS,
        &mut result,
    );

    if sent == 0 {
        PostMessageW(hwnd, message, wparam, lparam);
        return false;
    }

    true
}

pub(super) unsafe fn monitor_info_from_point(point: ScreenPoint) -> MonitorConfig {
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
        monitor_rect: ScreenRect::new(point.x, point.y, 1, 1),
        work_rect: ScreenRect::new(point.x, point.y, 1, 1),
        columns: 2,
        rows: 2,
    })
}

pub(super) unsafe fn enumerate_monitors() -> Vec<MonitorConfig> {
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
        monitor_rect: rect_to_screen_rect(info.rc_monitor),
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

pub(super) fn left_button_is_down() -> bool {
    unsafe { (GetAsyncKeyState(VK_LBUTTON) & i16::MIN) != 0 }
}
