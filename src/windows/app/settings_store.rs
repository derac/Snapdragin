use super::desktop::enumerate_monitors;
use super::*;
use std::path::Path;

const CLSCTX_INPROC_SERVER: Dword = 0x0000_0001;
const COINIT_APARTMENTTHREADED: Dword = 0x0000_0002;
const RPC_E_CHANGED_MODE: i32 = 0x8001_0106_u32 as i32;
const CLSID_SHELL_LINK: Guid = Guid {
    data1: 0x0002_1401,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
const IID_ISHELL_LINKW: Guid = Guid {
    data1: 0x0002_14F9,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
const IID_IPERSIST_FILE: Guid = Guid {
    data1: 0x0000_010B,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

pub(super) fn load_settings() -> SettingsData {
    let stored = load_stored_settings()
        .or_else(load_original_settings)
        .unwrap_or_else(default_stored_settings);
    merge_monitors(stored, unsafe { enumerate_monitors() })
}

fn default_stored_settings() -> StoredSettings {
    StoredSettings {
        monitors: Vec::new(),
        grid_color: DEFAULT_GRID_COLOR.to_string(),
        selection_color: DEFAULT_SELECTION_COLOR.to_string(),
        selection_border_color: DEFAULT_SELECTION_BORDER_COLOR.to_string(),
        run_on_startup: None,
        is_dark_mode: Some(true),
    }
}

pub(super) fn stored_settings_from_data(settings: &SettingsData) -> StoredSettings {
    StoredSettings {
        monitors: settings
            .monitors
            .iter()
            .map(|monitor| StoredMonitorConfig {
                device_name: monitor.device_name.clone(),
                display_name: monitor.display_name.clone(),
                columns: monitor.columns,
                rows: monitor.rows,
            })
            .collect(),
        grid_color: settings.grid_color.clone(),
        selection_color: settings.selection_color.clone(),
        selection_border_color: settings.selection_border_color.clone(),
        run_on_startup: Some(settings.run_on_startup),
        is_dark_mode: Some(settings.is_dark_mode),
    }
}

pub(super) fn merge_monitors(
    stored: StoredSettings,
    mut monitors: Vec<MonitorConfig>,
) -> SettingsData {
    if monitors.is_empty() {
        monitors.push(MonitorConfig {
            device_name: "DISPLAY".to_string(),
            display_name: "Primary Monitor".to_string(),
            monitor_rect: ScreenRect::new(0, 0, 1, 1),
            work_rect: ScreenRect::new(0, 0, 1, 1),
            columns: 2,
            rows: 2,
        });
    }

    for monitor in &mut monitors {
        if let Some(stored_monitor) = stored
            .monitors
            .iter()
            .find(|candidate| candidate.device_name == monitor.device_name)
        {
            monitor.columns = clamp_grid_dimension(stored_monitor.columns);
            monitor.rows = clamp_grid_dimension(stored_monitor.rows);
            if monitor.display_name == monitor.device_name
                && !stored_monitor.display_name.is_empty()
            {
                monitor.display_name = stored_monitor.display_name.clone();
            }
        }
    }

    SettingsData {
        monitors,
        grid_color: normalize_hex_color(Some(&stored.grid_color), DEFAULT_GRID_COLOR),
        selection_color: normalize_hex_color(
            Some(&stored.selection_color),
            DEFAULT_SELECTION_COLOR,
        ),
        selection_border_color: normalize_hex_color(
            Some(&stored.selection_border_color),
            DEFAULT_SELECTION_BORDER_COLOR,
        ),
        run_on_startup: stored.run_on_startup.unwrap_or_else(startup_is_enabled),
        is_dark_mode: stored.is_dark_mode.unwrap_or(true),
    }
}

fn load_stored_settings() -> Option<StoredSettings> {
    let path = settings_path().filter(|path| path.exists())?;
    let contents = fs::read_to_string(path).ok()?;
    let mut settings = default_stored_settings();

    for line in contents.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key.trim() {
            "grid_color" => settings.grid_color = value.trim().to_string(),
            "selection_color" => settings.selection_color = value.trim().to_string(),
            "selection_border_color" => settings.selection_border_color = value.trim().to_string(),
            "run_on_startup" => {
                settings.run_on_startup = Some(value.trim().eq_ignore_ascii_case("true"))
            }
            "is_dark_mode" => {
                settings.is_dark_mode = Some(value.trim().eq_ignore_ascii_case("true"))
            }
            "monitor" => {
                let parts: Vec<&str> = value.split('|').collect();
                if parts.len() >= 4 {
                    settings.monitors.push(StoredMonitorConfig {
                        device_name: parts[0].to_string(),
                        display_name: parts[1].to_string(),
                        columns: parts[2].parse().unwrap_or(2),
                        rows: parts[3].parse().unwrap_or(2),
                    });
                }
            }
            _ => {}
        }
    }

    Some(settings)
}

fn load_original_settings() -> Option<StoredSettings> {
    let contents = fs::read_to_string(original_settings_path()?).ok()?;
    let mut settings = default_stored_settings();

    if let Some(color) = extract_json_string(&contents, "GridColor") {
        settings.grid_color = argb_to_rgba_hex(&color, DEFAULT_GRID_COLOR);
    }
    if let Some(color) = extract_json_string(&contents, "SelectionColor") {
        settings.selection_color = argb_to_rgba_hex(&color, DEFAULT_SELECTION_COLOR);
    }
    if let Some(color) = extract_json_string(&contents, "SelectionBorderColor") {
        settings.selection_border_color = argb_to_rgba_hex(&color, DEFAULT_SELECTION_BORDER_COLOR);
    }
    settings.run_on_startup = extract_json_bool(&contents, "RunOnStartup");
    settings.is_dark_mode = extract_json_bool(&contents, "IsDarkMode").or(settings.is_dark_mode);

    let mut cursor = contents.as_str();
    while let Some(index) = cursor.find("\"DeviceName\"") {
        cursor = &cursor[index..];
        let block_end = cursor.find('}').unwrap_or(cursor.len());
        let block = &cursor[..block_end];
        if let Some(device_name) = extract_json_string(block, "DeviceName") {
            settings.monitors.push(StoredMonitorConfig {
                device_name,
                display_name: extract_json_string(block, "FriendlyName").unwrap_or_default(),
                columns: extract_json_number(block, "Columns").unwrap_or(2),
                rows: extract_json_number(block, "Rows").unwrap_or(2),
            });
        }
        cursor = &cursor[block_end..];
    }

    Some(settings)
}

pub(super) fn save_settings(settings: &SettingsData) {
    let Some(path) = settings_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut contents = String::new();
    contents.push_str(&format!("grid_color={}\n", settings.grid_color));
    contents.push_str(&format!("selection_color={}\n", settings.selection_color));
    contents.push_str(&format!(
        "selection_border_color={}\n",
        settings.selection_border_color
    ));
    contents.push_str(&format!("run_on_startup={}\n", settings.run_on_startup));
    contents.push_str(&format!("is_dark_mode={}\n", settings.is_dark_mode));

    for monitor in &settings.monitors {
        contents.push_str(&format!(
            "monitor={}|{}|{}|{}\n",
            monitor.device_name, monitor.display_name, monitor.columns, monitor.rows
        ));
    }

    let _ = fs::write(path, contents);
}

fn extract_json_string(contents: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = contents.find(&needle)?;
    let after_key = &contents[start + needle.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    let after_quote = after_colon.strip_prefix('"')?;
    let end = after_quote.find('"')?;
    Some(after_quote[..end].replace("\\\\", "\\"))
}

fn extract_json_bool(contents: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let start = contents.find(&needle)?;
    let after_key = &contents[start + needle.len()..];
    let colon = after_key.find(':')?;
    let value = after_key[colon + 1..].trim_start();
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn extract_json_number(contents: &str, key: &str) -> Option<u16> {
    let needle = format!("\"{key}\"");
    let start = contents.find(&needle)?;
    let after_key = &contents[start + needle.len()..];
    let colon = after_key.find(':')?;
    let value = after_key[colon + 1..].trim_start();
    let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .map(|base| base.join(APP_DIR_NAME).join("settings.ini"))
}

fn original_settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|base| base.join("TheGriddler").join("settings.json"))
}

fn startup_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from).map(|base| {
        base.join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join(STARTUP_SHORTCUT_NAME)
    })
}

fn startup_script_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from).map(|base| {
        base.join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join(LEGACY_STARTUP_SCRIPT_NAME)
    })
}

pub(super) fn startup_is_enabled() -> bool {
    startup_path().is_some_and(|path| path.exists())
        || startup_script_path().is_some_and(|path| path.exists())
}

pub(super) fn set_startup_enabled(enabled: bool) {
    let Some(path) = startup_path() else {
        return;
    };

    if enabled {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(exe) = env::current_exe() {
            let _ = create_startup_shortcut(&path, &exe);
        }
        if let Some(script_path) = startup_script_path() {
            let _ = fs::remove_file(script_path);
        }
    } else {
        let _ = fs::remove_file(path);
        if let Some(script_path) = startup_script_path() {
            let _ = fs::remove_file(script_path);
        }
    }
}

fn create_startup_shortcut(path: &Path, exe: &Path) -> bool {
    unsafe { create_startup_shortcut_com(path, exe) }
}

unsafe fn create_startup_shortcut_com(path: &Path, exe: &Path) -> bool {
    let init_result = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED);
    let should_uninitialize = init_result >= 0;
    if init_result < 0 && init_result != RPC_E_CHANGED_MODE {
        return false;
    }

    let mut shell_link = null_mut();
    let created = CoCreateInstance(
        &CLSID_SHELL_LINK,
        null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_ISHELL_LINKW,
        &mut shell_link,
    );

    let ok = if created >= 0 && !shell_link.is_null() {
        save_shell_link(shell_link.cast::<IShellLinkW>(), path, exe)
    } else {
        false
    };

    if should_uninitialize {
        CoUninitialize();
    }

    ok
}

unsafe fn save_shell_link(shell_link: *mut IShellLinkW, path: &Path, exe: &Path) -> bool {
    let exe_path = wide(&exe.to_string_lossy());
    let set_path = ((*(*shell_link).lp_vtbl).set_path)(shell_link, exe_path.as_ptr()) >= 0;
    if !set_path {
        ((*(*shell_link).lp_vtbl).release)(shell_link);
        return false;
    }

    if let Some(working_dir) = exe.parent() {
        let working_dir = wide(&working_dir.to_string_lossy());
        let _ = ((*(*shell_link).lp_vtbl).set_working_directory)(shell_link, working_dir.as_ptr());
    }

    let mut persist_file = null_mut();
    let queried = ((*(*shell_link).lp_vtbl).query_interface)(
        shell_link,
        &IID_IPERSIST_FILE,
        &mut persist_file,
    );

    let ok = if queried >= 0 && !persist_file.is_null() {
        let persist_file = persist_file.cast::<IPersistFile>();
        let shortcut_path = wide(&path.to_string_lossy());
        let saved = ((*(*persist_file).lp_vtbl).save)(persist_file, shortcut_path.as_ptr(), 1) >= 0;
        ((*(*persist_file).lp_vtbl).release)(persist_file);
        saved
    } else {
        false
    };

    ((*(*shell_link).lp_vtbl).release)(shell_link);
    ok
}

pub(super) fn clamp_grid_dimension(value: u16) -> u16 {
    value.clamp(MIN_GRID_DIMENSION, MAX_GRID_DIMENSION)
}

pub(super) fn normalize_hex_color(value: Option<&str>, default: &str) -> String {
    let Some(value) = value else {
        return default.to_string();
    };

    let value = value.trim().to_ascii_uppercase();
    if !is_valid_hex_color(&value) {
        return default.to_string();
    }

    match value.as_str() {
        PREVIOUS_DEFAULT_GRID_COLOR => DEFAULT_GRID_COLOR.to_string(),
        PREVIOUS_DEFAULT_SELECTION_COLOR => DEFAULT_SELECTION_COLOR.to_string(),
        PREVIOUS_DEFAULT_SELECTION_BORDER_COLOR => DEFAULT_SELECTION_BORDER_COLOR.to_string(),
        LEGACY_DEFAULT_GRID_COLOR => DEFAULT_GRID_COLOR.to_string(),
        LEGACY_DEFAULT_SELECTION_COLOR => DEFAULT_SELECTION_COLOR.to_string(),
        LEGACY_DEFAULT_SELECTION_BORDER_COLOR => DEFAULT_SELECTION_BORDER_COLOR.to_string(),
        _ => value,
    }
}

fn argb_to_rgba_hex(value: &str, default: &str) -> String {
    let value = value.trim().to_ascii_uppercase();
    if !is_valid_hex_color(&value) {
        return default.to_string();
    }

    if value.len() == 9 {
        format!("#{}{}", &value[3..9], &value[1..3],)
    } else {
        value
    }
}

pub(super) fn is_valid_hex_color(value: &str) -> bool {
    let value = value.trim();
    let valid_len = value.len() == 7 || value.len() == 9;
    valid_len && value.starts_with('#') && value[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

pub(super) fn colorref_from_hex(value: &str) -> Option<u32> {
    let normalized = normalize_hex_color(Some(value), DEFAULT_GRID_COLOR);
    let hex = normalized.trim_start_matches('#');
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(rgb(red, green, blue))
}

pub(super) fn rgba_from_hex(value: &str, default: &str) -> RgbaColor {
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
