// Owner: Engine Input Plugin - Tests
use crate::plugins::{
    InputBindingChange, InputBindingChangeResult, InputState, KeyChord, TouchInputPhase, action,
};
use winit::event::{ElementState, MouseButton};
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

#[test]
fn cursor_motion_samples_preserve_all_positions_until_frame_end() {
    let mut state = InputState::new();

    state.handle_cursor_moved(10.0, 12.0);
    state.handle_cursor_moved(14.0, 15.0);
    state.handle_cursor_moved(21.0, 19.0);

    assert_eq!(
        state.mouse_motion_samples(),
        &[
            crate::plugins::MouseMotionSample {
                position: (10.0, 12.0),
                delta: (10.0, 12.0),
            },
            crate::plugins::MouseMotionSample {
                position: (14.0, 15.0),
                delta: (4.0, 3.0),
            },
            crate::plugins::MouseMotionSample {
                position: (21.0, 19.0),
                delta: (7.0, 4.0),
            },
        ]
    );
    assert_eq!(state.mouse_position, (21.0, 19.0));

    state.clear_frame();

    assert!(state.mouse_motion_samples().is_empty());
    assert_eq!(state.mouse_position, (21.0, 19.0));
}

#[test]
fn mouse_button_transitions_record_position_and_motion_sample_index() {
    let mut state = InputState::new();

    state.handle_cursor_moved(10.0, 12.0);
    state.handle_mouse_input(ElementState::Pressed, MouseButton::Left);
    state.handle_cursor_moved(14.0, 15.0);
    state.handle_mouse_input(ElementState::Released, MouseButton::Left);
    state.handle_cursor_moved(21.0, 19.0);

    let press = state
        .left_mouse_pressed_transition()
        .expect("left press transition should be recorded");
    assert!(press.is_left_pressed());
    assert_eq!(press.position, (10.0, 12.0));
    assert_eq!(
        press.motion_sample_index, 1,
        "press should remember how many motion samples happened before contact"
    );

    let release = state
        .left_mouse_released_transition()
        .expect("left release transition should be recorded");
    assert!(release.is_left_released());
    assert_eq!(release.position, (14.0, 15.0));
    assert_eq!(
        release.motion_sample_index, 2,
        "release should remember how many motion samples happened before release"
    );
    assert_eq!(state.mouse_button_transitions().len(), 2);

    state.clear_frame();

    assert!(state.mouse_button_transitions().is_empty());
    assert!(state.left_mouse_pressed_transition().is_none());
    assert!(state.left_mouse_released_transition().is_none());
}

#[test]
fn touch_samples_preserve_primary_pointer_history_until_frame_end() {
    let mut state = InputState::new();

    state.handle_touch_input(TouchInputPhase::Started, 7, 10.0, 12.0, Some(0.4));
    state.handle_touch_input(TouchInputPhase::Moved, 7, 14.0, 16.0, Some(0.5));
    state.handle_touch_input(TouchInputPhase::Started, 8, 50.0, 60.0, Some(0.8));
    state.handle_touch_input(TouchInputPhase::Moved, 7, 21.0, 20.0, Some(1.2));
    state.handle_touch_input(TouchInputPhase::Ended, 7, 25.0, 24.0, Some(0.0));

    assert_eq!(
        state.touch_samples(),
        &[
            crate::plugins::TouchInputSample {
                id: 7,
                phase: TouchInputPhase::Started,
                position: (10.0, 12.0),
                delta: (0.0, 0.0),
                pressure: Some(0.4),
            },
            crate::plugins::TouchInputSample {
                id: 7,
                phase: TouchInputPhase::Moved,
                position: (14.0, 16.0),
                delta: (4.0, 4.0),
                pressure: Some(0.5),
            },
            crate::plugins::TouchInputSample {
                id: 7,
                phase: TouchInputPhase::Moved,
                position: (21.0, 20.0),
                delta: (7.0, 4.0),
                pressure: Some(1.0),
            },
            crate::plugins::TouchInputSample {
                id: 7,
                phase: TouchInputPhase::Ended,
                position: (25.0, 24.0),
                delta: (4.0, 4.0),
                pressure: Some(0.0),
            },
        ],
        "only the first active touch should be routed as the primary drawing pointer"
    );

    state.clear_frame();

    assert!(state.touch_samples().is_empty());
}
