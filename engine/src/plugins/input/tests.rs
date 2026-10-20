// Owner: Engine Input Plugin - Tests
use crate::plugins::{
    ActionState, InputBindingChange, InputBindingChangeResult, InputState, KeyChord,
    TouchInputPhase, action,
};
use runen_input::{
    ContactId, ContactInput, ContactPhase, ContinuityLoss, CoordinateSpace, DigitalState,
    InputContext, InputDeviceId, InputObservation, InputObservationGroup, InputSourceId,
    KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey, ObservationOrigin,
    PhysicalKeyIdentity, Point2, PointerButton, PointerButtonInput, RelativeMotionUnit, Vector2,
};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

fn physical_key(key: KeyCode) -> PhysicalKeyIdentity {
    PhysicalKeyIdentity::code(format!("{key:?}"))
}

fn press_key(input: &mut InputState, actions: &mut ActionState, key: KeyCode) {
    input.handle_keyboard_input(key, ElementState::Pressed, None);
    actions.project(input);
}

fn release_key(input: &mut InputState, actions: &mut ActionState, key: KeyCode) {
    input.handle_keyboard_input(key, ElementState::Released, None);
    actions.project(input);
}

fn clear_frame(input: &mut InputState, actions: &mut ActionState) {
    actions.clear_frame(input);
    input.clear_frame();
}

fn normalized_key(state: DigitalState) -> KeyboardInput {
    KeyboardInput {
        physical_key: PhysicalKeyIdentity::code("KeyW"),
        logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
        location: KeyLocation::Standard,
        state,
        repeat: false,
        origin: ObservationOrigin::SourceReport,
    }
}

#[test]
fn default_bindings_split_enter_by_shift() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    press_key(&mut input, &mut actions, KeyCode::Enter);
    assert!(actions.action_pressed(action::UI_SUBMIT));
    assert!(!actions.action_pressed(action::UI_INSERT_NEWLINE));

    let mut input = InputState::new();
    let mut actions = ActionState::new();
    press_key(&mut input, &mut actions, KeyCode::ShiftLeft);
    press_key(&mut input, &mut actions, KeyCode::Enter);
    assert!(actions.action_pressed(action::UI_INSERT_NEWLINE));
    assert!(!actions.action_pressed(action::UI_SUBMIT));
}

#[test]
fn default_bindings_split_scene_f2_by_shift() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    press_key(&mut input, &mut actions, KeyCode::F2);
    assert!(actions.action_pressed(action::SCENE_NEXT));
    assert!(!actions.action_pressed(action::SCENE_PREV));

    let mut input = InputState::new();
    let mut actions = ActionState::new();
    press_key(&mut input, &mut actions, KeyCode::ShiftLeft);
    press_key(&mut input, &mut actions, KeyCode::F2);
    assert!(actions.action_pressed(action::SCENE_PREV));
    assert!(!actions.action_pressed(action::SCENE_NEXT));
}

#[test]
fn save_template_requires_ctrl_or_super() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    press_key(&mut input, &mut actions, KeyCode::KeyS);
    assert!(!actions.action_pressed(action::UI_SAVE_TEMPLATE));

    let mut input = InputState::new();
    let mut actions = ActionState::new();
    press_key(&mut input, &mut actions, KeyCode::ControlLeft);
    press_key(&mut input, &mut actions, KeyCode::KeyS);
    assert!(actions.action_pressed(action::UI_SAVE_TEMPLATE));
}

#[test]
fn press_time_modifiers_survive_later_modifier_release_before_projection() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();

    input.handle_keyboard_input(KeyCode::ShiftLeft, ElementState::Pressed, None);
    input.handle_keyboard_input(KeyCode::Enter, ElementState::Pressed, None);
    input.handle_keyboard_input(KeyCode::ShiftLeft, ElementState::Released, None);
    actions.project(&input);

    assert!(actions.action_pressed(action::UI_INSERT_NEWLINE));
    assert!(!actions.action_pressed(action::UI_SUBMIT));
}

#[test]
fn runtime_map_key_rebinds_world_move_left() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    assert_eq!(
        actions.unmap_key(
            &input,
            action::WORLD_MOVE_LEFT,
            &physical_key(KeyCode::KeyA)
        ),
        1
    );
    actions.map_key(&input, action::WORLD_MOVE_LEFT, physical_key(KeyCode::KeyJ));

    press_key(&mut input, &mut actions, KeyCode::KeyJ);
    assert!(actions.action_pressed(action::WORLD_MOVE_LEFT));
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));

    clear_frame(&mut input, &mut actions);
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));
    assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));

    release_key(&mut input, &mut actions, KeyCode::KeyJ);
    assert!(!actions.action_down(action::WORLD_MOVE_LEFT));
}

#[test]
fn binding_changes_while_held_recompute_down_without_fabricating_pressed() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();

    press_key(&mut input, &mut actions, KeyCode::KeyA);
    assert!(actions.action_pressed(action::WORLD_MOVE_LEFT));
    clear_frame(&mut input, &mut actions);

    assert_eq!(
        actions.unmap_key(
            &input,
            action::WORLD_MOVE_LEFT,
            &physical_key(KeyCode::KeyA)
        ),
        1
    );
    assert!(!actions.action_down(action::WORLD_MOVE_LEFT));
    assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));

    actions.map_key(&input, action::WORLD_MOVE_LEFT, physical_key(KeyCode::KeyA));
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));
    assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));

    actions.reset_default_bindings(&input);
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));
    assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));
}

#[test]
fn custom_action_is_runtime_queryable() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    actions.map_chord(
        &input,
        "debug.toggle_freecam",
        KeyChord::code("KeyP").with_shift_required(),
    );
    press_key(&mut input, &mut actions, KeyCode::ShiftLeft);
    press_key(&mut input, &mut actions, KeyCode::KeyP);
    assert!(actions.action_pressed("debug.toggle_freecam"));
    assert!(actions.action_down("debug.toggle_freecam"));
}

#[test]
fn apply_binding_change_supports_event_style_updates() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let result = actions.apply_binding_change(
        &input,
        InputBindingChange::UnmapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: physical_key(KeyCode::KeyA),
        },
    );
    assert_eq!(result, InputBindingChangeResult::Applied);
    actions.apply_binding_change(
        &input,
        InputBindingChange::MapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: physical_key(KeyCode::KeyJ),
        },
    );

    press_key(&mut input, &mut actions, KeyCode::KeyJ);
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));
}

#[test]
fn apply_binding_changes_batches_operations() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let applied = actions.apply_binding_changes(
        &input,
        [
            InputBindingChange::UnmapKey {
                action: action::WORLD_MOVE_RIGHT.to_string(),
                key: physical_key(KeyCode::KeyD),
            },
            InputBindingChange::MapChord {
                action: action::WORLD_MOVE_RIGHT.to_string(),
                chord: KeyChord::code("ArrowRight"),
            },
        ],
    );
    assert_eq!(applied, 2);
    press_key(&mut input, &mut actions, KeyCode::ArrowRight);
    assert!(actions.action_down(action::WORLD_MOVE_RIGHT));
}

#[test]
fn repeated_key_down_does_not_create_a_second_pressed_edge() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();

    press_key(&mut input, &mut actions, KeyCode::KeyW);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    clear_frame(&mut input, &mut actions);
    press_key(&mut input, &mut actions, KeyCode::KeyW);

    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn native_ui_denies_generationless_repeat_or_synthetic_hold_until_authentic_up() {
    for (source, repeat, origin) in [
        (101, true, ObservationOrigin::SourceReport),
        (
            102,
            false,
            ObservationOrigin::BackendSyntheticReconciliation,
        ),
    ] {
        let mut input = InputState::new();
        let mut actions = ActionState::new();
        let context = InputContext::new(InputSourceId::new(source), Some(InputDeviceId::new(9)));
        let unknown_down = KeyboardInput {
            repeat,
            origin,
            ..normalized_key(DigitalState::Pressed)
        };
        input.handle_normalized_keyboard(context, &unknown_down);
        assert_eq!(
            input.keyboard_down_generation_in(context, &unknown_down.physical_key),
            None,
            "unverified original press has no genuine Down generation"
        );
        // The generic legacy reducer deliberately retains neutral synthetic
        // held observations. A selected native UI recipient must deny this
        // very source/device/key without fabricating a real generation.
        actions.revoke_generationless_keyboard_lifetime(
            &input,
            context,
            &unknown_down.physical_key,
        );
        actions.project(&input);
        assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
        assert!(!actions.action_down(action::WORLD_MOVE_UP));

        // The denial is permanent for the physical lifetime. A repeat or
        // synthetic restoration cannot turn it into a permitted game hold.
        input.handle_normalized_keyboard(context, &unknown_down);
        actions.project(&input);
        assert!(!actions.action_down(action::WORLD_MOVE_UP));
        // Only a genuine OS Up retires the blocked identity.
        input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Released));
        actions.project(&input);
        assert!(!actions.action_down(action::WORLD_MOVE_UP));
        input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
        actions.project(&input);
        assert!(actions.action_pressed(action::WORLD_MOVE_UP));
        assert!(actions.action_down(action::WORLD_MOVE_UP));
    }
}

#[test]
fn normalized_repeat_metadata_cannot_create_a_pressed_edge() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(90), None);
    let first = normalized_key(DigitalState::Pressed);
    input.handle_normalized_keyboard(context, &first);
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));

    clear_frame(&mut input, &mut actions);
    let repeat = KeyboardInput {
        repeat: true,
        ..first
    };
    input.handle_normalized_keyboard(context, &repeat);
    actions.project(&input);

    assert!(actions.action_down(action::WORLD_MOVE_UP));
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
}

#[test]
fn device_scoped_key_transitions_have_independent_pressed_edges() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let source = InputSourceId::new(90);
    let context_a = InputContext::new(source, Some(InputDeviceId::new(1)));
    let context_b = InputContext::new(source, Some(InputDeviceId::new(2)));

    input.handle_normalized_keyboard(context_a, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(context_b, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    input.handle_normalized_keyboard(context_a, &normalized_key(DigitalState::Released));
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    input.handle_normalized_keyboard(context_b, &normalized_key(DigitalState::Released));
    actions.project(&input);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn source_scoped_modifiers_do_not_combine_across_native_windows() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let first = InputContext::new(InputSourceId::new(901), Some(InputDeviceId::new(1)));
    let second = InputContext::new(InputSourceId::new(902), Some(InputDeviceId::new(2)));

    let shift = KeyboardInput {
        physical_key: physical_key(KeyCode::ShiftLeft),
        ..normalized_key(DigitalState::Pressed)
    };
    let enter = KeyboardInput {
        physical_key: physical_key(KeyCode::Enter),
        ..normalized_key(DigitalState::Pressed)
    };
    input.handle_normalized_keyboard(first, &shift);
    input.handle_normalized_keyboard(second, &enter);
    actions.project(&input);
    assert!(actions.action_pressed(action::UI_SUBMIT));
    assert!(!actions.action_pressed(action::UI_INSERT_NEWLINE));
    assert!(actions.action_down(action::UI_SUBMIT));
    assert!(!actions.action_down(action::UI_INSERT_NEWLINE));

    // Binding mutations and FrameEnd recompute may not mix two source chords.
    actions.reset_default_bindings(&input);
    assert!(actions.action_down(action::UI_SUBMIT));
    assert!(!actions.action_down(action::UI_INSERT_NEWLINE));
    clear_frame(&mut input, &mut actions);
    assert!(actions.action_down(action::UI_SUBMIT));
}

#[test]
fn same_frame_source_local_down_up_down_preserves_both_physical_edges() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let first = InputContext::new(InputSourceId::new(903), Some(InputDeviceId::new(1)));
    let second = InputContext::new(InputSourceId::new(904), Some(InputDeviceId::new(2)));
    let down = normalized_key(DigitalState::Pressed);
    let up = normalized_key(DigitalState::Released);

    input.handle_normalized_keyboard(first, &down);
    input.handle_normalized_keyboard(first, &up);
    input.handle_normalized_keyboard(first, &down);
    input.handle_normalized_keyboard(second, &down);

    let samples = input.keyboard_press_samples();
    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].context, first);
    assert_eq!(samples[1].context, first);
    assert_eq!(samples[2].context, second);
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    input.handle_continuity_loss(first, ContinuityLoss::Source);
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));
    input.handle_normalized_keyboard(second, &up);
    actions.project(&input);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn key_release_cannot_inherit_modifier_from_another_source() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let first = InputContext::new(InputSourceId::new(905), None);
    let second = InputContext::new(InputSourceId::new(906), None);
    let shift = KeyboardInput {
        physical_key: physical_key(KeyCode::ShiftLeft),
        ..normalized_key(DigitalState::Pressed)
    };
    let enter = KeyboardInput {
        physical_key: physical_key(KeyCode::Enter),
        ..normalized_key(DigitalState::Pressed)
    };
    input.handle_normalized_keyboard(first, &shift);
    input.handle_normalized_keyboard(second, &enter);
    actions.project(&input);
    assert!(actions.action_down(action::UI_SUBMIT));
    input.handle_continuity_loss(second, ContinuityLoss::Source);
    actions.project(&input);
    assert!(!actions.action_down(action::UI_SUBMIT));
    assert!(!actions.action_down(action::UI_INSERT_NEWLINE));
}

#[test]
fn group_keyboard_projection_observes_prefix_modifiers_atomically() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(907), Some(InputDeviceId::new(3)));
    let shift = KeyboardInput {
        physical_key: physical_key(KeyCode::ShiftLeft),
        ..normalized_key(DigitalState::Pressed)
    };
    let enter = KeyboardInput {
        physical_key: physical_key(KeyCode::Enter),
        ..normalized_key(DigitalState::Pressed)
    };
    let shift_up = KeyboardInput {
        state: DigitalState::Released,
        ..shift.clone()
    };
    input
        .admit_device_observation_group(InputObservationGroup::new(
            context,
            vec![
                InputObservation::Keyboard(shift),
                InputObservation::Keyboard(enter),
                InputObservation::Keyboard(shift_up),
            ],
        ))
        .expect("admit one valid chronological source group");

    let samples = input.keyboard_press_samples();
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].context, context);
    assert_eq!(samples[1].context, context);
    actions.project(&input);
    assert!(actions.action_pressed(action::UI_INSERT_NEWLINE));
    assert!(!actions.action_pressed(action::UI_SUBMIT));
    // The admitted reducer has released Shift at the group end.
    assert!(actions.action_down(action::UI_SUBMIT));
    assert!(!actions.action_down(action::UI_INSERT_NEWLINE));
}

#[test]
fn keyboard_reconciliation_changes_held_state_without_pressed_edges() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();

    input.handle_keyboard_reconciliation(KeyCode::KeyW, ElementState::Pressed);
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));

    input.handle_keyboard_reconciliation(KeyCode::KeyW, ElementState::Released);
    actions.project(&input);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
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
fn repeated_mouse_down_does_not_create_a_second_pressed_edge() {
    let mut state = InputState::new();

    state.handle_mouse_input(ElementState::Pressed, MouseButton::Left);
    assert!(state.left_mouse_pressed());
    assert_eq!(state.mouse_button_transitions().len(), 1);

    state.clear_frame();
    state.handle_mouse_input(ElementState::Pressed, MouseButton::Left);

    assert!(!state.left_mouse_pressed());
    assert!(state.left_mouse_down());
    assert!(state.mouse_button_transitions().is_empty());
}

#[test]
fn device_scoped_button_state_keeps_legacy_edges_aggregate() {
    let mut state = InputState::new();
    let source = InputSourceId::new(91);
    let context_a = InputContext::new(source, Some(InputDeviceId::new(1)));
    let context_b = InputContext::new(source, Some(InputDeviceId::new(2)));
    let pressed = PointerButtonInput {
        button: PointerButton::Left,
        state: DigitalState::Pressed,
    };
    let released = PointerButtonInput {
        button: PointerButton::Left,
        state: DigitalState::Released,
    };

    state.handle_pointer_button(context_a, pressed);
    assert!(state.left_mouse_pressed());
    assert!(state.left_mouse_down());

    state.clear_frame();
    state.handle_pointer_button(context_b, pressed);
    assert!(!state.left_mouse_pressed());
    assert!(state.left_mouse_down());
    assert!(state.mouse_button_transitions().is_empty());

    state.handle_pointer_button(context_a, released);
    assert!(!state.left_mouse_released());
    assert!(state.left_mouse_down());

    state.handle_pointer_button(context_b, released);
    assert!(state.left_mouse_released());
    assert!(!state.left_mouse_down());
}

#[test]
fn touch_samples_preserve_primary_projection_while_neutral_state_keeps_all_contacts() {
    let mut state = InputState::new();

    state.handle_touch_input(TouchInputPhase::Started, 7, 10.0, 12.0, Some(0.4));
    state.handle_touch_input(TouchInputPhase::Moved, 7, 14.0, 16.0, Some(0.5));
    state.handle_touch_input(TouchInputPhase::Started, 8, 50.0, 60.0, Some(0.8));
    assert!(state.neutral_touch_active(7));
    assert!(state.neutral_touch_active(8));

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
        "legacy drawing projection should remain single-primary while neutral state keeps all contacts"
    );
    assert!(!state.neutral_touch_active(7));
    assert!(state.neutral_touch_active(8));

    state.clear_frame();

    assert!(state.touch_samples().is_empty());
    assert!(state.neutral_touch_active(8));
}

#[test]
fn frame_clear_keeps_durable_neutral_and_action_held_state() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();

    press_key(&mut input, &mut actions, KeyCode::KeyD);
    input.handle_mouse_input(ElementState::Pressed, MouseButton::Left);
    input.handle_touch_input(TouchInputPhase::Started, 11, 4.0, 5.0, None);
    clear_frame(&mut input, &mut actions);

    assert!(actions.action_down(action::WORLD_MOVE_RIGHT));
    assert!(input.left_mouse_down());
    assert!(input.neutral_touch_active(11));
    assert!(!actions.action_pressed(action::WORLD_MOVE_RIGHT));
    assert!(!input.left_mouse_pressed());
}

#[test]
fn continuity_loss_invalidates_confirmed_state_without_fabricating_edges() {
    let mut input = InputState::new();
    let context = InputContext::new(InputSourceId::new(700), Some(InputDeviceId::new(11)));

    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    input.handle_pointer_button(
        context,
        PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Pressed,
        },
    );
    input.clear_frame();

    input.handle_continuity_loss(context, ContinuityLoss::Source);

    assert!(!input.physical_key_down(&PhysicalKeyIdentity::code("KeyW")));
    assert!(!input.left_mouse_down());
    assert!(!input.left_mouse_released());
    assert!(input.mouse_button_transitions().is_empty());
    assert!(input.keyboard_press_samples().is_empty());
}

#[test]
fn source_continuity_loss_is_scoped_and_releases_stale_primary_touch_ownership() {
    let mut input = InputState::new();
    let lost = InputContext::new(InputSourceId::new(701), Some(InputDeviceId::new(21)));
    let retained = InputContext::new(InputSourceId::new(702), Some(InputDeviceId::new(22)));

    input.handle_normalized_keyboard(lost, &normalized_key(DigitalState::Pressed));
    input.handle_normalized_keyboard(
        retained,
        &KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyD"),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: DigitalState::Pressed,
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        },
    );
    input.handle_contact_input(
        lost,
        &ContactInput {
            contact: ContactId::new(1),
            phase: ContactPhase::Begin,
            position: Point2::new(10.0, 10.0, CoordinateSpace::WindowPhysicalPixels),
            pressure: None,
            altitude_angle_radians: None,
        },
    );
    input.clear_frame();

    input.handle_continuity_loss(lost, ContinuityLoss::Source);

    assert!(!input.physical_key_down(&PhysicalKeyIdentity::code("KeyW")));
    assert!(input.physical_key_down(&PhysicalKeyIdentity::code("KeyD")));

    input.handle_contact_input(
        retained,
        &ContactInput {
            contact: ContactId::new(2),
            phase: ContactPhase::Begin,
            position: Point2::new(20.0, 20.0, CoordinateSpace::WindowPhysicalPixels),
            pressure: None,
            altitude_angle_radians: None,
        },
    );

    assert_eq!(input.touch_samples().len(), 1);
    assert_eq!(input.touch_samples()[0].id, 2);
}

#[test]
fn admitted_input_capture_is_opt_in_and_preserves_admission_order_across_frames() {
    let mut input = InputState::new();

    input.handle_mouse_input(ElementState::Pressed, MouseButton::Left);
    assert!(input.drain_admitted_input_capture().is_empty());

    input.start_admitted_input_capture();
    assert!(input.admitted_input_capture_active());

    input.handle_mouse_input(ElementState::Pressed, MouseButton::Right);
    input.handle_mouse_motion(3.0, -2.0);
    input.clear_frame();
    input.handle_mouse_wheel_delta(1.5);

    let captured = input.drain_admitted_input_capture();
    assert_eq!(captured.len(), 3);
    assert!(matches!(
        captured[0].observations.as_slice(),
        [InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Right,
            state: DigitalState::Pressed,
        })]
    ));
    assert!(matches!(
        captured[1].observations.as_slice(),
        [InputObservation::RelativeMotion {
            delta: Vector2 { x: 3.0, y: -2.0 },
            unit: RelativeMotionUnit::BackendDeviceUnits,
        }]
    ));
    assert!(matches!(
        captured[2].observations.as_slice(),
        [InputObservation::Scroll(_)]
    ));
    assert!(
        input.admitted_input_capture_active(),
        "draining capture must not implicitly stop a still-active recorder"
    );

    input.reset_admitted_input_capture();
    input.handle_mouse_input(ElementState::Released, MouseButton::Right);
    let stopped = input.stop_admitted_input_capture();
    assert_eq!(stopped.len(), 1);
    assert!(!input.admitted_input_capture_active());

    input.handle_mouse_motion(9.0, 9.0);
    assert!(input.drain_admitted_input_capture().is_empty());
}

#[test]
fn rejected_group_is_not_captured_or_staged() {
    let mut input = InputState::new();
    input.start_admitted_input_capture();
    let group = InputObservationGroup::single(
        InputContext::new(InputSourceId::new(711), Some(InputDeviceId::new(9))),
        InputObservation::RelativeMotion {
            delta: Vector2::new(f32::NAN, 0.0),
            unit: RelativeMotionUnit::BackendDeviceUnits,
        },
    );

    assert!(input.admit_device_observation_group(group).is_err());
    assert!(input.drain_admitted_input_capture().is_empty());
    assert!(input.drain_device_observation_groups().is_empty());
}

#[test]
fn continuity_loss_is_captured_as_continuity_without_fabricated_release() {
    let mut input = InputState::new();
    let context = InputContext::new(InputSourceId::new(712), Some(InputDeviceId::new(10)));
    input.handle_pointer_button(
        context,
        PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Pressed,
        },
    );
    input.clear_frame();

    input.start_admitted_input_capture();
    input.handle_continuity_loss(context, ContinuityLoss::Source);

    let captured = input.stop_admitted_input_capture();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].context, context);
    assert_eq!(
        captured[0].observations,
        vec![InputObservation::ContinuityLoss(ContinuityLoss::Source)]
    );
    assert!(!input.left_mouse_released());
    assert!(input.mouse_button_transitions().is_empty());
}

#[test]
fn automation_absolute_pointer_uses_same_cursor_projection_as_native_path() {
    let mut state = InputState::new();
    let context = InputContext::new(InputSourceId::new(310), None);

    assert!(
        state
            .admit_automation_observation(
                context,
                InputObservation::AbsolutePointerPosition {
                    position: Point2::new(10.0, 20.0, CoordinateSpace::WindowPhysicalPixels,),
                },
            )
            .unwrap()
    );
    assert!(
        state
            .admit_automation_observation(
                context,
                InputObservation::AbsolutePointerPosition {
                    position: Point2::new(13.0, 26.0, CoordinateSpace::WindowPhysicalPixels,),
                },
            )
            .unwrap()
    );

    assert_eq!(state.mouse_position, (13.0, 26.0));
    assert_eq!(
        state.mouse_motion_samples(),
        &[
            crate::plugins::MouseMotionSample {
                position: (10.0, 20.0),
                delta: (10.0, 20.0),
            },
            crate::plugins::MouseMotionSample {
                position: (13.0, 26.0),
                delta: (3.0, 6.0),
            },
        ]
    );
}

#[test]
fn native_ui_exclusive_down_revocation_immediately_removes_pressed_and_held() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(311), Some(InputDeviceId::new(7)));
    let key = physical_key(KeyCode::KeyW);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
    let generation = input
        .keyboard_down_generation_in(context, &key)
        .expect("genuine down has a unique generation");

    actions.revoke_keyboard_lifetime(&input, context, &key, generation);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    // UI focus leaving, repeated projection and every normal frame boundary
    // cannot restore an already exclusive/denied physical Down.
    actions.project(&input);
    clear_frame(&mut input, &mut actions);
    actions.project(&input);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    let duplicate = KeyboardInput {
        repeat: true,
        ..normalized_key(DigitalState::Pressed)
    };
    input.handle_normalized_keyboard(context, &duplicate);
    actions.project(&input);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    // Genuine release, followed by a fresh real Down, alone creates another
    // gameplay-eligible physical lifetime.
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Released));
    actions.project(&input);
    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    let next = input
        .keyboard_down_generation_in(context, &key)
        .expect("fresh real down");
    assert_ne!(generation, next);
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn revoked_down_does_not_reappear_through_binding_changes_or_other_devices() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let first = InputContext::new(InputSourceId::new(312), Some(InputDeviceId::new(1)));
    let second = InputContext::new(InputSourceId::new(313), Some(InputDeviceId::new(2)));
    let key = physical_key(KeyCode::KeyW);
    input.handle_normalized_keyboard(first, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    let generation = input.keyboard_down_generation_in(first, &key).unwrap();
    actions.revoke_keyboard_lifetime(&input, first, &key, generation);

    actions.unmap_key(&input, action::WORLD_MOVE_UP, &key);
    actions.map_key(&input, action::WORLD_MOVE_UP, key.clone());
    actions.reset_default_bindings(&input);
    actions.apply_binding_change(
        &input,
        InputBindingChange::MapKey {
            action: "native.test".to_string(),
            key: key.clone(),
        },
    );
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    assert!(!actions.action_down("native.test"));

    // A different window/device owns a distinct Down generation and remains
    // capable of contributing without releasing the first device's denial.
    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(second, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
    input.handle_normalized_keyboard(second, &normalized_key(DigitalState::Released));
    actions.project(&input);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn genuine_release_required_after_synthetic_cancellation_of_revoked_down() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(314), Some(InputDeviceId::new(9)));
    let key = physical_key(KeyCode::KeyW);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    let generation = input.keyboard_down_generation_in(context, &key).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &key, generation);
    let synthetic_up = KeyboardInput {
        origin: ObservationOrigin::BackendSyntheticReconciliation,
        ..normalized_key(DigitalState::Released)
    };
    input.handle_normalized_keyboard(context, &synthetic_up);
    clear_frame(&mut input, &mut actions);
    let synthetic_down = KeyboardInput {
        origin: ObservationOrigin::BackendSyntheticReconciliation,
        ..normalized_key(DigitalState::Pressed)
    };
    input.handle_normalized_keyboard(context, &synthetic_down);
    actions.project(&input);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    // A real Up, rather than an OS reconciliation edge, unlocks a later Down.
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Released));
    actions.project(&input);
    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn revocation_preserves_other_source_pressed_contributors_without_rebinding_replay() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let one = InputContext::new(InputSourceId::new(315), Some(InputDeviceId::new(1)));
    let two = InputContext::new(InputSourceId::new(316), Some(InputDeviceId::new(2)));
    let key = physical_key(KeyCode::KeyW);

    input.handle_normalized_keyboard(one, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    let denied = input.keyboard_down_generation_in(one, &key).unwrap();
    input.handle_normalized_keyboard(two, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    actions.revoke_keyboard_lifetime(&input, one, &key, denied);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    // Once the second source is denied, no pressed or held contributors
    // remain; binding edits must never reinterpret old presses as new.
    let second_generation = input.keyboard_down_generation_in(two, &key).unwrap();
    actions.revoke_keyboard_lifetime(&input, two, &key, second_generation);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    actions.map_key(&input, "new.mapping", key);
    assert!(!actions.action_pressed("new.mapping"));
    assert!(!actions.action_down("new.mapping"));
}

#[test]
fn late_m13_source_denial_retracts_projected_presses_and_held_without_cross_source_loss() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let ui = InputContext::new(InputSourceId::new(901), Some(InputDeviceId::new(21)));
    let game = InputContext::new(InputSourceId::new(902), Some(InputDeviceId::new(22)));
    let w = physical_key(KeyCode::KeyW);
    let a = physical_key(KeyCode::KeyA);

    input.handle_normalized_keyboard(ui, &normalized_key(DigitalState::Pressed));
    input.handle_normalized_keyboard(
        ui,
        &KeyboardInput {
            physical_key: a,
            ..normalized_key(DigitalState::Pressed)
        },
    );
    input.handle_normalized_keyboard(
        game,
        &KeyboardInput {
            physical_key: physical_key(KeyCode::KeyD),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));
    assert!(actions.action_down(action::WORLD_MOVE_RIGHT));

    // Caller has certified a new incompatible M13 ownership boundary for
    // the exact UI-native source; the one gameplay owner acts synchronously.
    actions.revoke_source_keyboard_for_ui_ownership(&input, ui.source);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_LEFT));
    assert!(actions.action_down(action::WORLD_MOVE_RIGHT));
    actions.revoke_source_keyboard_for_ui_ownership(&input, ui.source);

    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(
        ui,
        &KeyboardInput {
            repeat: true,
            ..normalized_key(DigitalState::Pressed)
        },
    );
    actions.map_key(&input, "new.w", w);
    actions.project(&input);
    assert!(!actions.action_down("new.w"));
    assert!(actions.action_down(action::WORLD_MOVE_RIGHT));

    input.handle_normalized_keyboard(ui, &normalized_key(DigitalState::Released));
    actions.project(&input);
    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(ui, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down("new.w"));
}

#[test]
fn late_m13_source_denial_removes_already_released_same_frame_press() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let ui = InputContext::new(InputSourceId::new(903), Some(InputDeviceId::new(23)));
    let game = InputContext::new(InputSourceId::new(904), Some(InputDeviceId::new(24)));

    input.handle_normalized_keyboard(ui, &normalized_key(DigitalState::Pressed));
    input.handle_normalized_keyboard(ui, &normalized_key(DigitalState::Released));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    actions.revoke_source_keyboard_for_ui_ownership(&input, ui.source);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    input.handle_normalized_keyboard(game, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn late_m13_denial_replaces_retired_block_after_up_and_new_down() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let source = InputContext::new(InputSourceId::new(905), Some(InputDeviceId::new(25)));
    let key = physical_key(KeyCode::KeyW);

    input.handle_normalized_keyboard(source, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    let first = input.keyboard_down_generation_in(source, &key).unwrap();
    actions.revoke_keyboard_lifetime(&input, source, &key, first);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    // This genuine Up and fresh Down happen without an intervening
    // ActionState projection. The previously blocked generation must be
    // retired before a new M13 conflict denies the new physical lifetime.
    input.handle_normalized_keyboard(source, &normalized_key(DigitalState::Released));
    input.handle_normalized_keyboard(source, &normalized_key(DigitalState::Pressed));
    let second = input.keyboard_down_generation_in(source, &key).unwrap();
    assert_ne!(first, second);
    actions.revoke_source_keyboard_for_ui_ownership(&input, source.source);
    actions.project(&input);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    clear_frame(&mut input, &mut actions);
    assert!(
        !actions.action_down(action::WORLD_MOVE_UP),
        "the new denied generation must remain blocked after FrameEnd"
    );

    input.handle_normalized_keyboard(source, &normalized_key(DigitalState::Released));
    actions.project(&input);
    clear_frame(&mut input, &mut actions);
    input.handle_normalized_keyboard(source, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn ui_owned_modifier_cannot_keep_a_game_chord_held() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(317), Some(InputDeviceId::new(4)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.shift-w",
        KeyChord::code("KeyW").with_shift_required(),
    );
    let shift_down = KeyboardInput {
        physical_key: shift.clone(),
        ..normalized_key(DigitalState::Pressed)
    };
    input.handle_normalized_keyboard(context, &shift_down);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_down("game.shift-w"));

    let generation = input
        .keyboard_down_generation_in(context, &shift)
        .expect("real modifier Down");
    actions.revoke_keyboard_lifetime(&input, context, &shift, generation);
    assert!(!actions.action_down("game.shift-w"));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
    clear_frame(&mut input, &mut actions);
    assert!(!actions.action_down("game.shift-w"));
}

#[test]
fn delayed_old_m13_denial_never_claims_new_physical_down_generation() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(318), Some(InputDeviceId::new(5)));
    let key = physical_key(KeyCode::KeyW);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    let old_generation = input.keyboard_down_generation_in(context, &key).unwrap();
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Released));
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    let new_generation = input.keyboard_down_generation_in(context, &key).unwrap();
    assert_ne!(old_generation, new_generation);
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    // A late exact M13 receipt for the first generation may cancel that
    // press, never a later independent genuine physical Down.
    actions.revoke_keyboard_lifetime(&input, context, &key, old_generation);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn source_continuity_retracts_only_affected_action_presses_even_after_projection() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let old = InputContext::new(InputSourceId::new(410), Some(InputDeviceId::new(1)));
    let other = InputContext::new(InputSourceId::new(411), Some(InputDeviceId::new(2)));

    input.handle_normalized_keyboard(old, &normalized_key(DigitalState::Pressed));
    input.handle_normalized_keyboard(
        other,
        &KeyboardInput {
            physical_key: physical_key(KeyCode::KeyA),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    actions.project(&input);
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_pressed(action::WORLD_MOVE_LEFT));

    // A source invalidation, not an authentic Up, removes only that source's
    // pending and already-projected action contribution.
    input.handle_continuity_loss(old, ContinuityLoss::Source);
    actions.project(&input);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    assert!(actions.action_pressed(action::WORLD_MOVE_LEFT));
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));

    actions.retire_native_keyboard_source(&input, old.source);
    clear_frame(&mut input, &mut actions);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_LEFT));
}

#[test]
fn continuity_before_projection_does_not_replay_old_source_pressed() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let old = InputContext::new(InputSourceId::new(412), Some(InputDeviceId::new(3)));
    input.handle_normalized_keyboard(old, &normalized_key(DigitalState::Pressed));
    input.handle_continuity_loss(old, ContinuityLoss::Source);
    actions.project(&input);
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn actual_native_source_retirement_frees_denial_but_focus_loss_does_not() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let old = InputContext::new(InputSourceId::new(413), Some(InputDeviceId::new(3)));
    let key = physical_key(KeyCode::KeyW);
    input.handle_normalized_keyboard(old, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    let generation = input.keyboard_down_generation_in(old, &key).unwrap();
    actions.revoke_keyboard_lifetime(&input, old, &key, generation);
    input.handle_continuity_loss(old, ContinuityLoss::Source);
    clear_frame(&mut input, &mut actions);

    // Source still exists after native focus loss. Even a synthetic press
    // cannot resurrect a previously revoked physical lifetime.
    input.handle_normalized_keyboard(
        old,
        &KeyboardInput {
            origin: ObservationOrigin::BackendSyntheticReconciliation,
            ..normalized_key(DigitalState::Pressed)
        },
    );
    actions.project(&input);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));

    // Destruction uses monotonic never-reused source identity and reclaims
    // denials; other sources are unaffected.
    input.handle_continuity_loss(old, ContinuityLoss::Source);
    actions.retire_native_keyboard_source(&input, old.source);
    clear_frame(&mut input, &mut actions);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
}

#[test]
fn ui_owned_shift_never_authorizes_a_later_shift_required_press_or_forbidden_branch() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(421), Some(InputDeviceId::new(1)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.shift-w",
        KeyChord::code("KeyW").with_shift_required(),
    );
    actions.map_chord(
        &input,
        "game.unshifted-w",
        KeyChord::code("KeyW").with_shift_forbidden(),
    );
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    let shift_generation = input.keyboard_down_generation_in(context, &shift).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &shift, shift_generation);
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);

    assert!(!actions.action_pressed("game.shift-w"));
    assert!(!actions.action_pressed("game.unshifted-w"));
    assert!(!actions.action_down("game.shift-w"));
    assert!(!actions.action_down("game.unshifted-w"));
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
}

#[test]
fn late_ui_modifier_claim_retracts_exact_prior_chord_and_keeps_other_sources() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let one = InputContext::new(InputSourceId::new(422), Some(InputDeviceId::new(2)));
    let two = InputContext::new(InputSourceId::new(423), Some(InputDeviceId::new(3)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.shift-w",
        KeyChord::code("KeyW").with_shift_required(),
    );
    for context in [one, two] {
        input.handle_normalized_keyboard(
            context,
            &KeyboardInput {
                physical_key: shift.clone(),
                ..normalized_key(DigitalState::Pressed)
            },
        );
        input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    }
    actions.project(&input);
    assert!(actions.action_pressed("game.shift-w"));
    let first_gen = input.keyboard_down_generation_in(one, &shift).unwrap();
    actions.revoke_keyboard_lifetime(&input, one, &shift, first_gen);
    assert!(
        actions.action_pressed("game.shift-w"),
        "second source still qualifies"
    );
    let second_gen = input.keyboard_down_generation_in(two, &shift).unwrap();
    actions.revoke_keyboard_lifetime(&input, two, &shift, second_gen);
    assert!(
        !actions.action_pressed("game.shift-w"),
        "both exact supporting modifiers denied"
    );
    assert!(actions.action_pressed(action::WORLD_MOVE_UP));
    actions.map_chord(
        &input,
        "late.mapping",
        KeyChord::code("KeyW").with_shift_required(),
    );
    assert!(
        !actions.action_pressed("late.mapping"),
        "binding changes cannot replay rejected press"
    );
}

#[test]
fn one_of_two_shift_keys_still_qualifies_required_chord_after_other_is_claimed() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(424), Some(InputDeviceId::new(4)));
    let left = physical_key(KeyCode::ShiftLeft);
    let right = physical_key(KeyCode::ShiftRight);
    actions.map_chord(
        &input,
        "game.shift-w",
        KeyChord::code("KeyW").with_shift_required(),
    );
    for shift in [&left, &right] {
        input.handle_normalized_keyboard(
            context,
            &KeyboardInput {
                physical_key: shift.clone(),
                ..normalized_key(DigitalState::Pressed)
            },
        );
    }
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    let left_generation = input.keyboard_down_generation_in(context, &left).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &left, left_generation);
    assert!(
        actions.action_pressed("game.shift-w"),
        "right Shift independently qualifies"
    );
    let right_generation = input.keyboard_down_generation_in(context, &right).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &right, right_generation);
    assert!(!actions.action_pressed("game.shift-w"));
}

#[test]
fn late_old_modifier_receipt_cannot_invalidate_chord_with_new_generation() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(425), Some(InputDeviceId::new(5)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.shift-w",
        KeyChord::code("KeyW").with_shift_required(),
    );
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    let original = input.keyboard_down_generation_in(context, &shift).unwrap();
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Released)
        },
    );
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    actions.revoke_keyboard_lifetime(&input, context, &shift, original);
    assert!(actions.action_pressed("game.shift-w"));
    assert!(actions.action_down("game.shift-w"));
}

#[test]
fn claimed_ui_modifier_cannot_fabricate_a_forbidden_held_game_chord() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(426), Some(InputDeviceId::new(6)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.shift-w",
        KeyChord::code("KeyW").with_shift_required(),
    );
    actions.map_chord(
        &input,
        "game.no-shift-w",
        KeyChord::code("KeyW").with_shift_forbidden(),
    );
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_down("game.shift-w"));
    assert!(!actions.action_down("game.no-shift-w"));

    let shift_generation = input.keyboard_down_generation_in(context, &shift).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &shift, shift_generation);
    assert!(!actions.action_pressed("game.shift-w"));
    assert!(!actions.action_down("game.shift-w"));
    assert!(!actions.action_down("game.no-shift-w"));

    // A new binding cannot use denied Shift as either positive or
    // negative modifier evidence. A separate window is unaffected.
    actions.map_chord(
        &input,
        "game.added-after-denial",
        KeyChord::code("KeyW").with_shift_forbidden(),
    );
    assert!(!actions.action_down("game.added-after-denial"));
    clear_frame(&mut input, &mut actions);
    assert!(!actions.action_down("game.no-shift-w"));

    let other = InputContext::new(InputSourceId::new(427), Some(InputDeviceId::new(7)));
    input.handle_normalized_keyboard(other, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(actions.action_down("game.no-shift-w"));
    assert!(actions.action_down("game.added-after-denial"));
}

#[test]
fn genuine_modifier_release_allows_forbidden_held_chord_without_new_press() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(428), Some(InputDeviceId::new(8)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.no-shift-w",
        KeyChord::code("KeyW").with_shift_forbidden(),
    );
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    let generation = input.keyboard_down_generation_in(context, &shift).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &shift, generation);
    actions.project(&input);
    assert!(!actions.action_down("game.no-shift-w"));

    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift,
            ..normalized_key(DigitalState::Released)
        },
    );
    actions.project(&input);
    assert!(actions.action_down("game.no-shift-w"));
    assert!(
        !actions.action_pressed("game.no-shift-w"),
        "changing held modifier truth must not fabricate a fresh W press"
    );
}

#[test]
fn synthetic_modifier_release_cannot_fabricate_forbidden_pressed_or_held_shortcuts() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let context = InputContext::new(InputSourceId::new(429), Some(InputDeviceId::new(9)));
    let shift = physical_key(KeyCode::ShiftLeft);
    actions.map_chord(
        &input,
        "game.no-shift-w",
        KeyChord::code("KeyW").with_shift_forbidden(),
    );
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            ..normalized_key(DigitalState::Pressed)
        },
    );
    let shift_generation = input.keyboard_down_generation_in(context, &shift).unwrap();
    actions.revoke_keyboard_lifetime(&input, context, &shift, shift_generation);
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift.clone(),
            origin: ObservationOrigin::BackendSyntheticReconciliation,
            ..normalized_key(DigitalState::Released)
        },
    );
    input.handle_normalized_keyboard(context, &normalized_key(DigitalState::Pressed));
    actions.project(&input);
    assert!(!actions.action_pressed("game.no-shift-w"));
    assert!(!actions.action_down("game.no-shift-w"));

    clear_frame(&mut input, &mut actions);
    actions.project(&input);
    assert!(!actions.action_pressed("game.no-shift-w"));
    assert!(!actions.action_down("game.no-shift-w"));

    // Only a genuine Up resolves the denied lifetime's uncertainty.
    input.handle_normalized_keyboard(
        context,
        &KeyboardInput {
            physical_key: shift,
            ..normalized_key(DigitalState::Released)
        },
    );
    actions.project(&input);
    assert!(actions.action_down("game.no-shift-w"));
    assert!(!actions.action_pressed("game.no-shift-w"));
}

#[test]
fn native_ui_pointer_truth_does_not_leak_into_legacy_gameplay_projection() {
    use crate::plugins::ui::NativeInputRecipient;
    use crate::runtime::platform::{PlatformEvent, apply_native_recipient_input_event};
    use runen_input::{ScrollDelta, ScrollDomain, ScrollInput};

    let legacy = InputContext::new(InputSourceId::new(801), Some(InputDeviceId::new(11)));
    let ui = InputContext::new(InputSourceId::new(802), Some(InputDeviceId::new(12)));
    let legacy_route = Some(NativeInputRecipient::LegacyNativeConsumer);
    let ui_route = None;
    let left_down = |context| PlatformEvent::MouseInput {
        context,
        input: PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Pressed,
        },
    };
    let left_up = |context| PlatformEvent::MouseInput {
        context,
        input: PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Released,
        },
    };

    let mut input = InputState::new();
    input.start_admitted_input_capture();
    apply_native_recipient_input_event(&mut input, &left_down(legacy), legacy_route);
    assert!(input.left_mouse_down());
    assert!(input.left_mouse_pressed());
    input.clear_frame();

    // A UI Down must be neutral physical truth but not a second legacy press.
    apply_native_recipient_input_event(&mut input, &left_down(ui), ui_route);
    assert!(input.pointer_button_down_anywhere(PointerButton::Left));
    assert!(
        input.left_mouse_down(),
        "the independent legacy held source survives"
    );
    assert!(!input.left_mouse_pressed());
    assert!(input.mouse_button_transitions().is_empty());

    // UI cursor/wheel/touch do not move the gameplay cursor or steal the
    // legacy primary contact. Original source-local observations remain.
    apply_native_recipient_input_event(
        &mut input,
        &PlatformEvent::CursorMoved {
            context: legacy,
            position: Point2::new(10.0, 20.0, CoordinateSpace::WindowPhysicalPixels),
        },
        legacy_route,
    );
    apply_native_recipient_input_event(
        &mut input,
        &PlatformEvent::MouseWheel {
            context: legacy,
            input: ScrollInput {
                delta: ScrollDelta::vertical_only(1.0),
                domain: ScrollDomain::Unspecified,
                phase: None,
            },
        },
        legacy_route,
    );
    assert_eq!(input.mouse_position, (10.0, 20.0));
    assert_eq!(input.scroll_delta, 1.0);
    apply_native_recipient_input_event(
        &mut input,
        &PlatformEvent::CursorMoved {
            context: ui,
            position: Point2::new(99.0, 88.0, CoordinateSpace::WindowPhysicalPixels),
        },
        ui_route,
    );
    apply_native_recipient_input_event(
        &mut input,
        &PlatformEvent::MouseWheel {
            context: ui,
            input: ScrollInput {
                delta: ScrollDelta::vertical_only(7.0),
                domain: ScrollDomain::Unspecified,
                phase: None,
            },
        },
        ui_route,
    );
    apply_native_recipient_input_event(
        &mut input,
        &PlatformEvent::Touch {
            context: ui,
            input: ContactInput {
                contact: ContactId::new(33),
                phase: ContactPhase::Begin,
                position: Point2::new(40.0, 50.0, CoordinateSpace::WindowPhysicalPixels),
                pressure: None,
                altitude_angle_radians: None,
            },
        },
        ui_route,
    );
    assert_eq!(input.mouse_position, (10.0, 20.0));
    assert_eq!(input.scroll_delta, 1.0);
    assert!(input.touch_samples().is_empty());

    // A genuine legacy release ends only the legacy game held projection,
    // even when the UI's separate source remains physically pressed.
    input.clear_frame();
    apply_native_recipient_input_event(&mut input, &left_up(legacy), legacy_route);
    assert!(!input.left_mouse_down());
    assert!(input.left_mouse_released());
    assert!(
        input.pointer_button_down_anywhere(PointerButton::Left),
        "the UI source still owns its independently admitted physical Down"
    );
    input.clear_frame();
    apply_native_recipient_input_event(&mut input, &left_up(ui), ui_route);
    assert!(!input.left_mouse_down());
    assert!(!input.left_mouse_released());

    // Source retirement must clear derived legacy held eligibility without
    // fabricating a mouse-button release or erasing unrelated physical truth.
    apply_native_recipient_input_event(&mut input, &left_down(legacy), legacy_route);
    input.clear_frame();
    apply_native_recipient_input_event(
        &mut input,
        &PlatformEvent::InputContinuityLost {
            context: legacy,
            loss: ContinuityLoss::Device,
        },
        ui_route,
    );
    assert!(!input.left_mouse_down());
    assert!(!input.left_mouse_released());
    assert!(input.mouse_button_transitions().is_empty());
    let captured = input.stop_admitted_input_capture();
    assert!(captured.iter().any(|group| group.context == ui
        && matches!(
            group.observations.as_slice(),
            [InputObservation::Contact(_)]
        )));
}

#[test]
fn native_duplicate_retains_only_eligible_same_source_down_generation() {
    let mut input = InputState::new();
    let mut actions = ActionState::new();
    let a = InputContext::new(InputSourceId::new(177), Some(InputDeviceId::new(31)));
    let b = InputContext::new(InputSourceId::new(178), Some(InputDeviceId::new(32)));
    let key = physical_key(KeyCode::KeyW);
    assert!(!actions.original_native_held_generation_remains_eligible(&input, a, &key, None));
    input.handle_normalized_keyboard(a, &normalized_key(DigitalState::Pressed));
    let first = input.keyboard_down_generation_in(a, &key);
    assert!(first.is_some());
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));
    assert!(actions.original_native_held_generation_remains_eligible(&input, a, &key, first));
    assert!(!actions.original_native_held_generation_remains_eligible(&input, b, &key, first));
    // Actual neutral repeated observation does not mint a second Down.
    let mut repeat = normalized_key(DigitalState::Pressed);
    repeat.repeat = true;
    input.handle_normalized_keyboard(a, &repeat);
    assert!(actions.original_native_held_generation_remains_eligible(&input, a, &key, first));
    actions.revoke_source_keyboard_for_ui_ownership(&input, a.source);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
    assert!(!actions.original_native_held_generation_remains_eligible(&input, a, &key, first));
    // Only a genuine Up followed by a fresh Down may create a new eligible
    // generation, and it can never match the previously retained generation.
    input.handle_normalized_keyboard(a, &normalized_key(DigitalState::Released));
    actions.project(&input);
    input.handle_normalized_keyboard(a, &normalized_key(DigitalState::Pressed));
    let next = input.keyboard_down_generation_in(a, &key);
    assert_ne!(next, first);
    assert!(!actions.original_native_held_generation_remains_eligible(&input, a, &key, first));
    assert!(actions.original_native_held_generation_remains_eligible(&input, a, &key, next));
}
