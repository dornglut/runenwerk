use anyhow::Result;
use std::{fs, process::Command};

#[test]
fn creates_a_ron_only_project_and_reports_invalid_manifest() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let executable = env!("CARGO_BIN_EXE_rune");

    let created = Command::new(executable)
        .current_dir(temp.path())
        .args(["new", "hello-rune"])
        .output()?;
    assert!(
        created.status.success(),
        "rune new failed: {}",
        String::from_utf8_lossy(&created.stderr)
    );

    let project = temp.path().join("hello-rune");
    assert!(project.join("app.ron").is_file());
    assert!(project.join("ui/main.ron").is_file());
    assert!(!project.join("Cargo.toml").exists());

    // A malformed manifest must fail in the child before a native window is started.
    fs::write(project.join("app.ron"), "(invalid")?;

    let run = Command::new(executable)
        .current_dir(&project)
        .arg("run")
        .output()?;
    assert!(!run.status.success(), "rune run accepted malformed RON");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("invalid"),
        "missing useful error: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    Ok(())
}
