use ui_controls::BaseControlsPlugin;
use ui_runtime::{
    InteractionStoryExecutionMode, base_controls_executable_interaction_story_session,
    base_controls_generic_interaction_positive_script, interaction_visual_proof_to_frame,
    phase12_executable_generic_interaction_expected_evidence,
};
use ui_static_mount::UiStaticMountReport;

#[test]
fn phase12_executable_interaction_story_frame_passes_static_mount() {
    let compiled = BaseControlsPlugin::new().compile();
    let script = base_controls_generic_interaction_positive_script();
    let expected = phase12_executable_generic_interaction_expected_evidence();
    let mut session = base_controls_executable_interaction_story_session(
        &compiled,
        InteractionStoryExecutionMode::Replay,
    );

    let run_report = session.run_script_with_expected(&script, &expected);
    assert!(run_report.evidence_result.passed());

    let rendered = interaction_visual_proof_to_frame(&run_report.visual_proof);
    let mount_report = UiStaticMountReport::from_frame(rendered.frame.clone());

    assert!(mount_report.passed(), "{:?}", mount_report.diagnostics());
    assert!(rendered.summary.has_main_inspector_and_report);
    assert!(rendered.summary.marker_count >= expected.required_markers.len());
    assert!(run_report.boundary_assertions.no_bypass_evidence());
}
