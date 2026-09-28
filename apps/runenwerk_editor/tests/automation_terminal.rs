use std::process::Command;

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
