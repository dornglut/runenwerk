use engine::runtime::{NativeWindowId, ResMut, WindowState, WindowStateRegistryResource};

/// Approves primary-window close intents for standalone workbenches that do not own
/// editor document state or unsaved-work confirmation.
pub(super) fn approve_primary_window_close_intent_system(
    mut window: ResMut<WindowState>,
    mut windows: ResMut<WindowStateRegistryResource>,
) {
    approve_primary_window_close_intent(&mut window, &mut windows);
}

fn approve_primary_window_close_intent(
    window: &mut WindowState,
    windows: &mut WindowStateRegistryResource,
) {
    let primary_window_id = windows.primary_window_id().unwrap_or_else(NativeWindowId::primary);
    let registry_close_intent = windows
        .record(primary_window_id)
        .is_some_and(|record| record.close_intent_pending);

    if !window.close_intent_pending && !registry_close_intent {
        return;
    }

    window.request_close();
    let primary_window_id = windows.ensure_primary_from_legacy(window);
    if let Some(record) = windows.record_mut(primary_window_id) {
        record.approve_close();
    }
}

#[cfg(test)]
mod tests {
    use engine::runtime::NativeWindowLifecycleState;

    use super::*;

    #[test]
    fn standalone_workbench_close_policy_approves_primary_close_intent() {
        let mut window = WindowState::windowed("UI Gallery");
        let mut windows = WindowStateRegistryResource::from_legacy(&window);
        windows
            .record_mut(NativeWindowId::primary())
            .expect("primary window record")
            .receive_close_intent();

        approve_primary_window_close_intent(&mut window, &mut windows);

        assert!(window.close_requested);
        assert!(!window.close_intent_pending);
        let primary = windows.primary_window_id().expect("primary window id");
        let record = windows.record(primary).expect("primary window record");
        assert!(record.close_requested);
        assert!(!record.close_intent_pending);
        assert_eq!(record.lifecycle_state, NativeWindowLifecycleState::CloseApproved);
    }
}
