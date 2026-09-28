use std::fs;
use std::process::Command;

use engine::automation::{
    AutomationScenarioRelativeArtifactRef, AutomationScenarioStepV1, export_automation_scenario_v1,
};
use runenwerk_editor::automation::{
    EDITOR_AUTOMATION_SCENARIO_PRODUCT_ID, EDITOR_AUTOMATION_SCENARIO_PRODUCT_VERSION,
    EditorAutomationScenarioStepV1, EditorAutomationScenarioTargetV1,
    EditorAutomationViewportToolV1, validate_editor_automation_scenario_step_v1,
};

fn editor_binary() -> &'static str {
    env!("CARGO_BIN_EXE_runenwerk_editor")
}

#[test]
fn terminal_can_run_product_semantic_viewport_tool_scenario() {
    let output = Command::new(editor_binary())
        .args(["--automation-viewport-tool", "rotate"])
        .output()
        .expect("Editor automation process should launch");

    assert!(
        output.status.success(),
        "Editor automation process failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("automation completed: viewport_tool=rotate"),
        "unexpected Editor automation stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn invalid_terminal_automation_tool_fails_without_native_fallback() {
    let output = Command::new(editor_binary())
        .args(["--automation-viewport-tool", "teleport"])
        .output()
        .expect("Editor automation process should launch");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unsupported viewport tool 'teleport'"),
        "unexpected Editor automation stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn editor_scenario_steps() -> Vec<AutomationScenarioStepV1<EditorAutomationScenarioStepV1>> {
    vec![
        AutomationScenarioStepV1::Owner(EditorAutomationScenarioStepV1::ActivateViewportTool {
            target: EditorAutomationScenarioTargetV1::UniqueSceneViewport,
            tool: EditorAutomationViewportToolV1::Rotate,
        }),
        AutomationScenarioStepV1::Owner(EditorAutomationScenarioStepV1::AssertViewportTool {
            target: EditorAutomationScenarioTargetV1::UniqueSceneViewport,
            tool: EditorAutomationViewportToolV1::Rotate,
        }),
    ]
}

#[test]
fn terminal_can_run_persisted_editor_scenario() {
    let encoded = export_automation_scenario_v1(
        EDITOR_AUTOMATION_SCENARIO_PRODUCT_ID,
        EDITOR_AUTOMATION_SCENARIO_PRODUCT_VERSION,
        &editor_scenario_steps(),
        validate_editor_automation_scenario_step_v1,
    )
    .expect("Editor scenario should export");
    let directory = tempfile::tempdir().expect("temporary scenario directory should create");
    let path = directory.path().join("scenario.ron");
    fs::write(&path, encoded).expect("Editor scenario should write");

    let output = Command::new(editor_binary())
        .arg("--automation-scenario")
        .arg(&path)
        .output()
        .expect("Editor automation scenario process should launch");

    assert!(
        output.status.success(),
        "Editor automation scenario failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "automation scenario completed: product=runenwerk.editor.automation/v1 steps=2"
        ),
        "unexpected Editor scenario stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn editor_scenario_product_mismatch_fails_without_native_fallback() {
    let encoded = export_automation_scenario_v1(
        "runenwerk.other.automation",
        1,
        &editor_scenario_steps(),
        validate_editor_automation_scenario_step_v1,
    )
    .expect("mismatched scenario should still serialize");
    let directory = tempfile::tempdir().expect("temporary scenario directory should create");
    let path = directory.path().join("mismatch.ron");
    fs::write(&path, encoded).expect("mismatched scenario should write");

    let output = Command::new(editor_binary())
        .arg("--automation-scenario")
        .arg(&path)
        .output()
        .expect("Editor mismatched scenario process should launch");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("product contract mismatch"),
        "unexpected Editor mismatch stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn editor_scenario_normalized_replay_is_explicitly_unsupported() {
    let trace_ref =
        AutomationScenarioRelativeArtifactRef::new("trace.ron").expect("valid relative ref");
    let encoded = export_automation_scenario_v1::<EditorAutomationScenarioStepV1, _>(
        EDITOR_AUTOMATION_SCENARIO_PRODUCT_ID,
        EDITOR_AUTOMATION_SCENARIO_PRODUCT_VERSION,
        &[AutomationScenarioStepV1::ReplayNormalizedTrace(trace_ref)],
        validate_editor_automation_scenario_step_v1,
    )
    .expect("Editor replay scenario should serialize");
    let directory = tempfile::tempdir().expect("temporary scenario directory should create");
    let path = directory.path().join("unsupported.ron");
    fs::write(&path, encoded).expect("unsupported scenario should write");

    let output = Command::new(editor_binary())
        .arg("--automation-scenario")
        .arg(&path)
        .output()
        .expect("Editor unsupported scenario process should launch");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("does not support normalized trace replay"),
        "unexpected Editor unsupported stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
