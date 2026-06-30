use runenwerk_editor::editor_features::BaseControlsInteractionProofHost;
use ui_input::{
    FocusInputFact, FocusTargetId, Key, KeyState, KeyboardEvent, Modifiers, PointerButton,
    PointerDelta, PointerEvent, PointerEventKind, TextInputEvent, UiInputEvent,
};
use ui_math::UiPoint;
use ui_runtime::{InteractionVisibleState, WidgetId};

#[test]
fn base_controls_interaction_proof_host_processes_events() {
    let mut host = BaseControlsInteractionProofHost::new();

    host.apply_focus("focus_button", FocusInputFact::target(FocusTargetId(1)));
    host.apply_input_event(
        "move_button",
        pointer_event(PointerEventKind::Move, 12.0, 12.0),
    );
    host.apply_input_event(
        "press_button",
        pointer_event(PointerEventKind::Down, 12.0, 12.0),
    );
    host.apply_input_event(
        "release_button",
        pointer_event(PointerEventKind::Up, 12.0, 12.0),
    );
    host.apply_focus("focus_action", FocusInputFact::target(FocusTargetId(2)));
    host.apply_input_event("activate_action", key_event(Key::Enter));
    host.apply_focus("focus_inspector", FocusInputFact::target(FocusTargetId(3)));
    host.apply_input_event("text_probe", text_event("probe"));
    host.apply_focus("focus_list", FocusInputFact::target(FocusTargetId(4)));
    host.apply_input_event("list_down", key_event(Key::Down));
    host.apply_focus("focus_tree", FocusInputFact::target(FocusTargetId(5)));
    host.apply_input_event("tree_right", key_event(Key::Right));
    host.apply_focus("focus_table", FocusInputFact::target(FocusTargetId(6)));
    host.apply_input_event("table_down", key_event(Key::Down));
    host.apply_focus("focus_read_only", FocusInputFact::target(FocusTargetId(8)));
    host.apply_input_event("text_read_only_probe", text_event("probe"));
    host.apply_input_event(
        "disabled_button",
        pointer_event(PointerEventKind::Down, 12.0, 132.0),
    );
    host.apply_input_event(
        "outside",
        pointer_event(PointerEventKind::Down, 260.0, 260.0),
    );

    let proof = host.current_proof();
    assert_marker(&proof, WidgetId(1), InteractionVisibleState::Hovered);
    assert_marker(&proof, WidgetId(1), InteractionVisibleState::Pressed);
    assert_marker(&proof, WidgetId(1), InteractionVisibleState::Captured);
    assert_marker(
        &proof,
        WidgetId(1),
        InteractionVisibleState::ActivationRequested,
    );
    assert_marker(
        &proof,
        WidgetId(3),
        InteractionVisibleState::TextIntentProbe,
    );
    assert_marker(
        &proof,
        WidgetId(4),
        InteractionVisibleState::ListActiveItemIntent,
    );
    assert_marker(&proof, WidgetId(5), InteractionVisibleState::TreeNodeIntent);
    assert_marker(
        &proof,
        WidgetId(6),
        InteractionVisibleState::TableCellOrRowIntent,
    );
    assert_marker(
        &proof,
        WidgetId(8),
        InteractionVisibleState::ReadOnlyTextIntentProbe,
    );
    assert_marker(&proof, WidgetId(7), InteractionVisibleState::Suppressed);

    assert!(host.boundary_assertions().no_bypass_evidence());
    assert!(host.static_mount_report().passed());
    assert!(host.replay_live_parity_report().passed());
}

fn pointer_event(kind: PointerEventKind, x: f32, y: f32) -> UiInputEvent {
    UiInputEvent::Pointer(PointerEvent::new(
        kind,
        UiPoint::new(x, y),
        PointerDelta::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        1,
    ))
}

fn key_event(key: Key) -> UiInputEvent {
    UiInputEvent::Keyboard(KeyboardEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
    })
}

fn text_event(text: &str) -> UiInputEvent {
    UiInputEvent::Text(TextInputEvent {
        text: text.to_owned(),
    })
}

fn assert_marker(
    proof: &ui_runtime::InteractionVisualProof,
    widget_id: WidgetId,
    state: InteractionVisibleState,
) {
    let control = proof
        .main_view
        .control(widget_id)
        .expect("proof should contain control");
    assert!(
        control.has_marker(state),
        "{:?} missing {:?}",
        control.observed_markers,
        state
    );
}
