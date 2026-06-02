use super::snap;
use super::*;

pub(super) fn handle_mouse_message(message: u32, point: ScreenPoint) -> bool {
    match message {
        WM_MOUSEMOVE => {
            if with_state(|state| state.dragging).unwrap_or(false)
                && !desktop::left_button_is_down()
            {
                finish_drag_at(point);
            } else {
                update_drag(point);
            }
            false
        }
        WM_LBUTTONDOWN => {
            with_state(|state| state.left_button_down = true);
            false
        }
        WM_LBUTTONUP => {
            let should_finish = with_state(|state| {
                state.left_button_down = false;
                state.dragging
            })
            .unwrap_or(false);

            if should_finish {
                finish_drag_at(point);
            }
            false
        }
        WM_RBUTTONDOWN => handle_right_button_down(point),
        WM_RBUTTONUP => with_state(|state| {
            if state.suppress_right_up {
                state.suppress_right_up = false;
                true
            } else {
                state.dragging
            }
        })
        .unwrap_or(false),
        _ => false,
    }
}

fn handle_right_button_down(point: ScreenPoint) -> bool {
    let physical_left_down = desktop::left_button_is_down();
    let already_dragging = with_state(|state| {
        state.left_button_down = physical_left_down;
        state.dragging
    })
    .unwrap_or(false);

    if already_dragging {
        finish_drag_at(point);
        with_state(|state| state.suppress_right_up = true);
        return true;
    }

    if !physical_left_down {
        return false;
    }

    let Some(target) = desktop::target_window(point) else {
        return false;
    };

    if !desktop::window_is_in_move_size_loop(target) {
        return false;
    }

    with_state(|state| state.suppress_right_up = true);

    if unsafe { !desktop::break_drag_loop(target, point) } {
        return true;
    }

    begin_drag(target, point);
    true
}

fn begin_drag(target: Hwnd, point: ScreenPoint) {
    settings::refresh_monitors();
    let (monitor_info, grid) = with_state(|state| {
        let monitor_info = active_monitor_for_point(state, point);
        let grid = grid_for_monitor(state, &monitor_info);
        (monitor_info, grid)
    })
    .unwrap_or_else(|| {
        let monitor_info = unsafe { desktop::monitor_info_from_point(point) };
        let grid = default_grid();
        (monitor_info, grid)
    });
    let monitor_rect = monitor_info.work_rect;
    let mut tracker = SelectionTracker::new(grid, monitor_rect);
    let selection = tracker.begin(point);

    with_state(|state| {
        state.dragging = true;
        state.target_hwnd = target;
        state.grid = grid;
        state.monitor_device_name = monitor_info.device_name.clone();
        state.monitor_rect = monitor_rect;
        state.tracker = Some(tracker);
        state.selection = selection;
    });

    unsafe {
        overlay::show_overlay(monitor_rect);
    }
    snap::queue_current_snap();
}

fn grid_for_monitor(state: &AppState, monitor_info: &MonitorConfig) -> GridSpec {
    state
        .settings
        .monitors
        .iter()
        .find(|monitor| monitor.device_name == monitor_info.device_name)
        .map(|monitor| GridSpec::clamped(monitor.columns, monitor.rows))
        .unwrap_or_else(|| GridSpec::clamped(monitor_info.columns, monitor_info.rows))
}

fn monitor_for_point(state: &AppState, point: ScreenPoint) -> Option<MonitorConfig> {
    state
        .settings
        .monitors
        .iter()
        .find(|monitor| monitor.monitor_rect.contains(point))
        .cloned()
        .or_else(|| {
            state
                .settings
                .monitors
                .iter()
                .min_by_key(|monitor| point_distance_to_rect_squared(point, monitor.monitor_rect))
                .cloned()
        })
}

fn active_monitor_for_point(state: &AppState, point: ScreenPoint) -> MonitorConfig {
    let mut native = unsafe { desktop::monitor_info_from_point(point) };

    if let Some(monitor) = state
        .settings
        .monitors
        .iter()
        .find(|monitor| monitor.device_name == native.device_name)
        .cloned()
    {
        return monitor;
    }

    if let Some(saved) = monitor_for_point(state, point) {
        native.columns = saved.columns;
        native.rows = saved.rows;
    }

    native
}

fn point_distance_to_rect_squared(point: ScreenPoint, rect: ScreenRect) -> i64 {
    let x = i64::from(point.x);
    let y = i64::from(point.y);
    let left = i64::from(rect.x);
    let top = i64::from(rect.y);
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);

    let dx = if x < left {
        left - x
    } else if x > right {
        x - right
    } else {
        0
    };
    let dy = if y < top {
        top - y
    } else if y > bottom {
        y - bottom
    } else {
        0
    };

    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
}

fn update_drag(point: ScreenPoint) {
    let (changed, new_overlay_monitor) = update_drag_monitor_and_selection(point);

    if changed {
        unsafe {
            if let Some(monitor) = new_overlay_monitor {
                overlay::show_overlay(monitor);
            } else {
                overlay::invalidate_overlay();
            }
        }
        snap::queue_current_snap();
    }
}

fn update_drag_monitor_and_selection(point: ScreenPoint) -> (bool, Option<ScreenRect>) {
    with_state(|state| {
        if !state.dragging {
            return (false, None);
        }

        let monitor_info = active_monitor_for_point(state, point);
        let monitor_rect = monitor_info.work_rect;

        if state.monitor_device_name == monitor_info.device_name {
            return (update_drag_selection_in_state(state, point), None);
        }

        let grid = grid_for_monitor(state, &monitor_info);
        let mut tracker = SelectionTracker::new(grid, monitor_rect);
        let selection = tracker.begin(point);

        state.grid = grid;
        state.monitor_device_name = monitor_info.device_name.clone();
        state.monitor_rect = monitor_rect;
        state.tracker = Some(tracker);
        state.selection = selection;

        (selection.is_some(), Some(monitor_rect))
    })
    .unwrap_or((false, None))
}

fn update_drag_selection_in_state(state: &mut AppState, point: ScreenPoint) -> bool {
    if !state.dragging {
        return false;
    }

    let Some(mut tracker) = state.tracker else {
        return false;
    };

    let selection = tracker.update(point);
    state.tracker = Some(tracker);

    if selection != state.selection {
        state.selection = selection;
        return true;
    }

    false
}

fn finish_drag() {
    let (overlay, snap) = with_state(|state| {
        let overlay = state.overlay_hwnd;
        let snap = snap::current_snap(state);
        state.clear_drag();
        (overlay, snap)
    })
    .unwrap_or_default();

    unsafe {
        if overlay != 0 {
            ShowWindow(overlay, SW_HIDE);
        }
        ReleaseCapture();
    }

    if let Some(snap) = snap {
        snap::queue_snap(snap);
    }
}

fn finish_drag_at(point: ScreenPoint) {
    update_drag(point);
    finish_drag();
}
