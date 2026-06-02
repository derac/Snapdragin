use super::*;

pub(super) fn queue_current_snap() {
    let snap = with_state(|state| current_snap(state)).flatten();

    if let Some(snap) = snap {
        queue_snap(snap);
    }
}

pub(super) fn current_snap(state: &AppState) -> Option<PendingSnap> {
    let selection = state.selection?;
    let rect = selection.screen_rect(state.grid, state.monitor_rect)?;
    Some(PendingSnap {
        target: state.target_hwnd,
        rect,
    })
}

pub(super) fn queue_snap(snap: PendingSnap) {
    let main_hwnd = with_state(|state| {
        state.queued_snap = Some(snap);

        if state.snap_apply_pending {
            return 0;
        }

        state.snap_apply_pending = true;
        state.main_hwnd
    })
    .unwrap_or_default();

    if main_hwnd != 0 {
        unsafe {
            PostMessageW(main_hwnd, WM_APPLY_SNAP, 0, 0);
        }
    }
}

pub(super) fn apply_queued_snap() {
    let snap = with_state(|state| {
        state.snap_apply_pending = false;
        state.queued_snap.take()
    })
    .flatten();

    if let Some(snap) = snap {
        unsafe {
            apply_snap_to_target(snap);
        }
        queue_snap_settle(snap);
    }
}

unsafe fn apply_snap_to_target(snap: PendingSnap) {
    for attempt in 0..SNAP_SETTLE_ATTEMPTS {
        if desktop::window_is_in_move_size_loop(snap.target) {
            let _ = desktop::break_drag_loop(snap.target, desktop::cursor_position());
        }

        set_target_window_rect(snap.target, snap.rect);

        if attempt + 1 < SNAP_SETTLE_ATTEMPTS {
            thread::sleep(Duration::from_millis(SNAP_SETTLE_DELAY_MS));
        }
    }

    redraw_target_window(snap.target);
}

unsafe fn set_target_window_rect(target: Hwnd, rect: ScreenRect) {
    SetWindowPos(
        target,
        0,
        rect.x,
        rect.y,
        i32::try_from(rect.width).unwrap_or(i32::MAX),
        i32::try_from(rect.height).unwrap_or(i32::MAX),
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
    );
}

unsafe fn redraw_target_window(target: Hwnd) {
    InvalidateRect(target, null(), 1);
    RedrawWindow(
        target,
        null(),
        0,
        RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_UPDATENOW | RDW_FRAME,
    );
}

fn queue_snap_settle(snap: PendingSnap) {
    let main_hwnd = with_state(|state| {
        state.settle_snap = Some(snap);
        state.main_hwnd
    })
    .unwrap_or_default();

    if main_hwnd != 0 {
        unsafe {
            SetTimer(main_hwnd, SNAP_SETTLE_TIMER_ID, SNAP_SETTLE_TIMER_MS, None);
        }
    }
}

pub(super) fn apply_snap_settle_timer(hwnd: Hwnd) {
    unsafe {
        KillTimer(hwnd, SNAP_SETTLE_TIMER_ID);
    }

    let snap = with_state(|state| state.settle_snap.take()).flatten();
    if let Some(snap) = snap {
        unsafe {
            if desktop::window_is_in_move_size_loop(snap.target) {
                let _ = desktop::break_drag_loop(snap.target, desktop::cursor_position());
            }

            set_target_window_rect(snap.target, snap.rect);
            redraw_target_window(snap.target);
        }
    }
}
