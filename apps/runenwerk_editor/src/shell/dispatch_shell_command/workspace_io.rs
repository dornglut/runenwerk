use std::path::PathBuf;

use editor_core::EditorMutationError;

use crate::editor_app::RunenwerkEditorApp;
use crate::persistence::{
    default_composition_layout_root_for_profile, load_editor_composition_layout,
    probe_legacy_layout_path, read_retained_change_log, retained_change_log_path_for_scene,
    save_editor_composition_layout, write_retained_change_log,
};
use crate::shell::RunenwerkEditorShellState;

const DEFAULT_EDITOR_SCENE_PATH: &str = "editor-scenes/default.scene.ron";

fn default_scene_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_EDITOR_SCENE_PATH)
}

pub(super) fn adjacent_workspace_profile(
    registry: &editor_shell::WorkspaceProfileRegistry,
    active_profile_id: editor_shell::WorkspaceProfileId,
    delta: isize,
) -> Option<editor_shell::WorkspaceProfileId> {
    let profiles = registry
        .profiles()
        .map(|profile| profile.id)
        .collect::<Vec<_>>();
    if profiles.is_empty() {
        return None;
    }
    let active_index = profiles
        .iter()
        .position(|profile_id| *profile_id == active_profile_id)?;
    let next_index = (active_index as isize + delta).rem_euclid(profiles.len() as isize) as usize;
    profiles.get(next_index).copied()
}

pub(super) fn switch_workspace_profile(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    profile_id: editor_shell::WorkspaceProfileId,
) -> Result<(), EditorMutationError> {
    load_workspace_profile_layout(app, shell_state, profile_id)
}

pub(super) fn close_workspace_profile(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    profile_id: editor_shell::WorkspaceProfileId,
) -> Result<(), EditorMutationError> {
    let was_active = shell_state.active_workspace_profile_id() == profile_id;
    let Some(next_active_profile_id) = shell_state.close_workspace_profile_id(profile_id) else {
        app.append_console_line("[workspace] close ignored; at least one workspace remains open");
        return Ok(());
    };

    if was_active {
        load_workspace_profile_layout(app, shell_state, next_active_profile_id)?;
    }
    shell_state.close_toolbar_menu();
    app.append_console_line(format!("[workspace] closed profile {}", profile_id.raw()));
    Ok(())
}

pub(super) fn load_workspace_profile_layout(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    profile_id: editor_shell::WorkspaceProfileId,
) -> Result<(), EditorMutationError> {
    let profile = app
        .workbench_host()
        .workspace_profile(profile_id)
        .cloned()
        .ok_or(EditorMutationError::runtime_rejected(
            "workspace profile missing",
        ))?;
    let composition_root = default_composition_layout_root_for_profile(profile_id);
    if composition_root.join("active-generation.ron").exists() {
        let runtime = load_editor_composition_layout(&composition_root).map_err(|error| {
            app.append_console_line(format!(
                "[composition_persistence.load_failed] {}: {}",
                composition_root.display(),
                error_chain_summary(&error)
            ));
            EditorMutationError::runtime_rejected("failed to load composition layout")
        })?;
        if runtime.extension().workspace_profile_raw() != profile_id.raw() {
            return Err(EditorMutationError::runtime_rejected(
                "composition profile identity mismatch",
            ));
        }
        shell_state
            .queue_composition_restore(runtime)
            .map_err(|_| {
                EditorMutationError::runtime_rejected("composition restore queue failed")
            })?;
        app.append_console_line(format!(
            "[composition] queued persisted {} layout restore",
            profile.label
        ));
        return Ok(());
    }

    shell_state
        .activate_workspace_profile_ref_with_registry(
            &profile.profile_ref,
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().tool_surface_registry(),
        )
        .map_err(|_| EditorMutationError::runtime_rejected("composition profile import failed"))?;
    app.prune_surface_sessions_for_composition(shell_state.composition_runtime());
    app.append_console_line(format!("[composition] loaded {} layout", profile.label));
    Ok(())
}

fn error_chain_summary(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(|cause| cause.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}

pub(super) fn save_workspace_layout_for_active_profile(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    ensure_composition_save_allowed(app, shell_state)?;
    let composition_root =
        default_composition_layout_root_for_profile(shell_state.active_workspace_profile_id());
    save_editor_composition_layout(&composition_root, shell_state.composition_runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save composition layout"))?;
    app.append_console_line(format!(
        "[composition] saved layout {}",
        composition_root.display()
    ));
    Ok(())
}

pub(super) fn save_scene_to_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    ensure_composition_save_allowed(app, shell_state)?;
    let path = default_scene_file_path();
    app.save_scene_persistence_to_path(&path)?;
    let retained_path = retained_change_log_path_for_scene(&path);
    let composition_root =
        default_composition_layout_root_for_profile(shell_state.active_workspace_profile_id());
    let entry_count = write_retained_change_log(&retained_path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save retained change log"))?;
    save_editor_composition_layout(&composition_root, shell_state.composition_runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save composition layout"))?;
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::SceneSaved {
            path: path.display().to_string(),
        });
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::RetainedChangesSaved {
            path: retained_path.display().to_string(),
            entry_count,
        });
    app.append_console_line(format!("[io] saved {}", path.display()));
    app.append_console_line(format!(
        "[io] retained {} ratified changes at {}",
        entry_count,
        retained_path.display()
    ));
    app.append_console_line(format!(
        "[io] saved composition layout {}",
        composition_root.display()
    ));
    Ok(())
}

fn ensure_composition_save_allowed(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    if !shell_state.composition_coordination_pending() {
        return Ok(());
    }
    app.append_console_line(format!(
        "[{}] Wait for the pending composition transition to commit or roll back before saving.",
        editor_shell::EditorCompositionDiagnosticCode::CoordinationPending.as_str()
    ));
    Err(EditorMutationError::runtime_rejected(
        "composition save blocked by pending coordination",
    ))
}

pub(super) fn load_scene_from_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    let path = default_scene_file_path();
    if !path.exists() {
        app.append_console_line(format!(
            "[io] scene file missing, skipping load: {}",
            path.display()
        ));
        return Ok(());
    }

    let migration = match app.load_scene_persistence_from_path(&path) {
        Ok(migration) => migration,
        Err(class) => {
            app.append_console_line(format!(
                "[io] load failed ({})",
                migration_failure_class_label(class)
            ));
            return Err(EditorMutationError::runtime_rejected(
                "failed to load editor scene",
            ));
        }
    };
    let retained_path = retained_change_log_path_for_scene(&path);
    let composition_root =
        default_composition_layout_root_for_profile(shell_state.active_workspace_profile_id());
    let legacy_workspace_layout_path = path.with_extension("workspace.ron");
    let retained = if retained_path.exists() {
        Some(read_retained_change_log(&retained_path).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to load retained change log")
        })?)
    } else {
        None
    };
    app.reset_transient_editor_ui_state();
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::SceneLoaded {
            path: path.display().to_string(),
            migration_path: migration,
        });
    if let Some(migration_path) = migration {
        app.append_console_line(format!(
            "[io] scene migration applied: {}",
            editor_core::migration_path_label(migration_path)
        ));
    }
    if let Some(retained) = retained {
        app.runtime_mut().record_workflow_event(
            editor_core::WorkflowEventKind::RetainedChangesLoaded {
                path: retained_path.display().to_string(),
                entry_count: retained.entries.len(),
            },
        );
        app.append_console_line(format!(
            "[io] loaded retained change log: {} entries ({})",
            retained.entries.len(),
            retained_path.display()
        ));
    }
    if composition_root.join("active-generation.ron").exists() {
        match load_editor_composition_layout(&composition_root) {
            Ok(runtime) => {
                if runtime.extension().workspace_profile_raw()
                    != shell_state.active_workspace_profile_id().raw()
                {
                    return Err(EditorMutationError::runtime_rejected(
                        "composition profile identity mismatch",
                    ));
                }
                shell_state
                    .queue_composition_restore(runtime)
                    .map_err(|_| {
                        EditorMutationError::runtime_rejected("composition restore queue failed")
                    })?;
                app.append_console_line(format!(
                    "[io] queued composition layout restore {}",
                    composition_root.display()
                ));
            }
            Err(error) => app.append_console_line(format!(
                "[io] composition layout load failed, keeping current layout: {} ({error})",
                composition_root.display()
            )),
        }
    } else if legacy_workspace_layout_path.exists() {
        let _ = probe_legacy_layout_path(&legacy_workspace_layout_path);
        app.append_console_line(format!(
            "[composition_persistence.legacy_unsupported] left legacy layout unchanged: {}",
            legacy_workspace_layout_path.display()
        ));
    } else {
        app.append_console_line(format!(
            "[io] composition layout missing, keeping current layout: {}",
            composition_root.display()
        ));
    }
    app.append_console_line(format!("[io] loaded {}", path.display()));
    Ok(())
}

fn migration_failure_class_label(class: editor_core::MigrationFailureClass) -> &'static str {
    match class {
        editor_core::MigrationFailureClass::DecodeFailure => "decode-failure",
        editor_core::MigrationFailureClass::NormalizationFailure => "normalization-failure",
        editor_core::MigrationFailureClass::FormationFailure => "formation-failure",
        editor_core::MigrationFailureClass::ApplyFailure => "apply-failure",
    }
}

#[cfg(test)]
mod composition_tests {
    use super::*;

    #[test]
    fn pending_composition_coordination_blocks_save_before_io() {
        let mut app = RunenwerkEditorApp::new();
        let mut shell = RunenwerkEditorShellState::new();
        let candidate = shell.composition_runtime().clone();
        shell
            .queue_composition_restore(candidate)
            .expect("restore candidate should queue");

        assert!(ensure_composition_save_allowed(&mut app, &shell).is_err());
        assert!(app.console_lines().iter().any(|line| {
            line.text
                .contains("editor_composition.coordination.pending")
        }));
    }
}
