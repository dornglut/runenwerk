use ui_controls::BaseControlsPlugin;
use ui_runtime::{
    InteractionStoryExecutionMode, InteractionVisibleState, WidgetId,
    phase12_executable_generic_interaction_expected_evidence,
    phase12_executable_generic_interaction_story_session,
    phase12_generic_interaction_positive_script,
};

#[test]
fn phase12_executable_interaction_story_replay_produces_expected_evidence() {
    let compiled = BaseControlsPlugin::new().compile();
    let script = phase12_generic_interaction_positive_script();
    let expected = phase12_executable_generic_interaction_expected_evidence();
    let mut session = phase12_executable_generic_interaction_story_session(
        &compiled,
        InteractionStoryExecutionMode::Replay,
    );

    let report = session.run_script_with_expected(&script, &expected);

    assert!(
        report.evidence_result.passed(),
        "missing evidence: {:?}",
        report.evidence_result.missing
    );
    assert_eq!(report.input_log.len(), script.steps.len());
    assert!(report.boundary_assertions.no_bypass_evidence());
    assert!(report.render_summary.has_main_inspector_and_report);

    let button = report
        .visual_proof
        .main_view
        .control(WidgetId(1))
        .expect("button should exist in proof");
    assert!(button.has_marker(InteractionVisibleState::Pressed));
    assert!(button.has_marker(InteractionVisibleState::Captured));
    assert!(button.has_marker(InteractionVisibleState::ActivationRequested));
    assert!(
        !button.has_current_state(InteractionVisibleState::Pressed),
        "pressed must be observed evidence, not final current state"
    );
}

#[test]
fn phase12_executable_interaction_story_live_log_replays_with_semantic_parity() {
    let compiled = BaseControlsPlugin::new().compile();
    let script = phase12_generic_interaction_positive_script();
    let expected = phase12_executable_generic_interaction_expected_evidence();
    let mut live = phase12_executable_generic_interaction_story_session(
        &compiled,
        InteractionStoryExecutionMode::Live,
    );

    for step in &script.steps {
        live.apply_step(step.clone());
    }

    let live_report = live.run_report_with_expected(&expected);
    assert!(
        live_report.evidence_result.passed(),
        "missing live evidence: {:?}",
        live_report.evidence_result.missing
    );

    let parity = live.replay_live_parity_report();
    assert!(parity.passed(), "parity failed: {parity:#?}");
    assert_eq!(
        parity.live_report.input_log,
        parity.replayed_live_log_report.input_log
    );
    assert!(parity.equivalent_target_resolution);
    assert!(parity.equivalent_focus_resolution);
    assert!(parity.equivalent_state_transitions);
    assert!(parity.equivalent_runtime_facts);
    assert!(parity.equivalent_runtime_events);
    assert!(parity.equivalent_semantic_outcomes);
    assert!(parity.equivalent_suppression);
    assert!(parity.equivalent_no_target);
    assert!(parity.equivalent_observed_markers);
    assert!(parity.equivalent_final_current_states);
    assert!(parity.equivalent_boundaries);
}
