// Owner: Engine Input Plugin - Tests
#[cfg(test)]
mod tests {
    use crate::plugins::{
        InputBindingChange, InputBindingChangeResult, InputState, KeyChord, action,
    };
    use winit::keyboard::KeyCode;

    #[test]
    fn default_bindings_split_enter_by_shift() {
        let mut submit = InputState::new();
        submit.keys_down.insert(KeyCode::Enter);
        submit.apply_action_press_for_key(KeyCode::Enter);
        assert!(submit.submitted);
        assert!(!submit.insert_newline);

        let mut newline = InputState::new();
        newline.keys_down.insert(KeyCode::ShiftLeft);
        newline.keys_down.insert(KeyCode::Enter);
        newline.apply_action_press_for_key(KeyCode::Enter);
        assert!(newline.insert_newline);
        assert!(!newline.submitted);
    }

    #[test]
    fn default_bindings_split_scene_f2_by_shift() {
        let mut next = InputState::new();
        next.keys_down.insert(KeyCode::F2);
        next.apply_action_press_for_key(KeyCode::F2);
        assert!(next.scene_next);
        assert!(!next.scene_prev);

        let mut prev = InputState::new();
        prev.keys_down.insert(KeyCode::ShiftLeft);
        prev.keys_down.insert(KeyCode::F2);
        prev.apply_action_press_for_key(KeyCode::F2);
        assert!(prev.scene_prev);
        assert!(!prev.scene_next);
    }

    #[test]
    fn save_template_requires_ctrl_or_super() {
        let mut plain_s = InputState::new();
        plain_s.keys_down.insert(KeyCode::KeyS);
        plain_s.apply_action_press_for_key(KeyCode::KeyS);
        assert!(!plain_s.save_ui_template);

        let mut ctrl_s = InputState::new();
        ctrl_s.keys_down.insert(KeyCode::ControlLeft);
        ctrl_s.keys_down.insert(KeyCode::KeyS);
        ctrl_s.apply_action_press_for_key(KeyCode::KeyS);
        assert!(ctrl_s.save_ui_template);
    }

    #[test]
    fn runtime_map_key_rebinds_world_move_left() {
        let mut state = InputState::new();
        assert_eq!(state.unmap_key(action::WORLD_MOVE_LEFT, KeyCode::KeyA), 1);
        state.map_key(action::WORLD_MOVE_LEFT, KeyCode::KeyJ);

        state.keys_down.insert(KeyCode::KeyJ);
        state.apply_action_press_for_key(KeyCode::KeyJ);
        assert!(state.world_move_left);
        assert!(state.action_pressed(action::WORLD_MOVE_LEFT));

        state.clear_frame();
        assert!(state.world_move_left);
        assert!(!state.action_pressed(action::WORLD_MOVE_LEFT));

        state.keys_down.remove(&KeyCode::KeyJ);
        state.recompute_action_down_states();
        state.sync_legacy_flags();
        assert!(!state.world_move_left);
    }

    #[test]
    fn custom_action_is_runtime_queryable() {
        let mut state = InputState::new();
        state.map_chord(
            "debug.toggle_freecam",
            KeyChord::new(KeyCode::KeyP).with_shift_required(),
        );
        state.keys_down.insert(KeyCode::ShiftLeft);
        state.keys_down.insert(KeyCode::KeyP);
        state.apply_action_press_for_key(KeyCode::KeyP);
        assert!(state.action_pressed("debug.toggle_freecam"));
        assert!(state.action_down("debug.toggle_freecam"));
    }

    #[test]
    fn apply_binding_change_supports_event_style_updates() {
        let mut state = InputState::new();
        let result = state.apply_binding_change(InputBindingChange::UnmapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: KeyCode::KeyA,
        });
        assert_eq!(result, InputBindingChangeResult::Applied);
        state.apply_binding_change(InputBindingChange::MapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: KeyCode::KeyJ,
        });

        state.keys_down.insert(KeyCode::KeyJ);
        state.recompute_action_down_states();
        state.apply_action_press_for_key(KeyCode::KeyJ);
        state.sync_legacy_flags();
        assert!(state.world_move_left);
    }

    #[test]
    fn apply_binding_changes_batches_operations() {
        let mut state = InputState::new();
        let applied = state.apply_binding_changes([
            InputBindingChange::UnmapKey {
                action: action::WORLD_MOVE_RIGHT.to_string(),
                key: KeyCode::KeyD,
            },
            InputBindingChange::MapChord {
                action: action::WORLD_MOVE_RIGHT.to_string(),
                chord: KeyChord::new(KeyCode::ArrowRight),
            },
        ]);
        assert_eq!(applied, 2);
        state.keys_down.insert(KeyCode::ArrowRight);
        state.recompute_action_down_states();
        state.sync_legacy_flags();
        assert!(state.world_move_right);
    }
}
