use super::super::*;
use super::{colorref_from_hex, load_settings};

pub(super) unsafe fn paint_settings(hwnd: Hwnd) {
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

pub(super) fn settings_control_brush() -> Lresult {
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
