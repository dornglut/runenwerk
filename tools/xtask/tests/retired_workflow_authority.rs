use std::{fs, path::Path};

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask must live at tools/xtask")
}

#[test]
fn retired_track_execution_authority_cannot_return() {
    let root = repository_root();

    for retired in [
        "docs-site/src/content/docs/workspace/track-execution-manifest.md",
        "docs-site/src/content/docs/workspace/track-execution-locks",
    ] {
        assert!(
            !root.join(retired).exists(),
            "retired workflow authority must not return: {retired}"
        );
    }
}

#[test]
fn context_profiles_do_not_request_retired_planning_authority() {
    let root = repository_root();

    for profile in [
        "tools/context/profiles/ai-core.toml",
        "tools/context/profiles/current-work.toml",
        "tools/context/profiles/domain-work.toml",
        "tools/context/profiles/implementation-work.toml",
    ] {
        let text = fs::read_to_string(root.join(profile))
            .unwrap_or_else(|error| panic!("failed to read {profile}: {error}"));
        assert!(
            !text.contains("production-tracks.md"),
            "current context profile must not request retired production-track authority: {profile}"
        );
    }

    let workspace_planning = "tools/context/profiles/workspace-planning.toml";
    let text = fs::read_to_string(root.join(workspace_planning))
        .unwrap_or_else(|error| panic!("failed to read {workspace_planning}: {error}"));
    assert!(
        !text.contains("workspace/routines/"),
        "current workspace-planning context must not request retired routine authority"
    );
}
