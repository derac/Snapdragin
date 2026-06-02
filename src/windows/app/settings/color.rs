use super::super::*;

pub(in crate::windows::app) fn normalize_hex_color(value: Option<&str>, default: &str) -> String {
    let Some(value) = value else {
        return default.to_string();
    };

    let value = value.trim().to_ascii_uppercase();
    if is_valid_hex_color(&value) {
        value
    } else {
        default.to_string()
    }
}

pub(in crate::windows::app) fn is_valid_hex_color(value: &str) -> bool {
    let value = value.trim();
    let valid_len = value.len() == 7 || value.len() == 9;
    valid_len && value.starts_with('#') && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

pub(in crate::windows::app) fn colorref_from_hex(value: &str) -> Option<u32> {
    let normalized = normalize_hex_color(Some(value), DEFAULT_GRID_COLOR);
    let hex = normalized.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(rgb(red, green, blue))
}

pub(in crate::windows::app) fn rgba_from_hex(value: &str, default: &str) -> RgbaColor {
    let normalized = normalize_hex_color(Some(value), default);
    let hex = normalized.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let green = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let blue = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    let alpha = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };

    RgbaColor {
        red,
        green,
        blue,
        alpha,
    }
}
