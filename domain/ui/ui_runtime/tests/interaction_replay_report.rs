use ui_controls::{
    BUTTON_CONTROL_KIND_ID, ControlInteractionDescriptor, ControlInteractionOutcome,
    ControlInteractionRequirement, ControlInteractionTrigger, ControlKindId,
    INSPECTOR_FIELD_CONTROL_KIND_ID, LIST_VIEW_CONTROL_KIND_ID, TABLE_VIEW_CONTROL_KIND_ID,
    TREE_VIEW_CONTROL_KIND_ID,
};
use ui_input::{
    FocusDirection, FocusInputFact, Key, KeyState, KeyboardInputFact, NormalizedInputFact,
    NormalizedInputSample, PointerButton, PointerEventKind, PointerInputFact, TextIntentFact,
};
use ui_math::{UiPoint, UiRect};
use ui_runtime::{
    InteractionReplayScript, InteractionReplayStep, MountedInteractionControl,
    MountedInteractionFixture, WidgetId, replay_interactions,
};

#[test]
fn mounted_interaction_replay_reports_facts_events_outcomes_and_boundaries() {
    let fixture = phase12_fixture();
    let script = InteractionReplayScript::new("phase12.replay")
        .with_step(pointer_step(
            "move_button",
            PointerEventKind::Move,
            12.0,
            12.0,
        ))
        .with_step(pointer_step(
            "press_button",
            PointerEventKind::Down,
            12.0,
            12.0,
        ))
        .with_step(pointer_step(
            "release_button",
            PointerEventKind::Up,
            12.0,
            12.0,
        ))
        .with_step(focus_next_step("focus_action"))
        .with_step(key_step("activate_action", Key::Enter))
        .with_step(focus_next_step("focus_inspector"))
        .with_step(text_intent_step("text_probe"))
        .with_step(focus_next_step("focus_list"))
        .with_step(key_step("list_down", Key::Down))
        .with_step(focus_next_step("focus_tree"))
        .with_step(key_step("tree_right", Key::Right))
        .with_step(focus_next_step("focus_table"))
        .with_step(key_step("table_down", Key::Down));

    let report = replay_interactions(&fixture, &script);

    assert!(report.boundary_assertions.no_bypass_evidence());
    assert!(
        report
            .runtime_facts
            .iter()
            .any(|fact| fact.fact == "hovered" && fact.target == WidgetId(1))
    );
    assert!(
        report
            .semantic_outcomes
            .iter()
            .any(|outcome| outcome.outcome == "activation-requested"
                && outcome.target == WidgetId(1))
    );
    assert!(
        report
            .semantic_outcomes
            .iter()
            .any(|outcome| outcome.outcome == "text-intent-seen")
    );
    assert!(
        report
            .semantic_outcomes
            .iter()
            .any(|outcome| outcome.outcome == "active-item-intent")
    );
    assert!(
        report
            .semantic_outcomes
            .iter()
            .any(|outcome| outcome.outcome == "node-intent")
    );
    assert!(
        report
            .semantic_outcomes
            .iter()
            .any(|outcome| outcome.outcome == "cell-or-row-intent")
    );
}

#[test]
fn replay_reports_disabled_no_target_and_keyboard_without_focus_cases() {
    let fixture = phase12_fixture();
    let script = InteractionReplayScript::new("phase12.negative")
        .with_step(key_step("keyboard_without_focus", Key::Enter))
        .with_step(pointer_step(
            "disabled_button",
            PointerEventKind::Down,
            12.0,
            132.0,
        ))
        .with_step(pointer_step(
            "outside",
            PointerEventKind::Down,
            260.0,
            260.0,
        ));

    let report = replay_interactions(&fixture, &script);

    assert!(
        report
            .no_target_events
            .iter()
            .any(|event| event.reason == "keyboard.no_focus")
    );
    assert!(
        report
            .suppressed_events
            .iter()
            .any(|event| event.reason == "control.disabled")
    );
    assert!(
        report
            .no_target_events
            .iter()
            .any(|event| event.reason == "pointer.no_target")
    );
    assert!(report.semantic_outcomes.is_empty());
    assert!(report.boundary_assertions.no_bypass_evidence());
}

#[test]
fn text_intent_against_non_text_probe_is_suppressed_without_text_editing() {
    let fixture = phase12_fixture();
    let script = InteractionReplayScript::new("phase12.text-negative")
        .with_step(focus_next_step("focus_action"))
        .with_step(text_intent_step("text_against_action"));

    let report = replay_interactions(&fixture, &script);

    assert!(
        report
            .suppressed_events
            .iter()
            .any(|event| event.reason == "text_intent.not_declared")
    );
    assert_eq!(report.boundary_assertions.text_edit_transactions, 0);
}

fn phase12_fixture() -> MountedInteractionFixture {
    MountedInteractionFixture::new("phase12.generic-interaction.fixture")
        .with_control(MountedInteractionControl::new(
            WidgetId(1),
            "Button",
            UiRect::new(0.0, 0.0, 80.0, 24.0),
            activatable(BUTTON_CONTROL_KIND_ID),
        ))
        .with_control(MountedInteractionControl::new(
            WidgetId(2),
            "Action",
            UiRect::new(0.0, 28.0, 80.0, 24.0),
            activatable("runenwerk.ui.control.action_prompt"),
        ))
        .with_control(MountedInteractionControl::new(
            WidgetId(3),
            "Inspector",
            UiRect::new(0.0, 56.0, 80.0, 24.0),
            text_probe(INSPECTOR_FIELD_CONTROL_KIND_ID),
        ))
        .with_control(MountedInteractionControl::new(
            WidgetId(4),
            "List",
            UiRect::new(0.0, 84.0, 80.0, 24.0),
            navigable(
                LIST_VIEW_CONTROL_KIND_ID,
                ControlInteractionOutcome::ActiveItemIntent,
            ),
        ))
        .with_control(MountedInteractionControl::new(
            WidgetId(5),
            "Tree",
            UiRect::new(84.0, 84.0, 80.0, 24.0),
            navigable(
                TREE_VIEW_CONTROL_KIND_ID,
                ControlInteractionOutcome::NodeIntent,
            ),
        ))
        .with_control(MountedInteractionControl::new(
            WidgetId(6),
            "Table",
            UiRect::new(168.0, 84.0, 80.0, 24.0),
            navigable(
                TABLE_VIEW_CONTROL_KIND_ID,
                ControlInteractionOutcome::CellOrRowIntent,
            ),
        ))
        .with_control(
            MountedInteractionControl::new(
                WidgetId(7),
                "Disabled",
                UiRect::new(0.0, 120.0, 80.0, 24.0),
                activatable(BUTTON_CONTROL_KIND_ID),
            )
            .disabled(),
        )
}

fn activatable(kind_id: &str) -> ControlInteractionDescriptor {
    ControlInteractionDescriptor::new(ControlKindId::new(kind_id))
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::PointerPress)
                .with_outcome(ControlInteractionOutcome::ActivationRequested),
        )
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardActivate)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::ActivationRequested),
        )
}

fn text_probe(kind_id: &str) -> ControlInteractionDescriptor {
    ControlInteractionDescriptor::new(ControlKindId::new(kind_id))
        .with_text_intent_probe(true)
        .with_requirement(
            ControlInteractionRequirement::new(ControlInteractionTrigger::TextIntent)
                .requiring_focus()
                .with_outcome(ControlInteractionOutcome::TextIntentSeen),
        )
}

fn navigable(kind_id: &str, outcome: ControlInteractionOutcome) -> ControlInteractionDescriptor {
    ControlInteractionDescriptor::new(ControlKindId::new(kind_id)).with_requirement(
        ControlInteractionRequirement::new(ControlInteractionTrigger::KeyboardNavigate)
            .requiring_focus()
            .with_outcome(outcome),
    )
}

fn pointer_step(id: &str, kind: PointerEventKind, x: f32, y: f32) -> InteractionReplayStep {
    InteractionReplayStep::new(
        id,
        NormalizedInputSample::new(id).with_fact(NormalizedInputFact::Pointer(
            PointerInputFact::new(kind, UiPoint::new(x, y)).with_button(PointerButton::Primary),
        )),
    )
}

fn focus_next_step(id: &str) -> InteractionReplayStep {
    InteractionReplayStep::new(
        id,
        NormalizedInputSample::new(id).with_fact(NormalizedInputFact::Focus(
            FocusInputFact::traversal(FocusDirection::Next),
        )),
    )
}

fn key_step(id: &str, key: Key) -> InteractionReplayStep {
    InteractionReplayStep::new(
        id,
        NormalizedInputSample::new(id).with_fact(NormalizedInputFact::Keyboard(
            KeyboardInputFact::new(key, KeyState::Pressed),
        )),
    )
}

fn text_intent_step(id: &str) -> InteractionReplayStep {
    InteractionReplayStep::new(
        id,
        NormalizedInputSample::new(id).with_fact(NormalizedInputFact::TextIntent(
            TextIntentFact::insert_text("probe"),
        )),
    )
}
