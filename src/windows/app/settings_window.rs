use super::desktop::enumerate_monitors;
use super::settings_store::*;
use super::*;

pub(super) unsafe extern "system" fn wnd_proc(
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
            let dpi = normalized_dpi(hiword(wparam) as u32);
            with_state(|state| state.settings_dpi = dpi);
            let suggested = lparam as *const Rect;
            if !suggested.is_null() {
                let suggested = *suggested;
                SetWindowPos(
                    hwnd,
                    0,
                    suggested.left,
                    suggested.top,
                    suggested.right - suggested.left,
                    suggested.bottom - suggested.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            } else {
                SetWindowPos(
                    hwnd,
                    0,
                    0,
                    0,
                    scale_value(dpi, SETTINGS_WIDTH),
                    scale_value(dpi, SETTINGS_HEIGHT),
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE,
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

pub(super) unsafe fn show_settings_window(owner: Hwnd) {
    refresh_monitors();

    let existing = with_state(|state| state.settings_hwnd).unwrap_or_default();
    if existing != 0 {
        let dpi = dpi_for_window(existing);
        with_state(|state| state.settings_dpi = dpi);
        layout_settings_controls(existing);
        sync_settings_window(with_state(|state| state.grid).unwrap_or_else(default_grid));
        InvalidateRect(existing, null(), 1);
        ShowWindow(existing, SW_SHOW);
        SetForegroundWindow(existing);
        return;
    }

    let dpi = dpi_for_window(owner);
    let class_name = wide(SETTINGS_CLASS);
    let title = wide(APP_NAME);
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        title.as_ptr(),
        WS_POPUP,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        scale_value(dpi, SETTINGS_WIDTH),
        scale_value(dpi, SETTINGS_HEIGHT),
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

fn dpi_for_window(hwnd: Hwnd) -> u32 {
    let dpi = if hwnd == 0 {
        0
    } else {
        unsafe { GetDpiForWindow(hwnd) }
    };
    normalized_dpi(dpi)
}

fn normalized_dpi(dpi: u32) -> u32 {
    if dpi == 0 {
        96
    } else {
        dpi
    }
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

pub(super) fn refresh_monitors() {
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

pub(super) fn rebuild_settings_window_if_open() {
    let handles = with_state(|state| (state.main_hwnd, state.settings_hwnd)).unwrap_or_default();
    if handles.1 == 0 {
        return;
    }

    unsafe {
        DestroyWindow(handles.1);
        show_settings_window(handles.0);
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
    draw_lightbulb_icon(hdc, dpi, 645, 10);

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

unsafe fn draw_lightbulb_icon(hdc: Hdc, dpi: u32, x: i32, y: i32) {
    let dark = with_state(|state| state.settings.is_dark_mode).unwrap_or(true);
    let color = if dark {
        rgb(255, 214, 102)
    } else {
        rgb(56, 56, 56)
    };
    let pen = CreatePen(PS_SOLID, scale_value(dpi, 2).max(1), color);
    let previous = SelectObject(hdc, pen);
    let hollow = GetStockObject(5);
    let previous_brush = SelectObject(hdc, hollow);

    Ellipse(
        hdc,
        scale_value(dpi, x + 6),
        scale_value(dpi, y + 2),
        scale_value(dpi, x + 18),
        scale_value(dpi, y + 15),
    );
    draw_scaled_line(hdc, dpi, x + 9, y + 15, x + 9, y + 18);
    draw_scaled_line(hdc, dpi, x + 15, y + 15, x + 15, y + 18);
    Rectangle(
        hdc,
        scale_value(dpi, x + 8),
        scale_value(dpi, y + 18),
        scale_value(dpi, x + 16),
        scale_value(dpi, y + 22),
    );
    draw_scaled_line(hdc, dpi, x + 9, y + 20, x + 15, y + 20);
    draw_scaled_line(hdc, dpi, x + 12, y, x + 12, y - 3);
    draw_scaled_line(hdc, dpi, x + 4, y + 6, x + 1, y + 3);
    draw_scaled_line(hdc, dpi, x + 20, y + 6, x + 23, y + 3);

    SelectObject(hdc, previous_brush);
    SelectObject(hdc, previous);
    DeleteObject(pen);
}

unsafe fn draw_scaled_line(hdc: Hdc, dpi: u32, x1: i32, y1: i32, x2: i32, y2: i32) {
    MoveToEx(hdc, scale_value(dpi, x1), scale_value(dpi, y1), null_mut());
    LineTo(hdc, scale_value(dpi, x2), scale_value(dpi, y2));
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
