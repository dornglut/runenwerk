// Owner: Engine Input Plugin - Tests
use crate::plugins::{
    ActionState, DigitalState, InputBindingChange, InputBindingChangeResult, InputContext,
    InputDeviceId, InputSourceId, InputState, KeyChord, KeyLocation, KeyboardInput, LogicalKey,
    NativeLogicalKey, ObservationOrigin, PhysicalKeyIdentity, PointerButton, PointerButtonInput,
    TouchInputPhase, action,
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
    actions.map_key(
        &input,
        action::WORLD_MOVE_LEFT,
        physical_key(KeyCode::KeyJ),
    );

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

    actions.map_key(
        &input,
        action::WORLD_MOVE_LEFT,
        physical_key(KeyCode::KeyA),
    );
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
fn device_scoped_key_state_keeps_product_press_edge_aggregate() {
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
    assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    input.handle_normalized_keyboard(context_a, &normalized_key(DigitalState::Released));
    actions.project(&input);
    assert!(actions.action_down(action::WORLD_MOVE_UP));

    input.handle_normalized_keyboard(context_b, &normalized_key(DigitalState::Released));
    actions.project(&input);
    assert!(!actions.action_down(action::WORLD_MOVE_UP));
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
    assert_eq!(state.neutral_active_touch_count(), 2);
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
    assert_eq!(state.neutral_active_touch_count(), 1);

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
