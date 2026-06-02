use super::settings::rgba_from_hex;
use super::*;

pub(super) unsafe extern "system" fn wnd_proc(
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

pub(super) unsafe fn show_overlay(monitor: ScreenRect) {
    let overlay = with_state(|state| state.overlay_hwnd).unwrap_or_default();
    let hwnd = if overlay == 0 {
        let class_name = wide(OVERLAY_CLASS);
        let title = wide("Snapdragin Overlay");
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

pub(super) unsafe fn invalidate_overlay() {
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
