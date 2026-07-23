use std::fs;
use std::path::{Path, PathBuf};

use editor_shell::{ShellCommand, projected_host_tab_stacks};
use runenwerk_editor::editor_app::RunenwerkEditorApp;
use runenwerk_editor::shell::dispatch_shell_command;
use runenwerk_editor::shell::{EditorWindowPresentationBinding, RunenwerkEditorShellState};

#[test]
fn composition_is_the_only_live_editor_structural_authority() {
    let state = include_str!("../src/shell/state.rs");
    let state_fields = source_between(
        state,
        "pub struct RunenwerkEditorShellState {",
        "impl Default for RunenwerkEditorShellState",
    );

    assert!(state_fields.contains("composition_runtime: EditorCompositionRuntime"));
    assert!(state_fields.contains("composition_projection: EditorCompositionProjectionArtifact"));
    assert!(!state_fields.contains("workspace_state: WorkspaceState"));
    assert!(!state_fields.contains("legacy_workspace_oracle"));

    for forbidden_item in [
        "pub fn workspace_state",
        "pub fn replace_workspace_state",
        "pub fn apply_workspace_mutation",
        "pub fn apply_workspace_mutation_with_allocations",
        "pub fn try_apply_workspace_mutation_with_allocations",
    ] {
        assert!(
            !state.contains(forbidden_item),
            "legacy structural authority returned through {forbidden_item}",
        );
    }

    let controller = include_str!("../src/shell/controller.rs");
    assert!(!controller.contains("WorkspaceMutation"));
    assert!(!controller.contains("reduce_workspace"));
    assert!(!controller.contains("replace_workspace_state"));
}

#[test]
fn composition_projection_contract_is_owned_by_structural_composition() {
    let structural = include_str!(
        "../../../domain/editor/editor_shell/src/composition/structural/projection.rs"
    );
    let legacy = include_str!("../../../domain/editor/editor_shell/src/workspace/projection.rs");

    for owned_contract in [
        "pub struct ProjectedPanelSlot",
        "pub struct ProjectedTabStackSlot",
        "pub enum ProjectedWorkspaceHostSlot",
        "pub struct StructuralWidgetRoutingContext",
        "pub struct WorkspaceProjectionArtifact",
        "fn assemble_editor_shell_projection",
    ] {
        assert!(structural.contains(owned_contract));
        assert!(!legacy.contains(owned_contract));
    }
    assert!(!structural.contains("WorkspaceState"));
    assert!(legacy.contains("project_workspace_for_shell"));
    assert!(legacy.contains("assemble_editor_shell_projection"));
}

#[test]
fn composition_provider_session_and_viewport_identity_is_mounted_unit_keyed() {
    let provider_projection =
        include_str!("../src/shell/composition_runtime/provider_projection.rs");
    let sessions = include_str!("../src/shell/surface_session.rs");
    let viewports = include_str!("../src/runtime/viewport/instance_registry.rs");

    assert!(provider_projection.contains("mounted_unit_id: mounted_unit.id"));
    assert!(provider_projection.contains("sort_by_key(|request| request.mounted_unit_id)"));
    assert!(sessions.contains("BTreeMap<MountedUnitId, SurfaceSessionState>"));
    assert!(!sessions.contains("BTreeMap<ToolSurfaceInstanceId, SurfaceSessionState>"));
    assert!(viewports.contains("records_by_mounted_unit: BTreeMap<MountedUnitId"));
    assert!(viewports.contains("mounted_unit_by_tool_surface"));
}

#[test]
fn composition_unavailable_content_fallback_order_requires_explicit_hide_authority() {
    use ui_composition::{
        ContentLiveness, ContentProjectionFallback, UnavailableContentPolicy,
        select_content_projection_fallback,
    };

    let select = |app_projection, neutral, policy, host_hide| {
        select_content_projection_fallback(
            ContentLiveness::Missing,
            app_projection,
            neutral,
            policy,
            host_hide,
        )
    };
    assert_eq!(
        select(true, true, UnavailableContentPolicy::AllowHide, true),
        Some(ContentProjectionFallback::AppProvidedUnavailable)
    );
    assert_eq!(
        select(false, true, UnavailableContentPolicy::AllowHide, true),
        Some(ContentProjectionFallback::NeutralDiagnosticPlaceholder)
    );
    assert_eq!(
        select(false, false, UnavailableContentPolicy::AllowHide, true),
        Some(ContentProjectionFallback::Hidden)
    );
    assert_eq!(
        select(false, false, UnavailableContentPolicy::ShowFallback, true),
        None
    );
    assert_eq!(
        select(false, false, UnavailableContentPolicy::AllowHide, false),
        None
    );
}

#[test]
fn composition_persistence_has_no_active_legacy_writer_or_reverse_loader() {
    let persistence = include_str!("../src/persistence/workspace_layout.rs");
    let normalized = persistence.replace("\r\n", "\n");
    let active = normalized
        .split("#[cfg(test)]\npub fn write_workspace_layout")
        .next()
        .expect("active persistence source should precede test adapters");

    for forbidden in [
        "PersistedWorkspaceStateV1",
        "PersistedWorkspaceStateV2",
        "PersistedWorkspaceStateV3",
        "PersistedWorkspaceStateV4",
        "PersistedWorkspaceStateV5",
        "pub fn write_workspace_layout",
        "from_composition_to_workspace",
    ] {
        assert!(
            !active.contains(forbidden),
            "active persistence retained {forbidden}"
        );
    }
    assert!(active.contains("CompositionBundleRepository"));
    assert!(active.contains("save_editor_composition_layout"));
    assert!(active.contains("load_editor_composition_layout"));
    assert!(active.contains("probe_legacy_layout_path"));
}

#[test]
fn composition_targets_bind_to_app_owned_presentation_without_core_native_handles() {
    let shell = RunenwerkEditorShellState::new();
    let targets = shell
        .composition_runtime()
        .composition()
        .definition()
        .targets();

    assert_eq!(targets.len(), 1);
    assert_eq!(
        shell.composition_target_binding(targets[0].id),
        Some(EditorWindowPresentationBinding::primary())
    );

    let core_source_root = workspace_root().join("domain/ui/ui_composition/src");
    let mut sources = Vec::new();
    collect_rust_sources(&core_source_root, &mut sources);
    let joined = sources
        .iter()
        .map(|path| fs::read_to_string(path).expect("composition source should be readable"))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "NativeWindowId",
        "RenderSurfaceId",
        "UiProgram",
        "ui_surface",
    ] {
        assert!(
            !joined.contains(forbidden),
            "core composition leaked {forbidden}"
        );
    }
}

#[test]
fn structural_commands_commit_through_ui_composition_transactions() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell = RunenwerkEditorShellState::new();
    let revision_before = shell.composition_runtime().composition().revision();
    let stack = projected_host_tab_stacks(&shell.composition_projection().shell.root_host)
        .into_iter()
        .find(|stack| stack.active_panel.is_some())
        .expect("default composition should contain an active stack");
    let active_panel = stack.active_panel.as_ref().unwrap();
    let command = ShellCommand::SetTabStackActivePanel {
        tab_stack_id: stack.tab_stack_id,
        panel_instance_id: active_panel.panel_instance_id,
        projection_epoch: shell.current_projection_epoch(),
    };

    dispatch_shell_command(&mut app, Some(&mut shell), command, None, None, None, None)
        .expect("structural command should commit through the composition gateway");

    assert_eq!(
        shell.composition_runtime().composition().revision().raw(),
        revision_before.raw() + 1
    );
    assert!(!app.console_lines().iter().any(|line| {
        line.text
            .contains("editor_composition.static.mutation_deferred")
    }));
}

#[test]
fn composition_history_restores_paired_core_and_editor_extension_state() {
    let mut app = RunenwerkEditorApp::new();
    let mut shell = RunenwerkEditorShellState::new();
    let stack = projected_host_tab_stacks(&shell.composition_projection().shell.root_host)
        .into_iter()
        .find(|stack| {
            stack
                .active_panel
                .as_ref()
                .and_then(|panel| panel.active_stable_surface_key.as_ref())
                .is_some()
        })
        .expect("default composition should contain a lockable stack");
    let tab_stack_id = stack.tab_stack_id;
    let stable_key = stack
        .active_panel
        .as_ref()
        .and_then(|panel| panel.active_stable_surface_key.clone())
        .unwrap();
    let region_id = shell
        .region_id_for_tab_stack(tab_stack_id)
        .expect("projected tab stack should map to a composition region");

    dispatch_shell_command(
        &mut app,
        Some(&mut shell),
        ShellCommand::LockTabStackAreaStableKey {
            tab_stack_id,
            locked_stable_surface_key: Some(stable_key.clone()),
            projection_epoch: 0,
        },
        None,
        None,
        None,
        None,
    )
    .expect("lock edit should commit");
    assert_eq!(
        shell
            .composition_runtime()
            .extension()
            .region(region_id)
            .unwrap()
            .locked_content_key
            .as_deref(),
        Some(stable_key.as_str())
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell),
        ShellCommand::UndoCompositionLayout,
        None,
        None,
        None,
        None,
    )
    .expect("composition undo should revalidate and commit");
    assert_eq!(
        shell
            .composition_runtime()
            .extension()
            .region(region_id)
            .unwrap()
            .locked_content_key,
        None
    );

    dispatch_shell_command(
        &mut app,
        Some(&mut shell),
        ShellCommand::RedoCompositionLayout,
        None,
        None,
        None,
        None,
    )
    .expect("composition redo should revalidate and commit");
    assert_eq!(
        shell
            .composition_runtime()
            .extension()
            .region(region_id)
            .unwrap()
            .locked_content_key
            .as_deref(),
        Some(stable_key.as_str())
    );
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source
        .find(start)
        .expect("source start marker should exist");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("source end marker should exist");
    &source[start..end]
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runenwerk_editor should live under apps/")
        .to_path_buf()
}

fn collect_rust_sources(root: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("source directory should be readable") {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
    output.sort();
}
