use ui_runtime::{
    TextEditingLifecycleState, base_controls_text_editing_fixture,
    base_controls_text_editing_positive_script, base_controls_text_editing_suppression_script,
    replay_text_editing,
};

#[test]
fn text_editing_replay_reports_caret_selection_composition_and_boundaries() {
    let fixture = base_controls_text_editing_fixture();
    let script = base_controls_text_editing_positive_script();
    let report = replay_text_editing(&fixture, &script);

    assert_eq!(report.descriptor_evidence.len(), 1);
    assert!(report.input_steps.len() >= 12);
    assert!(
        report
            .accepted_edit_intents
            .iter()
            .any(|intent| intent.intent == "insert-text")
    );
    assert!(
        report
            .accepted_edit_intents
            .iter()
            .any(|intent| intent.intent == "replace-selection")
    );
    assert!(
        report
            .suppressed_edit_intents
            .iter()
            .any(|intent| intent.intent == "paste"
                && intent.reason == "host_owned_source.unsupported_by_descriptor")
    );
    assert!(!report.caret_evidence.is_empty());
    assert!(
        report
            .selection_evidence
            .iter()
            .any(|selection| !selection.collapsed && selection.accepted)
    );
    assert!(
        report
            .composition_evidence
            .iter()
            .any(|composition| composition.kind == "composition-start")
    );
    assert!(report.lifecycle_transitions.iter().any(|transition| {
        transition.to == TextEditingLifecycleState::Composing
            && transition.reason == "composition-start"
    }));
    assert!(report.boundary_assertions.no_bypass_evidence());
    assert_eq!(report.boundary_assertions.host_commands_executed, 0);
    assert_eq!(report.boundary_assertions.product_mutations, 0);
    assert_eq!(report.boundary_assertions.authored_ui_edits, 0);
    assert_eq!(report.boundary_assertions.product_undo_redo_operations, 0);
    assert_eq!(report.boundary_assertions.plugin_framework_operations, 0);
}

#[test]
fn text_editing_replay_suppresses_no_target_without_bypass() {
    let fixture = base_controls_text_editing_fixture();
    let script = base_controls_text_editing_suppression_script();
    let report = replay_text_editing(&fixture, &script);

    assert!(report.accepted_edit_intents.is_empty());
    assert_eq!(report.suppressed_edit_intents.len(), 1);
    assert_eq!(report.suppressed_edit_intents[0].reason, "target.missing");
    assert!(report.boundary_assertions.no_bypass_evidence());
}
