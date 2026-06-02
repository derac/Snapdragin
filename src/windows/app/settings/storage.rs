use super::super::desktop::enumerate_monitors;
use super::super::*;
use super::color::normalize_hex_color;
use super::startup::startup_is_enabled;

pub(in crate::windows::app) fn load_settings() -> SettingsData {
    let stored = load_stored_settings().unwrap_or_else(default_stored_settings);
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

pub(in crate::windows::app) fn stored_settings_from_data(
    settings: &SettingsData,
) -> StoredSettings {
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

pub(in crate::windows::app) fn merge_monitors(
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

pub(in crate::windows::app) fn save_settings(settings: &SettingsData) {
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

fn settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .map(|base| base.join(APP_DIR_NAME).join("settings.ini"))
}

pub(in crate::windows::app) fn clamp_grid_dimension(value: u16) -> u16 {
    value.clamp(MIN_GRID_DIMENSION, MAX_GRID_DIMENSION)
}
