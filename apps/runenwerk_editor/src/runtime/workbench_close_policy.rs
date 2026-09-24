use engine::runtime::{NativeWindowId, ResMut, WindowStateRegistryResource};

/// Approves primary-window close intents for standalone workbenches that do not own
/// editor document state or unsaved-work confirmation.
pub(super) fn approve_primary_window_close_intent_system(
    mut windows: ResMut<WindowStateRegistryResource>,
) {
    approve_primary_window_close_intent(&mut windows);
}

fn approve_primary_window_close_intent(windows: &mut WindowStateRegistryResource) {
    let Some(primary_window_id) = windows.primary_window_id() else {
        return;
    };
    let Some(record) = windows.record_mut(primary_window_id) else {
        return;
    };
    if record.close_intent_pending {
        record.approve_close();
    }
}

#[cfg(test)]
mod tests {
    use engine::runtime::NativeWindowLifecycleState;

    use super::*;

    #[test]
    fn standalone_workbench_close_policy_approves_primary_close_intent() {
        let mut windows = WindowStateRegistryResource::default();
        windows.register_primary_window("UI Gallery", (1280, 720), 1.0);
        windows
            .record_mut(NativeWindowId::primary())
            .expect("primary window record")
            .receive_close_intent();

        approve_primary_window_close_intent(&mut windows);

        let primary = windows.primary_window_id().expect("primary window id");
        let record = windows.record(primary).expect("primary window record");
        assert!(record.close_requested);
        assert!(!record.close_intent_pending);
        assert_eq!(
            record.lifecycle_state,
            NativeWindowLifecycleState::CloseApproved
        );
    }
}
