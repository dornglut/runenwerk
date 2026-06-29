//! Canonical Phase 12 generic interaction proof fixtures.
//!
//! These helpers build descriptor-backed replay fixtures and scripts for the
//! reusable interaction proof. They do not create product UI, execute
//! app/editor/game commands, mutate product state, open overlays, or perform
//! full text editing.

use ui_controls::{
    ACTION_PROMPT_CONTROL_KIND_ID, BUTTON_CONTROL_KIND_ID, CompiledControlPackage,
    INSPECTOR_FIELD_CONTROL_KIND_ID, LABEL_CONTROL_KIND_ID, LIST_VIEW_CONTROL_KIND_ID,
    TABLE_VIEW_CONTROL_KIND_ID, TREE_VIEW_CONTROL_KIND_ID,
};
use ui_input::{
    FocusDirection, FocusInputFact, FocusTargetId, Key, KeyState, KeyboardInputFact,
    NormalizedInputFact, NormalizedInputSample, PointerButton, PointerEventKind, PointerInputFact,
    TextIntentFact,
};
use ui_math::{UiPoint, UiRect};

use crate::{
    InteractionProofFrame, InteractionReplayScript, InteractionReplayStep,
    InteractionStoryExecutionMode, InteractionStorySession, InteractionVisualProof,
    MountedInteractionFixture, MountedInteractionPlacement, WidgetId, replay_interactions,
};

/// Stable proof id for the Phase 12 generic interaction visual proof.
pub const PHASE12_GENERIC_INTERACTION_PROOF_ID: &str = "phase12.generic_interaction";

/// Stable executable story id for the Phase 12A generic interaction proof-host core.
pub const PHASE12_EXECUTABLE_GENERIC_INTERACTION_STORY_ID: &str =
    "phase12a.executable_generic_interaction";

/// Builds the canonical Phase 12 mounted interaction fixture.
///
/// The fixture uses package-backed base-control interaction descriptors from
/// the compiled package. It does not recreate descriptor data locally and will
/// fail if a base control no longer lowers its package interaction descriptor.
pub fn phase12_generic_interaction_fixture(
    compiled: &CompiledControlPackage,
) -> MountedInteractionFixture {
    MountedInteractionFixture::from_compiled_controls(
        "phase12.generic-interaction.fixture",
        compiled,
        [
            MountedInteractionPlacement::new(
                WidgetId(1),
                BUTTON_CONTROL_KIND_ID,
                "Button",
                UiRect::new(0.0, 0.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(2),
                ACTION_PROMPT_CONTROL_KIND_ID,
                "Action",
                UiRect::new(0.0, 28.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(3),
                INSPECTOR_FIELD_CONTROL_KIND_ID,
                "Inspector",
                UiRect::new(0.0, 56.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(4),
                LIST_VIEW_CONTROL_KIND_ID,
                "List",
                UiRect::new(0.0, 84.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(5),
                TREE_VIEW_CONTROL_KIND_ID,
                "Tree",
                UiRect::new(84.0, 84.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(6),
                TABLE_VIEW_CONTROL_KIND_ID,
                "Table",
                UiRect::new(168.0, 84.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(7),
                BUTTON_CONTROL_KIND_ID,
                "Disabled",
                UiRect::new(0.0, 120.0, 80.0, 24.0),
            )
            .disabled(),
            MountedInteractionPlacement::new(
                WidgetId(8),
                INSPECTOR_FIELD_CONTROL_KIND_ID,
                "Read-only Inspector",
                UiRect::new(84.0, 120.0, 120.0, 24.0),
            )
            .read_only(),
            MountedInteractionPlacement::new(
                WidgetId(9),
                LABEL_CONTROL_KIND_ID,
                "Label",
                UiRect::new(0.0, 148.0, 80.0, 24.0),
            ),
            MountedInteractionPlacement::new(
                WidgetId(10),
                BUTTON_CONTROL_KIND_ID,
                "Inert Button",
                UiRect::new(84.0, 148.0, 120.0, 24.0),
            )
            .inert(),
        ],
    )
}

/// Builds the canonical positive replay script for Phase 12 generic interaction.
///
/// The script exercises hover, press/release activation, focus traversal,
/// keyboard intent, text-intent probe, list/tree/table navigation intent,
/// disabled suppression, and no-target evidence.
pub fn phase12_generic_interaction_positive_script() -> InteractionReplayScript {
    InteractionReplayScript::new("phase12.replay")
        .with_step(focus_target_step("focus_button", WidgetId(1)))
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
        .with_step(key_step("table_down", Key::Down))
        .with_step(focus_target_step("focus_read_only_inspector", WidgetId(8)))
        .with_step(text_intent_step("text_read_only_probe"))
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
        ))
}

/// Builds canonical negative replay scripts for suppression/no-target cases.
///
/// These scripts prove disabled, no-target, focus-validation, release-outside,
/// and text-intent boundary behavior without product mutation or text editing.
pub fn phase12_generic_interaction_negative_scripts() -> Vec<InteractionReplayScript> {
    vec![
        InteractionReplayScript::new("phase12.release-outside")
            .with_step(pointer_step(
                "press_button",
                PointerEventKind::Down,
                12.0,
                12.0,
            ))
            .with_step(pointer_step(
                "leave_button",
                PointerEventKind::Leave,
                120.0,
                12.0,
            ))
            .with_step(pointer_step(
                "release_outside",
                PointerEventKind::Up,
                260.0,
                260.0,
            )),
        InteractionReplayScript::new("phase12.focus-validation")
            .with_step(focus_target_step("focus_button", WidgetId(1)))
            .with_step(focus_target_step("focus_missing", WidgetId(404)))
            .with_step(focus_target_step("focus_disabled", WidgetId(7)))
            .with_step(focus_target_step("focus_label_without_focus", WidgetId(9)))
            .with_step(focus_target_step("focus_inert", WidgetId(10)))
            .with_step(focus_next_step("focus_traversal")),
        InteractionReplayScript::new("phase12.negative")
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
            )),
        InteractionReplayScript::new("phase12.text-negative")
            .with_step(focus_target_step("focus_action", WidgetId(2)))
            .with_step(text_intent_step("text_against_action")),
        InteractionReplayScript::new("phase12.read-only-text")
            .with_step(focus_target_step("focus_read_only_inspector", WidgetId(8)))
            .with_step(text_intent_step("text_read_only_probe")),
    ]
}

/// Builds the complete Phase 12 visual proof frame from compiled base controls.
///
/// This helper runs the canonical positive replay against package-backed base
/// control descriptors and returns semantic visible proof data. Static/generic
/// gallery adapters should project this frame rather than rebuilding fixtures.
pub fn phase12_generic_interaction_proof_frame(
    compiled: &CompiledControlPackage,
) -> InteractionProofFrame {
    let fixture = phase12_generic_interaction_fixture(compiled);
    let script = phase12_generic_interaction_positive_script();
    let report = replay_interactions(&fixture, &script);
    let proof = InteractionVisualProof::from_fixture_report(
        PHASE12_GENERIC_INTERACTION_PROOF_ID,
        &fixture,
        &report,
        WidgetId(1),
    );

    InteractionProofFrame::new(proof)
}

/// Builds the canonical Phase 12A executable interaction story session.
///
/// The returned session is empty. Callers can drive it with the canonical replay
/// script or live-shaped normalized input samples. Both paths reuse the same
/// descriptor-backed replay formation path.
pub fn phase12_executable_generic_interaction_story_session(
    compiled: &CompiledControlPackage,
    mode: InteractionStoryExecutionMode,
) -> InteractionStorySession {
    InteractionStorySession::new(
        PHASE12_EXECUTABLE_GENERIC_INTERACTION_STORY_ID,
        phase12_generic_interaction_fixture(compiled),
        mode,
        WidgetId(1),
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

fn focus_target_step(id: &str, widget_id: WidgetId) -> InteractionReplayStep {
    InteractionReplayStep::new(
        id,
        NormalizedInputSample::new(id).with_fact(NormalizedInputFact::Focus(
            FocusInputFact::target(FocusTargetId(widget_id.0)),
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
