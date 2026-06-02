use super::super::desktop::enumerate_monitors;
use super::super::*;
use super::controls::{create_button, create_checkbox, create_edit, layout_settings_controls};
use super::paint::{paint_settings, settings_control_brush};
use super::*;

pub(in crate::windows::app) unsafe extern "system" fn wnd_proc(
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

pub(in crate::windows::app) unsafe fn show_settings_window(owner: Hwnd) {
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

pub(in crate::windows::app) fn refresh_monitors() {
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

pub(in crate::windows::app) fn rebuild_settings_window_if_open() {
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
