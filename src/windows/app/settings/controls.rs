use super::super::*;

pub(super) unsafe fn create_edit(
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

pub(super) unsafe fn create_button(
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

pub(super) unsafe fn create_checkbox(
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

pub(super) unsafe fn layout_settings_controls(hwnd: Hwnd) {
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
        scale_value(dpi, SETTINGS_WIDTH),
        scale_value(dpi, SETTINGS_HEIGHT),
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
