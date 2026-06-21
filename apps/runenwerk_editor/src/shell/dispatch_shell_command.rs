use std::{path::PathBuf, time::Instant};

use editor_core::{DirtyDocumentClosePolicy, EditorMutationError};
use editor_definition::{
    EditorLabOperation, EditorLabOperationKind, EditorLabOperationReport, EditorLabOperationStatus,
    EditorWorkspaceSplitAxisDefinition,
};
use editor_shell::{
    DockSplitSide, EditorCompositionRejection, EditorDockingDestination, EditorDockingIntent,
    EditorStructuralEditPlan, ShellCommand, TabDropDestination, TabStackPopupMenuKind,
    ToolbarCommandKind, UI_DESIGNER_WORKBENCH_TARGET_PROFILE, editor_design_system_recipe_library,
    plan_editor_activate_unit, plan_editor_close_other_units, plan_editor_close_stack,
    plan_editor_close_unit, plan_editor_create_unit, plan_editor_duplicate_stack,
    plan_editor_reset_stack, plan_editor_set_stack_lock, plan_editor_split_with_new_unit,
};
use ui_adaptive_composition::DockZone;
use ui_composition::CompositionPolicies;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{redo_last_scene_change, undo_last_scene_change};
use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
use crate::persistence::{
    default_composition_layout_root_for_profile, load_editor_composition_layout,
    load_scene_file_into_runtime_classified, probe_legacy_layout_path, read_retained_change_log,
    retained_change_log_path_for_scene, save_editor_composition_layout, write_retained_change_log,
    write_scene_file,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource, ViewportRenderStateCommandQueueResource,
};
use crate::shell::editor_lab_evidence::{
    EditorLabEvidenceArtifact, EditorLabEvidenceArtifactKind, EditorLabEvidenceArtifactProvenance,
    EditorLabPerformanceBaseline, EditorLabPerformanceBaselineKind,
};
use crate::shell::providers::{
    EditorShellFrameMetrics, EditorSurfaceProviderRegistry,
    build_editor_shell_frame_model_with_frame_metrics,
};
use crate::shell::self_authoring::EditorLabProductPathEvidenceCapture;
use crate::shell::{
    EditorCommandAvailabilityContext, EditorCompositionPolicy, ROTATE_TOOL_ID,
    RunenwerkEditorShellState, SCALE_TOOL_ID, SELECT_TOOL_ID, TRANSLATE_TOOL_ID,
    editor_command_catalog,
};
use ui_theme::ThemeTokens;

const DEFAULT_EDITOR_SCENE_PATH: &str = "editor-scenes/default.scene.ron";

pub fn dispatch_shell_command(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&mut RunenwerkEditorShellState>,
    command: ShellCommand,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    current_projection_epoch: Option<u64>,
) -> Result<(), EditorMutationError> {
    dispatch_shell_command_with_viewport_commands(
        app,
        shell_state,
        command,
        viewport_presentations,
        viewport_observations,
        tool_surface_bindings,
        None,
        current_projection_epoch,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn dispatch_shell_command_with_viewport_commands(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    command: ShellCommand,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
    current_projection_epoch: Option<u64>,
) -> Result<(), EditorMutationError> {
    if let (Some(command_epoch), Some(expected_epoch)) =
        (command.projection_epoch(), current_projection_epoch)
        && command_epoch != expected_epoch
    {
        return Ok(());
    }

    app.runtime_mut().record_workflow_event(
        editor_core::WorkflowEventKind::ShellCommandDispatched {
            command: shell_command_label(&command),
        },
    );

    match command {
        ShellCommand::ActivateSelectTool => {
            app.runtime_mut().set_active_tool_with_origin(
                Some(SELECT_TOOL_ID),
                editor_core::ChangeOrigin::EditorShell,
            );
            app.surface_sessions_mut()
                .close_all_viewport_tool_radial_menus();
            app.surface_sessions_mut().close_all_viewport_tools_menus();
        }
        ShellCommand::ActivateTranslateTool => {
            app.runtime_mut().set_active_tool_with_origin(
                Some(TRANSLATE_TOOL_ID),
                editor_core::ChangeOrigin::EditorShell,
            );
            app.surface_sessions_mut()
                .close_all_viewport_tool_radial_menus();
            app.surface_sessions_mut().close_all_viewport_tools_menus();
        }
        ShellCommand::ActivateRotateTool => {
            app.runtime_mut().set_active_tool_with_origin(
                Some(ROTATE_TOOL_ID),
                editor_core::ChangeOrigin::EditorShell,
            );
            app.surface_sessions_mut()
                .close_all_viewport_tool_radial_menus();
            app.surface_sessions_mut().close_all_viewport_tools_menus();
        }
        ShellCommand::ActivateScaleTool => {
            app.runtime_mut().set_active_tool_with_origin(
                Some(SCALE_TOOL_ID),
                editor_core::ChangeOrigin::EditorShell,
            );
            app.surface_sessions_mut()
                .close_all_viewport_tool_radial_menus();
            app.surface_sessions_mut().close_all_viewport_tools_menus();
        }
        ShellCommand::ToggleToolbarMenu { menu } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for toolbar menu command",
                    ))?;
            shell_state.toggle_toolbar_menu(menu);
        }
        ShellCommand::ToggleTabStackActionMenu {
            tab_stack_id,
            anchor_widget_id,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for tab stack action menu command",
                    ))?;
            shell_state.toggle_tab_stack_popup_menu(
                TabStackPopupMenuKind::AreaActions,
                tab_stack_id,
                anchor_widget_id,
            );
        }
        ShellCommand::ToggleTabStackSurfaceMenu {
            tab_stack_id,
            anchor_widget_id,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for tab stack surface menu command",
                    ))?;
            shell_state.toggle_tab_stack_popup_menu(
                TabStackPopupMenuKind::SurfaceKinds,
                tab_stack_id,
                anchor_widget_id,
            );
        }
        ShellCommand::ToggleTabStackCreateSurfaceMenu {
            tab_stack_id,
            anchor_widget_id,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for tab stack create-surface menu command",
                    ))?;
            shell_state.toggle_tab_stack_popup_menu(
                TabStackPopupMenuKind::CreateSurface,
                tab_stack_id,
                anchor_widget_id,
            );
        }
        ShellCommand::RunToolbarCommand { command } => {
            dispatch_toolbar_command(app, shell_state.as_deref_mut(), command)?;
        }
        ShellCommand::SwitchWorkspaceProfile { profile_id } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace switch command",
                    ))?;
            switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ShellCommand::CloseWorkspaceProfile { profile_id } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace close command",
                    ))?;
            close_workspace_profile(app, shell_state, profile_id)?;
        }
        ShellCommand::Undo => {
            if let Some(entry) =
                undo_last_scene_change(app.runtime_mut(), editor_core::ChangeOrigin::EditorShell)
                    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?
            {
                app.append_console_line(format!("[history] undo: {}", entry.transaction.label));
            }
        }
        ShellCommand::Redo => {
            if let Some(entry) =
                redo_last_scene_change(app.runtime_mut(), editor_core::ChangeOrigin::EditorShell)
                    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?
            {
                app.append_console_line(format!("[history] redo: {}", entry.transaction.label));
            }
        }
        ShellCommand::UndoCompositionLayout => {
            let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
                EditorMutationError::runtime_rejected("missing shell state for composition undo")
            })?;
            let policy = EditorCompositionPolicy;
            let policies = CompositionPolicies {
                lifecycle: &policy,
                capability: &policy,
                target: &policy,
            };
            shell_state
                .undo_structural_composition(policies)
                .map_err(|rejection| record_composition_rejection(app, rejection))?;
        }
        ShellCommand::RedoCompositionLayout => {
            let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
                EditorMutationError::runtime_rejected("missing shell state for composition redo")
            })?;
            let policy = EditorCompositionPolicy;
            let policies = CompositionPolicies {
                lifecycle: &policy,
                capability: &policy,
                target: &policy,
            };
            shell_state
                .redo_structural_composition(policies)
                .map_err(|rejection| record_composition_rejection(app, rejection))?;
        }
        ShellCommand::SaveScene => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for save command",
                    ))?;
            save_scene_to_default_path(app, shell_state)?;
        }
        ShellCommand::LoadScene => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for load command",
                    ))?;
            load_scene_from_default_path(app, shell_state)?;
        }
        ShellCommand::ToggleDebugLogs => {
            app.toggle_debug_logs_enabled();
            app.append_console_line(format!(
                "[debug] interaction logs {}",
                if app.debug_logs_enabled() {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
        }
        ShellCommand::SelectAsset {
            asset_id,
            projection_epoch: _,
        } => {
            app.asset_catalog_runtime_mut().select_asset(Some(asset_id));
            app.append_console_line(format!("[asset] selected asset {}", asset_id.raw()));
        }
        ShellCommand::LoadAssetCatalog {
            projection_epoch: _,
        } => {
            app.load_asset_project_catalog().map_err(|error| {
                app.append_console_error(format!("[asset] catalog load failed: {error}"));
                EditorMutationError::runtime_rejected("asset catalog load failed")
            })?;
        }
        ShellCommand::SaveAssetCatalog {
            projection_epoch: _,
        } => {
            app.save_asset_project_catalog().map_err(|error| {
                app.append_console_error(format!("[asset] catalog save failed: {error}"));
                EditorMutationError::runtime_rejected("asset catalog save failed")
            })?;
        }
        ShellCommand::ReimportAsset {
            asset_id,
            projection_epoch: _,
        } => {
            app.reimport_asset(asset_id).map_err(|error| {
                app.append_console_error(format!("[asset] reimport failed: {error}"));
                EditorMutationError::runtime_rejected("asset reimport failed")
            })?;
        }
        ShellCommand::ReimportSelectedAsset {
            projection_epoch: _,
        } => {
            app.reimport_selected_asset().map_err(|error| {
                app.append_console_error(format!("[asset] selected reimport failed: {error}"));
                EditorMutationError::runtime_rejected("asset selected reimport failed")
            })?;
        }
        ShellCommand::ClearAssetDiagnostics {
            projection_epoch: _,
        } => {
            app.asset_catalog_runtime_mut().clear_diagnostics();
            app.append_console_line("[asset] diagnostics cleared");
        }
        ShellCommand::SelectMaterialAsset {
            asset_id,
            projection_epoch: _,
        } => {
            app.select_material_asset(asset_id);
        }
        ShellCommand::BuildMaterialPreview {
            asset_id,
            projection_epoch: _,
        } => {
            app.rebuild_material_preview(asset_id).map_err(|error| {
                app.append_console_error(format!("[material] preview build failed: {error}"));
                EditorMutationError::runtime_rejected("material preview build failed")
            })?;
        }
        ShellCommand::BuildSelectedMaterialPreview {
            projection_epoch: _,
        } => {
            app.rebuild_selected_material_preview().map_err(|error| {
                app.append_console_error(format!(
                    "[material] selected preview build failed: {error}"
                ));
                EditorMutationError::runtime_rejected("selected material preview build failed")
            })?;
        }
        ShellCommand::ClearMaterialDiagnostics {
            projection_epoch: _,
        } => {
            app.clear_material_diagnostics();
        }
        ShellCommand::ApplyMaterialSurfaceAction {
            action,
            projection_epoch: _,
        } => {
            app.apply_material_surface_action(action)?;
        }
        ShellCommand::ApplyTextureSurfaceAction {
            action,
            projection_epoch: _,
        } => {
            app.apply_texture_surface_action(action);
        }
        ShellCommand::SaveEditorLabProjectPackage => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for Editor Lab project save",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .save_project_package_to_ron()
            {
                Ok(source) => app.append_console_line(format!(
                    "[editor-definition] saved Editor Lab project package ({} bytes)",
                    source.len()
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] project package save blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "Editor Lab project package save blocked",
                    ));
                }
            }
        }
        ShellCommand::ReloadEditorLabProjectPackage => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for Editor Lab project reload",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .reload_last_saved_project_package()
            {
                Ok(report) => app.append_console_line(format!(
                    "[editor-definition] reloaded Editor Lab project package drafts={} applied={} last_applied={}",
                    report.draft_count, report.applied_count, report.last_applied_count
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] project package reload blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "Editor Lab project package reload blocked",
                    ));
                }
            }
        }
        ShellCommand::BuildSelectedEditorDefinitionApplyReview => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition apply review",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .prepare_selected_apply_review()
            {
                Ok(review) => app.append_console_line(format!(
                    "[editor-definition] apply review built for {} diffs={} diagnostics={}",
                    review.display_name,
                    review.diff_rows.len(),
                    review.diagnostics.len()
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] apply review blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition apply review blocked",
                    ));
                }
            }
        }
        ShellCommand::RejectSelectedEditorDefinitionApplyReview => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition apply review reject",
                    ))?;
            match shell_state.self_authoring_mut().reject_last_apply_review() {
                Ok(review) => app.append_console_line(format!(
                    "[editor-definition] rejected apply review for {}",
                    review.display_name
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] apply review reject blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition apply review reject blocked",
                    ));
                }
            }
        }
        ShellCommand::CreateEditorWorkbenchCompositionPackage => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workbench composition creation",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .create_custom_workbench_package()
            {
                Ok(document_id) => app.append_console_line(format!(
                    "[editor-definition] created custom workbench package {}",
                    document_id.as_str()
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] custom workbench creation blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "custom workbench creation blocked",
                    ));
                }
            }
        }
        ShellCommand::ApplySelectedEditorDefinition => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition apply",
                    ))?;
            match shell_state.self_authoring_mut().apply_selected() {
                Ok(preview) => {
                    if let Some(document) = shell_state
                        .self_authoring()
                        .applied_document(&preview.document_id)
                        .cloned()
                    {
                        let review_id = shell_state
                            .self_authoring()
                            .last_apply_review()
                            .map(|review| review.id.clone());
                        app.queue_editor_definition_activation_for_review(review_id, document);
                    }
                    app.append_console_line(format!(
                        "[editor-definition] applied {}",
                        preview.display_name
                    ));
                }
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] apply blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition apply blocked",
                    ));
                }
            }
        }
        ShellCommand::ActivateSelectedEditorWorkbenchComposition => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workbench composition activation",
                    ))?;
            match shell_state
                .self_authoring()
                .selected_workbench_composition_payload()
            {
                Ok(payload) => {
                    let review_id = shell_state
                        .self_authoring()
                        .last_apply_review()
                        .map(|review| review.id.clone());
                    app.queue_editor_definition_activation_payload_for_review(review_id, payload);
                    app.append_console_line(
                        "[editor-definition] queued custom workbench activation".to_string(),
                    );
                }
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] workbench activation blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "workbench composition activation blocked",
                    ));
                }
            }
        }
        ShellCommand::ReloadSelectedEditorDefinitionLastApplied => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition last-applied reload",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .reload_selected_from_last_applied()
            {
                Ok(document) => app.append_console_line(format!(
                    "[editor-definition] reloaded last applied {}",
                    document.display_name
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] reload last applied blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition reload last applied blocked",
                    ));
                }
            }
        }
        ShellCommand::RollbackSelectedEditorDefinition => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition rollback",
                    ))?;
            match shell_state.self_authoring_mut().rollback_selected() {
                Ok(document) => {
                    app.append_console_line(format!(
                        "[editor-definition] rolled back {}",
                        document.display_name
                    ));
                }
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] rollback blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition rollback blocked",
                    ));
                }
            }
        }
        ShellCommand::ApplyEditorLabOperation { operation } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor lab operation",
                    ))?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::UndoEditorLabOperation => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor lab undo",
                    ))?;
            match shell_state.self_authoring_mut().undo_editor_lab_operation() {
                Ok(report) => append_editor_lab_restore_console_line(app, "undo", &report),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-lab-operation] undo blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor lab operation undo blocked",
                    ));
                }
            }
        }
        ShellCommand::RedoEditorLabOperation => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor lab redo",
                    ))?;
            match shell_state.self_authoring_mut().redo_editor_lab_operation() {
                Ok(report) => append_editor_lab_restore_console_line(app, "redo", &report),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-lab-operation] redo blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor lab operation redo blocked",
                    ));
                }
            }
        }
        ShellCommand::SetTabStackActivePanel {
            tab_stack_id,
            panel_instance_id,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "activate tab",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let unit = composition_unit_for_panel(shell_state, panel_instance_id)?;
            let plan = plan_editor_activate_unit(
                shell_state.composition_runtime(),
                stack,
                unit,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::CommitTabDrop {
            panel_instance_id,
            source_tab_stack_id,
            destination,
            projection_epoch,
        } => {
            let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
                EditorMutationError::runtime_rejected("missing shell state for composition docking")
            })?;
            if !shell_state.is_projection_epoch_current(projection_epoch) {
                return Err(EditorMutationError::runtime_rejected(
                    "stale editor composition docking projection",
                ));
            }
            let unit = shell_state
                .mounted_unit_id_for_panel(panel_instance_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "docked panel has no mounted composition unit",
                    )
                })?;
            let source_region = shell_state
                .region_id_for_tab_stack(source_tab_stack_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "docking source tab stack has no composition region",
                    )
                })?;
            let actual_source = shell_state
                .composition_runtime()
                .composition()
                .definition()
                .regions()
                .iter()
                .find(|region| region.kind.mounted_units().contains(&unit))
                .map(|region| region.id);
            if actual_source != Some(source_region) {
                return Err(EditorMutationError::runtime_rejected(
                    "docking source no longer owns the mounted unit",
                ));
            }
            let destination = docking_destination(shell_state, destination)?;
            shell_state.queue_docking_intent(EditorDockingIntent {
                source_revision: shell_state.composition_runtime().composition().revision(),
                unit,
                destination,
            });
        }
        ShellCommand::CommitCompositionDock {
            intent,
            projection_epoch,
        } => {
            let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
                EditorMutationError::runtime_rejected("missing shell state for composition docking")
            })?;
            if !shell_state.is_projection_epoch_current(projection_epoch) {
                return Err(EditorMutationError::runtime_rejected(
                    "stale editor composition docking projection",
                ));
            }
            if intent.source_revision != shell_state.composition_runtime().composition().revision()
            {
                return Err(EditorMutationError::runtime_rejected(
                    "stale editor composition docking revision",
                ));
            }
            shell_state.queue_docking_intent(intent);
        }
        ShellCommand::ResizeCompositionSplit {
            split,
            fraction,
            expected_revision,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "resize composition split",
            )?;
            if shell_state.composition_runtime().composition().revision() != expected_revision {
                return Err(EditorMutationError::runtime_rejected(
                    "stale editor composition resize revision",
                ));
            }
            let plan = editor_shell::plan_editor_resize_split(
                shell_state.composition_runtime(),
                split,
                fraction,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::CreatePanelTabStableKey {
            tab_stack_id,
            panel_kind,
            stable_surface_key,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "create tab",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let plan = plan_editor_create_unit(
                shell_state.composition_runtime(),
                stack,
                panel_kind,
                stable_surface_key,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::ClosePanelTab {
            tab_stack_id,
            panel_instance_id,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "close tab",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let unit = composition_unit_for_panel(shell_state, panel_instance_id)?;
            require_unit_source(shell_state, unit, stack)?;
            let plan = plan_editor_close_unit(
                shell_state.composition_runtime(),
                unit,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::CloseOtherPanelTabs {
            tab_stack_id,
            keep_panel_instance_id,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "close other tabs",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let keep = composition_unit_for_panel(shell_state, keep_panel_instance_id)?;
            let plan = plan_editor_close_other_units(
                shell_state.composition_runtime(),
                stack,
                keep,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::SplitTabStackAreaStableKey {
            tab_stack_id,
            axis,
            panel_kind,
            stable_surface_key,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "split area",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let plan = plan_editor_split_with_new_unit(
                shell_state.composition_runtime(),
                stack,
                axis,
                panel_kind,
                stable_surface_key,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::DuplicateTabStackArea {
            tab_stack_id,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "duplicate area",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let plan = plan_editor_duplicate_stack(
                shell_state.composition_runtime(),
                stack,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::CloseTabStackArea {
            tab_stack_id,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "close area",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let plan = plan_editor_close_stack(
                shell_state.composition_runtime(),
                stack,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::ResetTabStackAreaStableKey {
            tab_stack_id,
            panel_kind,
            stable_surface_key,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "reset area",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let plan = plan_editor_reset_stack(
                shell_state.composition_runtime(),
                stack,
                panel_kind,
                stable_surface_key,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::LockTabStackAreaStableKey {
            tab_stack_id,
            locked_stable_surface_key,
            projection_epoch,
        } => {
            let shell_state = require_composition_shell_state(
                shell_state.as_deref_mut(),
                projection_epoch,
                "change area lock",
            )?;
            let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
            let plan = plan_editor_set_stack_lock(
                shell_state.composition_runtime(),
                stack,
                locked_stable_surface_key,
                shell_state.composition_identity_allocator(),
            )
            .map_err(|rejection| record_composition_rejection(app, rejection))?;
            apply_editor_structural_plan(app, shell_state, plan)?;
        }
        ShellCommand::ActivateDocumentTab { document_id } => {
            app.runtime_mut()
                .session_mut()
                .activate_document(document_id)
                .map_err(|_| EditorMutationError::runtime_rejected("activate document failed"))?;
        }
        ShellCommand::CloseDocumentTab { document_id } => {
            app.runtime_mut()
                .session_mut()
                .close_document(document_id, DirtyDocumentClosePolicy::RejectDirty)
                .map_err(|_| EditorMutationError::runtime_rejected("close document failed"))?;
        }
        ShellCommand::SaveDocumentTab { document_id } => {
            app.runtime_mut()
                .session_mut()
                .mark_document_saved(document_id)
                .map_err(|_| EditorMutationError::runtime_rejected("save document failed"))?;
        }
        ShellCommand::SelectEditorDefinitionDocument { document_id } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition selection",
                    ))?;
            if shell_state
                .self_authoring_mut()
                .select_document_by_str(&document_id)
            {
                app.append_console_line(format!("[editor-definition] selected {document_id}"));
            } else {
                app.append_console_line(format!(
                    "[editor-definition] select blocked: unresolved document {document_id}"
                ));
            }
        }
        ShellCommand::DuplicateSelectedEditorDefinition => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition duplicate",
                    ))?;
            let new_id = shell_state
                .self_authoring()
                .generated_duplicate_id()
                .ok_or(EditorMutationError::runtime_rejected(
                    "editor definition duplicate id allocation failed",
                ))?;
            let display_name = format!("{} copy", new_id.as_str());
            match shell_state
                .self_authoring_mut()
                .duplicate_selected(new_id.clone(), display_name)
            {
                Ok(id) => app
                    .append_console_line(format!("[editor-definition] duplicated {}", id.as_str())),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] duplicate blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition duplicate blocked",
                    ));
                }
            }
        }
        ShellCommand::RenameSelectedEditorDefinition { display_name } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition rename",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "rename",
                EditorLabOperationKind::RenameDocument { display_name },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::DeleteSelectedEditorDefinition => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition delete",
                    ))?;
            match shell_state.self_authoring_mut().delete_selected() {
                Ok(document) => app.append_console_line(format!(
                    "[editor-definition] deleted {}",
                    document.display_name
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] delete blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition delete blocked",
                    ));
                }
            }
        }
        ShellCommand::ExportSelectedEditorDefinition => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition export",
                    ))?;
            match shell_state.self_authoring().export_selected_to_ron() {
                Ok(source) => app.append_console_line(format!(
                    "[editor-definition] export preview generated ({} bytes)",
                    source.len()
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] export blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition export blocked",
                    ));
                }
            }
        }
        ShellCommand::SelectEditorDefinitionUiNode { node_id } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition node selection",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .select_ui_node(node_id.clone())
            {
                Ok(()) => app
                    .append_console_line(format!("[editor-definition] selected UI node {node_id}")),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] node selection blocked: {}",
                        diagnostic.message
                    ));
                }
            }
        }
        ShellCommand::InsertSelectedEditorDefinitionRecipe { recipe_id } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition recipe insertion",
                    ))?;
            let library = editor_design_system_recipe_library();
            match shell_state.self_authoring_mut().insert_selected_ui_recipe(
                &library,
                ui_definition::UiRecipeId::new(recipe_id.clone()),
                ui_definition::UiRecipeTargetProfileId::new(UI_DESIGNER_WORKBENCH_TARGET_PROFILE),
            ) {
                Ok(report) => {
                    let diff_count = report
                        .diff
                        .as_ref()
                        .map(|diff| diff.changes.len())
                        .unwrap_or(0);
                    app.append_console_line(format!(
                        "[editor-definition] inserted recipe {recipe_id} (diff changes: {diff_count})"
                    ));
                }
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] recipe insertion blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition recipe insertion blocked",
                    ));
                }
            }
        }
        ShellCommand::SetEditorDefinitionRecipeCatalogFilter { query } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition recipe catalog filter",
                    ))?;
            shell_state
                .self_authoring_mut()
                .set_recipe_catalog_filter(query);
        }
        ShellCommand::CaptureUiDesignerScenarioEvidence => {
            let theme = ThemeTokens::default();
            let product_capture = {
                let shell_state_ref =
                    shell_state
                        .as_deref()
                        .ok_or(EditorMutationError::runtime_rejected(
                            "missing shell state for UI Designer scenario evidence capture",
                        ))?;
                capture_ui_designer_product_path_evidence(app, shell_state_ref, &theme)
            };
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for UI Designer scenario evidence capture",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .capture_pm005_scenario_evidence_packets_with_product_capture(
                    &theme,
                    product_capture,
                ) {
                Ok(packets) => {
                    app.append_console_line(format!(
                        "[editor-definition] captured UI Designer scenario evidence packets: {}",
                        packets.len()
                    ));
                }
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] scenario evidence capture blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "UI Designer scenario evidence capture blocked",
                    ));
                }
            }
        }
        ShellCommand::SetSelectedEditorDefinitionUiNodeText { node_id, text } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition node edit",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "ui_text",
                EditorLabOperationKind::SetUiNodeText { node_id, text },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::SetSelectedEditorThemeColor { token, value } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor theme edit",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "theme_color",
                EditorLabOperationKind::SetThemeColor { token, value },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::SetSelectedWorkbenchInstalledSuites { installed_suites } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workbench composition edit",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "workbench_installed_suites",
                EditorLabOperationKind::SetWorkbenchInstalledSuites {
                    installed_suites: parse_editor_definition_list_field(&installed_suites),
                },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::SetSelectedWorkbenchProfileRefs { profile_refs } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workbench composition edit",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "workbench_profile_refs",
                EditorLabOperationKind::SetWorkbenchProfileRefs {
                    profile_refs: parse_editor_definition_list_field(&profile_refs),
                },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::SetSelectedWorkbenchDefaultProfileRef { profile_ref } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workbench composition edit",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "workbench_default_profile",
                EditorLabOperationKind::SetWorkbenchDefaultProfileRef { profile_ref },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::AddSelectedEditorWorkspaceLayoutTab {
            label,
            tool_surface,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor workspace layout edit",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "workspace_tab",
                EditorLabOperationKind::AddWorkspaceLayoutTab {
                    label,
                    tool_surface,
                },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::SplitSelectedEditorWorkspaceLayoutRoot { axis } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor workspace layout split",
                    ))?;
            let axis = match axis.as_str() {
                "horizontal" => EditorWorkspaceSplitAxisDefinition::Horizontal,
                "vertical" => EditorWorkspaceSplitAxisDefinition::Vertical,
                _ => {
                    app.append_console_line(format!(
                        "[editor-definition] workspace split blocked: unsupported axis {axis}"
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor workspace layout split axis unsupported",
                    ));
                }
            };
            let operation = selected_editor_lab_operation(
                shell_state,
                "workspace_split",
                EditorLabOperationKind::SplitWorkspaceLayoutRoot { axis },
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::CloseSelectedEditorWorkspaceLayoutLastTab => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor workspace layout close tab",
                    ))?;
            let operation = selected_editor_lab_operation(
                shell_state,
                "workspace_close_tab",
                EditorLabOperationKind::CloseWorkspaceLayoutLastTab,
            )?;
            dispatch_editor_lab_operation(app, shell_state, operation)?;
        }
        ShellCommand::ApplySurfaceSessionMutation {
            target,
            mutation,
            projection_epoch: _,
        } => {
            crate::shell::dispatch::dispatch_surface_session_mutation(
                app,
                shell_state.as_deref_mut(),
                target,
                mutation,
            )?;
        }
        ShellCommand::ApplyEditorDomainMutation {
            target,
            mutation,
            projection_epoch: _,
        } => {
            crate::shell::dispatch::dispatch_editor_domain_mutation(
                app,
                shell_state.as_deref(),
                target,
                mutation,
                viewport_presentations,
                viewport_observations,
                tool_surface_bindings,
                viewport_render_commands,
            )?;
        }
        ShellCommand::DispatchSurfaceLocalAction { .. }
        | ShellCommand::DispatchSurfaceInteraction { .. } => {
            return Err(EditorMutationError::session_rejected(
                "surface provider command must be resolved through provider registry before dispatch",
            ));
        }
        ShellCommand::NoOp => {}
    }

    Ok(())
}

fn selected_editor_lab_operation(
    shell_state: &RunenwerkEditorShellState,
    family: &str,
    kind: EditorLabOperationKind,
) -> Result<EditorLabOperation, EditorMutationError> {
    let document_id = shell_state
        .self_authoring()
        .selected_document_id()
        .cloned()
        .ok_or(EditorMutationError::runtime_rejected(
            "no editor definition document is selected for operation",
        ))?;
    Ok(EditorLabOperation {
        id: shell_state.self_authoring().next_operation_id(family),
        document_id,
        target_profile: "editor.workbench".to_string(),
        kind,
        preview_only: false,
        source: Some("shell.dispatch".to_string()),
    })
}

fn parse_editor_definition_list_field(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn dispatch_editor_lab_operation(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    operation: EditorLabOperation,
) -> Result<(), EditorMutationError> {
    let report = shell_state
        .self_authoring_mut()
        .apply_editor_lab_operation(operation)
        .map_err(|diagnostic| {
            app.append_console_line(format!(
                "[editor-lab-operation] dispatch blocked: {}",
                diagnostic.message
            ));
            EditorMutationError::runtime_rejected("editor lab operation dispatch blocked")
        })?;
    match report.status {
        EditorLabOperationStatus::Accepted => {
            let diff_count = report
                .diff
                .as_ref()
                .map(|diff| diff.changes.len())
                .unwrap_or(0);
            app.append_console_line(format!(
                "[editor-lab-operation] applied {} (diff changes: {diff_count})",
                report.operation_id
            ));
            Ok(())
        }
        EditorLabOperationStatus::PreviewOnly => {
            app.append_console_line(format!(
                "[editor-lab-operation] previewed {}",
                report.operation_id
            ));
            Ok(())
        }
        EditorLabOperationStatus::Rejected => {
            let reason = report
                .diagnostics
                .first()
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("operation was rejected");
            app.append_console_line(format!(
                "[editor-lab-operation] rejected {}: {reason}",
                report.operation_id
            ));
            Err(EditorMutationError::runtime_rejected(
                "editor lab operation rejected",
            ))
        }
    }
}

fn append_editor_lab_restore_console_line(
    app: &mut RunenwerkEditorApp,
    action: &str,
    report: &EditorLabOperationReport,
) {
    app.append_console_line(format!(
        "[editor-lab-operation] {action} {}",
        report.operation_id
    ));
}

fn capture_ui_designer_product_path_evidence(
    app: &RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
    theme: &ThemeTokens,
) -> EditorLabProductPathEvidenceCapture {
    let registry = EditorSurfaceProviderRegistry::runenwerk_ui_designer_workbench();
    let started = Instant::now();
    let frame_model = build_editor_shell_frame_model_with_frame_metrics(
        app,
        shell_state,
        &registry,
        theme,
        Some(EditorShellFrameMetrics {
            fps_ema: 60.0,
            frame_ms_ema: 16.67,
        }),
        None,
        None,
        None,
    );
    let elapsed_micros = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let frame_debug = format!("{frame_model:#?}");
    let digest = format!("blake3:{}", blake3::hash(frame_debug.as_bytes()).to_hex());
    let artifact = EditorLabEvidenceArtifact::from_content(
        EditorLabEvidenceArtifactKind::ProviderSnapshot,
        format!("evidence://ui-designer/runtime/frame-model/{digest}"),
        frame_debug.as_bytes(),
        EditorLabEvidenceArtifactProvenance::ProductPath,
        "UI Designer workbench frame model built through the surface provider registry",
    );
    let baseline = EditorLabPerformanceBaseline::product_path(
        EditorLabPerformanceBaselineKind::FrameBuild,
        elapsed_micros,
        frame_model.surfaces.len().max(1),
        "editor.workbench frame model built through surface provider registry",
    );
    EditorLabProductPathEvidenceCapture::new([artifact], [baseline])
}

fn shell_command_label(command: &ShellCommand) -> &'static str {
    match command {
        ShellCommand::ActivateSelectTool => "ActivateSelectTool",
        ShellCommand::ActivateTranslateTool => "ActivateTranslateTool",
        ShellCommand::ActivateRotateTool => "ActivateRotateTool",
        ShellCommand::ActivateScaleTool => "ActivateScaleTool",
        ShellCommand::ToggleToolbarMenu { .. } => "ToggleToolbarMenu",
        ShellCommand::ToggleTabStackActionMenu { .. } => "ToggleTabStackActionMenu",
        ShellCommand::ToggleTabStackSurfaceMenu { .. } => "ToggleTabStackSurfaceMenu",
        ShellCommand::ToggleTabStackCreateSurfaceMenu { .. } => "ToggleTabStackCreateSurfaceMenu",
        ShellCommand::RunToolbarCommand { .. } => "RunToolbarCommand",
        ShellCommand::SwitchWorkspaceProfile { .. } => "SwitchWorkspaceProfile",
        ShellCommand::CloseWorkspaceProfile { .. } => "CloseWorkspaceProfile",
        ShellCommand::Undo => "Undo",
        ShellCommand::Redo => "Redo",
        ShellCommand::UndoCompositionLayout => "UndoCompositionLayout",
        ShellCommand::RedoCompositionLayout => "RedoCompositionLayout",
        ShellCommand::SaveScene => "SaveScene",
        ShellCommand::LoadScene => "LoadScene",
        ShellCommand::ToggleDebugLogs => "ToggleDebugLogs",
        ShellCommand::SelectAsset { .. } => "SelectAsset",
        ShellCommand::LoadAssetCatalog { .. } => "LoadAssetCatalog",
        ShellCommand::SaveAssetCatalog { .. } => "SaveAssetCatalog",
        ShellCommand::ReimportAsset { .. } => "ReimportAsset",
        ShellCommand::ReimportSelectedAsset { .. } => "ReimportSelectedAsset",
        ShellCommand::ClearAssetDiagnostics { .. } => "ClearAssetDiagnostics",
        ShellCommand::SelectMaterialAsset { .. } => "SelectMaterialAsset",
        ShellCommand::BuildMaterialPreview { .. } => "BuildMaterialPreview",
        ShellCommand::BuildSelectedMaterialPreview { .. } => "BuildSelectedMaterialPreview",
        ShellCommand::ClearMaterialDiagnostics { .. } => "ClearMaterialDiagnostics",
        ShellCommand::ApplyMaterialSurfaceAction { .. } => "ApplyMaterialSurfaceAction",
        ShellCommand::ApplyTextureSurfaceAction { .. } => "ApplyTextureSurfaceAction",
        ShellCommand::SetTabStackActivePanel { .. } => "SetTabStackActivePanel",
        ShellCommand::CommitTabDrop { .. } => "CommitTabDrop",
        ShellCommand::CommitCompositionDock { .. } => "CommitCompositionDock",
        ShellCommand::ResizeCompositionSplit { .. } => "ResizeCompositionSplit",
        ShellCommand::CreatePanelTabStableKey { .. } => "CreatePanelTabStableKey",
        ShellCommand::ClosePanelTab { .. } => "ClosePanelTab",
        ShellCommand::CloseOtherPanelTabs { .. } => "CloseOtherPanelTabs",
        ShellCommand::SplitTabStackAreaStableKey { .. } => "SplitTabStackAreaStableKey",
        ShellCommand::DuplicateTabStackArea { .. } => "DuplicateTabStackArea",
        ShellCommand::CloseTabStackArea { .. } => "CloseTabStackArea",
        ShellCommand::ResetTabStackAreaStableKey { .. } => "ResetTabStackAreaStableKey",
        ShellCommand::LockTabStackAreaStableKey { .. } => "LockTabStackAreaStableKey",
        ShellCommand::ActivateDocumentTab { .. } => "ActivateDocumentTab",
        ShellCommand::CloseDocumentTab { .. } => "CloseDocumentTab",
        ShellCommand::SaveDocumentTab { .. } => "SaveDocumentTab",
        ShellCommand::SelectEditorDefinitionDocument { .. } => "SelectEditorDefinitionDocument",
        ShellCommand::DuplicateSelectedEditorDefinition => "DuplicateSelectedEditorDefinition",
        ShellCommand::RenameSelectedEditorDefinition { .. } => "RenameSelectedEditorDefinition",
        ShellCommand::DeleteSelectedEditorDefinition => "DeleteSelectedEditorDefinition",
        ShellCommand::SaveEditorLabProjectPackage => "SaveEditorLabProjectPackage",
        ShellCommand::ReloadEditorLabProjectPackage => "ReloadEditorLabProjectPackage",
        ShellCommand::ExportSelectedEditorDefinition => "ExportSelectedEditorDefinition",
        ShellCommand::CreateEditorWorkbenchCompositionPackage => {
            "CreateEditorWorkbenchCompositionPackage"
        }
        ShellCommand::BuildSelectedEditorDefinitionApplyReview => {
            "BuildSelectedEditorDefinitionApplyReview"
        }
        ShellCommand::RejectSelectedEditorDefinitionApplyReview => {
            "RejectSelectedEditorDefinitionApplyReview"
        }
        ShellCommand::ApplySelectedEditorDefinition => "ApplySelectedEditorDefinition",
        ShellCommand::ActivateSelectedEditorWorkbenchComposition => {
            "ActivateSelectedEditorWorkbenchComposition"
        }
        ShellCommand::RollbackSelectedEditorDefinition => "RollbackSelectedEditorDefinition",
        ShellCommand::ReloadSelectedEditorDefinitionLastApplied => {
            "ReloadSelectedEditorDefinitionLastApplied"
        }
        ShellCommand::ApplyEditorLabOperation { .. } => "ApplyEditorLabOperation",
        ShellCommand::UndoEditorLabOperation => "UndoEditorLabOperation",
        ShellCommand::RedoEditorLabOperation => "RedoEditorLabOperation",
        ShellCommand::SelectEditorDefinitionUiNode { .. } => "SelectEditorDefinitionUiNode",
        ShellCommand::InsertSelectedEditorDefinitionRecipe { .. } => {
            "InsertSelectedEditorDefinitionRecipe"
        }
        ShellCommand::SetEditorDefinitionRecipeCatalogFilter { .. } => {
            "SetEditorDefinitionRecipeCatalogFilter"
        }
        ShellCommand::CaptureUiDesignerScenarioEvidence => "CaptureUiDesignerScenarioEvidence",
        ShellCommand::SetSelectedEditorDefinitionUiNodeText { .. } => {
            "SetSelectedEditorDefinitionUiNodeText"
        }
        ShellCommand::SetSelectedEditorThemeColor { .. } => "SetSelectedEditorThemeColor",
        ShellCommand::SetSelectedWorkbenchInstalledSuites { .. } => {
            "SetSelectedWorkbenchInstalledSuites"
        }
        ShellCommand::SetSelectedWorkbenchProfileRefs { .. } => "SetSelectedWorkbenchProfileRefs",
        ShellCommand::SetSelectedWorkbenchDefaultProfileRef { .. } => {
            "SetSelectedWorkbenchDefaultProfileRef"
        }
        ShellCommand::AddSelectedEditorWorkspaceLayoutTab { .. } => {
            "AddSelectedEditorWorkspaceLayoutTab"
        }
        ShellCommand::SplitSelectedEditorWorkspaceLayoutRoot { .. } => {
            "SplitSelectedEditorWorkspaceLayoutRoot"
        }
        ShellCommand::CloseSelectedEditorWorkspaceLayoutLastTab => {
            "CloseSelectedEditorWorkspaceLayoutLastTab"
        }
        ShellCommand::ApplySurfaceSessionMutation { .. } => "ApplySurfaceSessionMutation",
        ShellCommand::ApplyEditorDomainMutation { .. } => "ApplyEditorDomainMutation",
        ShellCommand::DispatchSurfaceLocalAction { .. } => "DispatchSurfaceLocalAction",
        ShellCommand::DispatchSurfaceInteraction { .. } => "DispatchSurfaceInteraction",
        ShellCommand::NoOp => "NoOp",
    }
}

fn default_scene_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_EDITOR_SCENE_PATH)
}

fn require_composition_shell_state<'a>(
    shell_state: Option<&'a mut RunenwerkEditorShellState>,
    projection_epoch: u64,
    operation: &'static str,
) -> Result<&'a mut RunenwerkEditorShellState, EditorMutationError> {
    let shell_state = shell_state.ok_or_else(|| {
        EditorMutationError::runtime_rejected("missing shell state for composition edit")
    })?;
    if !shell_state.is_projection_epoch_current(projection_epoch) {
        return Err(EditorMutationError::runtime_rejected(match operation {
            "activate tab" => "stale composition projection for tab activation",
            "create tab" => "stale composition projection for tab creation",
            "close tab" => "stale composition projection for tab close",
            "close other tabs" => "stale composition projection for close-other-tabs",
            "split area" => "stale composition projection for area split",
            "duplicate area" => "stale composition projection for area duplicate",
            "close area" => "stale composition projection for area close",
            "reset area" => "stale composition projection for area reset",
            "change area lock" => "stale composition projection for area lock",
            _ => "stale composition projection for structural edit",
        }));
    }
    Ok(shell_state)
}

fn composition_region_for_stack(
    shell_state: &RunenwerkEditorShellState,
    tab_stack_id: editor_shell::TabStackId,
) -> Result<ui_composition::RegionId, EditorMutationError> {
    shell_state
        .region_id_for_tab_stack(tab_stack_id)
        .ok_or_else(|| {
            EditorMutationError::runtime_rejected(
                "tab stack has no region in the current composition projection",
            )
        })
}

fn composition_unit_for_panel(
    shell_state: &RunenwerkEditorShellState,
    panel_instance_id: editor_shell::PanelInstanceId,
) -> Result<ui_composition::MountedUnitId, EditorMutationError> {
    shell_state
        .mounted_unit_id_for_panel(panel_instance_id)
        .ok_or_else(|| {
            EditorMutationError::runtime_rejected(
                "panel has no mounted unit in the current composition projection",
            )
        })
}

fn require_unit_source(
    shell_state: &RunenwerkEditorShellState,
    unit: ui_composition::MountedUnitId,
    expected: ui_composition::RegionId,
) -> Result<(), EditorMutationError> {
    let actual = shell_state
        .composition_runtime()
        .composition()
        .definition()
        .regions()
        .iter()
        .find(|region| region.kind.mounted_units().contains(&unit))
        .map(|region| region.id);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(EditorMutationError::runtime_rejected(
            "panel no longer belongs to the selected composition stack",
        ))
    }
}

fn apply_editor_structural_plan(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    plan: EditorStructuralEditPlan,
) -> Result<(), EditorMutationError> {
    let policy = EditorCompositionPolicy;
    let policies = CompositionPolicies {
        lifecycle: &policy,
        capability: &policy,
        target: &policy,
    };
    shell_state
        .apply_structural_edit_plan(plan, policies)
        .map_err(|rejection| record_composition_rejection(app, rejection))?;
    shell_state.close_tab_stack_popup_menu();
    Ok(())
}

fn record_composition_rejection(
    app: &mut RunenwerkEditorApp,
    rejection: EditorCompositionRejection,
) -> EditorMutationError {
    for diagnostic in rejection.diagnostics() {
        app.append_console_line(format!(
            "[{}] {}",
            diagnostic.code().as_str(),
            diagnostic.message()
        ));
    }
    EditorMutationError::runtime_rejected("editor composition structural edit rejected")
}

fn docking_destination(
    shell_state: &RunenwerkEditorShellState,
    destination: TabDropDestination,
) -> Result<EditorDockingDestination, EditorMutationError> {
    let region_destination = |target_region, ordinal, zone| {
        Ok(EditorDockingDestination::Region {
            target_region,
            ordinal,
            zone,
        })
    };
    match destination {
        TabDropDestination::TabStack {
            tab_stack_id,
            insert_index,
        } => region_destination(
            shell_state
                .region_id_for_tab_stack(tab_stack_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "docking destination tab stack has no composition region",
                    )
                })?,
            insert_index,
            DockZone::Center,
        ),
        TabDropDestination::SplitIntoArea {
            target_tab_stack_id,
            side,
        } => region_destination(
            shell_state
                .region_id_for_tab_stack(target_tab_stack_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "split destination tab stack has no composition region",
                    )
                })?,
            0,
            dock_zone(side),
        ),
        TabDropDestination::SplitIntoHost {
            target_host_id,
            side,
        } => region_destination(
            shell_state
                .stack_region_for_host(target_host_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "split destination host has no stack composition region",
                    )
                })?,
            0,
            dock_zone(side),
        ),
        TabDropDestination::SplitIntoRoot { side } => region_destination(
            shell_state.primary_stack_region().ok_or_else(|| {
                EditorMutationError::runtime_rejected(
                    "primary composition root has no stack destination",
                )
            })?,
            0,
            dock_zone(side),
        ),
        TabDropDestination::NewFloatingHost => Ok(EditorDockingDestination::NewTarget),
    }
}

fn dock_zone(side: DockSplitSide) -> DockZone {
    match side {
        DockSplitSide::Left => DockZone::Left,
        DockSplitSide::Right => DockZone::Right,
        DockSplitSide::Top => DockZone::Top,
        DockSplitSide::Bottom => DockZone::Bottom,
    }
}

fn dispatch_toolbar_command(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&mut RunenwerkEditorShellState>,
    command: ToolbarCommandKind,
) -> Result<(), EditorMutationError> {
    match command {
        ToolbarCommandKind::SaveScene => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for save command",
            ))?;
            save_scene_to_default_path(app, shell_state)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::OpenScene => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for open command",
            ))?;
            load_scene_from_default_path(app, shell_state)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::Undo => {
            if let Some(entry) =
                undo_last_scene_change(app.runtime_mut(), editor_core::ChangeOrigin::EditorShell)
                    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?
            {
                app.append_console_line(format!("[history] undo: {}", entry.transaction.label));
            }
        }
        ToolbarCommandKind::Redo => {
            if let Some(entry) =
                redo_last_scene_change(app.runtime_mut(), editor_core::ChangeOrigin::EditorShell)
                    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?
            {
                app.append_console_line(format!("[history] redo: {}", entry.transaction.label));
            }
        }
        ToolbarCommandKind::NextWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            let profile_id = adjacent_workspace_profile(
                app.workbench_host().workspace_profile_registry(),
                shell_state.active_workspace_profile_id(),
                1,
            )
            .ok_or(EditorMutationError::runtime_rejected(
                "workspace profile missing",
            ))?;
            switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ToolbarCommandKind::PreviousWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            let profile_id = adjacent_workspace_profile(
                app.workbench_host().workspace_profile_registry(),
                shell_state.active_workspace_profile_id(),
                -1,
            )
            .ok_or(EditorMutationError::runtime_rejected(
                "workspace profile missing",
            ))?;
            switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ToolbarCommandKind::SaveWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            save_workspace_layout_for_active_profile(app, shell_state)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::LoadWorkspaceProfile(profile_id) => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            load_workspace_profile_layout(app, shell_state, profile_id)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::NewWindow => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for new window command",
            ))?;
            let editor_window_id = shell_state.open_editor_window_for_active_workspace();
            shell_state.close_toolbar_menu();
            app.append_console_line(format!(
                "[ui] requested editor window {}",
                editor_window_id.raw()
            ));
        }
        ToolbarCommandKind::LoadCustomWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for custom workspace command",
            ))?;
            dispatch_shell_command(
                app,
                Some(shell_state),
                ShellCommand::ActivateSelectedEditorWorkbenchComposition,
                None,
                None,
                None,
                None,
            )?;
        }
        ToolbarCommandKind::AddWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for add workspace command",
            ))?;
            dispatch_shell_command(
                app,
                Some(shell_state),
                ShellCommand::CreateEditorWorkbenchCompositionPackage,
                None,
                None,
                None,
                None,
            )?;
        }
        ToolbarCommandKind::SaveSceneAs
        | ToolbarCommandKind::OpenRecent
        | ToolbarCommandKind::EditPreferences => {
            return Err(unavailable_toolbar_command(app, command));
        }
    }
    Ok(())
}

fn unavailable_toolbar_command(
    app: &mut RunenwerkEditorApp,
    command: ToolbarCommandKind,
) -> EditorMutationError {
    let availability = editor_command_catalog()
        .descriptor_for_toolbar_command(command)
        .map(|descriptor| {
            descriptor.availability(EditorCommandAvailabilityContext {
                can_undo: false,
                can_redo: false,
            })
        });
    let diagnostic_code = availability
        .and_then(|availability| availability.diagnostic_code())
        .unwrap_or("editor.command.unavailable.unknown");
    let reason = availability
        .and_then(|availability| availability.reason())
        .unwrap_or("command is unavailable");

    app.append_console_line(format!(
        "[ui:{diagnostic_code}] command unavailable: {reason}"
    ));
    EditorMutationError::runtime_rejected(reason)
}

fn adjacent_workspace_profile(
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

fn switch_workspace_profile(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    profile_id: editor_shell::WorkspaceProfileId,
) -> Result<(), EditorMutationError> {
    load_workspace_profile_layout(app, shell_state, profile_id)
}

fn close_workspace_profile(
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

fn load_workspace_profile_layout(
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
            .install_composition_runtime(runtime)
            .map_err(|_| EditorMutationError::runtime_rejected("composition install failed"))?;
    } else {
        shell_state
            .activate_workspace_profile_ref_with_registry(
                &profile.profile_ref,
                app.workbench_host().workspace_profile_registry(),
                app.workbench_host().tool_surface_registry(),
            )
            .map_err(|_| {
                EditorMutationError::runtime_rejected("composition profile import failed")
            })?;
    }
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

fn save_workspace_layout_for_active_profile(
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

fn save_scene_to_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    ensure_composition_save_allowed(app, shell_state)?;
    let path = default_scene_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create editor scene folder")
        })?;
    }

    write_scene_file(&path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save editor scene"))?;
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

fn load_scene_from_default_path(
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

    {
        let runtime = app.runtime_mut();
        runtime.prepare_for_scene_load();
        register_mvp_component_types(runtime);
    }

    let migration = match load_scene_file_into_runtime_classified(&path, app.runtime_mut()) {
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
    bootstrap_mvp_scene_if_empty(app.runtime_mut())?;
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
                shell_state
                    .install_composition_runtime(runtime)
                    .map_err(|_| {
                        EditorMutationError::runtime_rejected("composition install failed")
                    })?;
                app.prune_surface_sessions_for_composition(shell_state.composition_runtime());
                app.append_console_line(format!(
                    "[io] loaded composition layout {}",
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
        shell.set_composition_coordination_pending(true);

        assert!(ensure_composition_save_allowed(&mut app, &shell).is_err());
        assert!(app.console_lines().iter().any(|line| {
            line.text
                .contains("editor_composition.coordination.pending")
        }));
    }
}

#[cfg(all(test, any()))]
mod tests {
    use super::*;
    use editor_shell::{
        MODELLING_WORKSPACE_PROFILE_ID, PanelKind, SCENE_WORKSPACE_PROFILE_ID,
        WorkspaceIdentityAllocator, reduce_workspace,
    };

    fn workspace_profile(
        profile_id: editor_shell::WorkspaceProfileId,
    ) -> editor_shell::WorkspaceProfile {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        host.workspace_profile(profile_id)
            .expect("workspace profile should exist")
            .clone()
    }

    fn scene_profile() -> editor_shell::WorkspaceProfile {
        workspace_profile(SCENE_WORKSPACE_PROFILE_ID)
    }

    fn modelling_profile() -> editor_shell::WorkspaceProfile {
        workspace_profile(MODELLING_WORKSPACE_PROFILE_ID)
    }

    fn default_scene_workspace() -> editor_shell::WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        scene_profile().build_default_workspace_state(workspace_id, &mut allocator)
    }

    fn workspace_surface_order(workspace: &editor_shell::WorkspaceState) -> Vec<String> {
        workspace
            .tab_stacks()
            .flat_map(|stack| stack.ordered_panels.iter())
            .filter_map(|panel_id| workspace.panel(*panel_id))
            .filter_map(|panel| panel.active_tool_surface)
            .filter_map(|surface_id| workspace.tool_surface(surface_id))
            .map(|surface| surface.stable_surface_key().as_str().to_string())
            .collect()
    }

    fn panel_and_stack_by_kind(
        workspace: &editor_shell::WorkspaceState,
        kind: PanelKind,
    ) -> (editor_shell::PanelInstanceId, editor_shell::TabStackId) {
        let panel_id = workspace
            .panels()
            .find(|panel| panel.panel_kind == kind)
            .expect("panel kind should exist")
            .id;
        let stack_id = workspace
            .tab_stacks()
            .find(|stack| stack.ordered_panels.contains(&panel_id))
            .expect("panel should be in a stack")
            .id;
        (panel_id, stack_id)
    }

    fn legacy_layout_result(
        workspace_state: editor_shell::WorkspaceState,
    ) -> WorkspaceLayoutReadResult {
        WorkspaceLayoutReadResult {
            workspace_state,
            workspace_profile_id: Some(SCENE_WORKSPACE_PROFILE_ID),
            layout_template: None,
            layout_template_version: None,
            last_saved_at_unix_seconds: None,
        }
    }

    #[test]
    fn material_epoch_stale_shell_commands_do_not_mutate_material_workflow_state() {
        let mut app = RunenwerkEditorApp::new();
        let asset_id = asset::asset_id(21);
        app.material_lab_runtime_mut()
            .record_diagnostic(asset::AssetDiagnosticRecord::error(
                asset::AssetDiagnosticCode::RatificationRejected,
                "existing diagnostic",
            ));

        for command in [
            ShellCommand::SelectMaterialAsset {
                asset_id,
                projection_epoch: 1,
            },
            ShellCommand::BuildMaterialPreview {
                asset_id,
                projection_epoch: 1,
            },
            ShellCommand::BuildSelectedMaterialPreview {
                projection_epoch: 1,
            },
            ShellCommand::ClearMaterialDiagnostics {
                projection_epoch: 1,
            },
            ShellCommand::ApplyMaterialSurfaceAction {
                action: editor_shell::MaterialSurfaceAction::PanGraph {
                    delta_x: 8,
                    delta_y: -4,
                },
                projection_epoch: 1,
            },
        ] {
            dispatch_shell_command(&mut app, None, command, None, None, None, Some(2))
                .expect("stale material command should fail closed");
        }

        assert_eq!(
            app.material_lab_runtime().selected_material_asset_id(),
            None
        );
        assert!(app.material_lab_runtime().active_preview().is_none());
        assert_eq!(app.material_lab_runtime().diagnostics().len(), 1);
        assert_eq!(app.pending_material_preview_publication_count(), 0);
    }

    #[test]
    fn graph_interaction_epoch_mismatch_fails_closed() {
        let mut app = RunenwerkEditorApp::new();
        let command = ShellCommand::ApplyMaterialSurfaceAction {
            action: editor_shell::MaterialSurfaceAction::SelectGraphNode {
                node_id: graph::NodeId::new(9),
            },
            projection_epoch: 1,
        };

        dispatch_shell_command(&mut app, None, command, None, None, None, Some(2))
            .expect("stale material graph interaction command should fail closed");

        assert!(
            app.material_lab_runtime().selected_graph_nodes().is_empty(),
            "stale graph interaction must not mutate Material Lab selection"
        );
    }

    #[test]
    fn graph_interaction_without_active_source_fails_closed() {
        let mut app = RunenwerkEditorApp::new();
        let command = ShellCommand::ApplyMaterialSurfaceAction {
            action: editor_shell::MaterialSurfaceAction::MoveGraphNode {
                node_id: graph::NodeId::new(9),
                delta_x: 10,
                delta_y: -4,
            },
            projection_epoch: 2,
        };

        let result = dispatch_shell_command(&mut app, None, command, None, None, None, Some(2));

        assert!(
            result.is_err(),
            "source-backed graph edits must fail closed when no material source is active"
        );
        assert!(
            app.material_lab_runtime()
                .active_source_document()
                .is_none(),
            "missing source failure must not synthesize Material Lab source state"
        );
    }

    #[test]
    fn unavailable_toolbar_command_emits_catalog_diagnostic() {
        let mut app = RunenwerkEditorApp::new();

        let result = dispatch_shell_command(
            &mut app,
            None,
            ShellCommand::RunToolbarCommand {
                command: ToolbarCommandKind::SaveSceneAs,
            },
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        let line = app
            .console_lines()
            .last()
            .expect("unavailable command should append a console diagnostic");
        assert!(
            line.text
                .contains("[ui:editor.command.unavailable.save_as]")
        );
        assert!(line.text.contains("save as is not implemented yet"));
    }

    #[test]
    fn stale_legacy_profile_layout_requires_default_scene_graph() {
        let workspace = default_scene_workspace();
        let (viewport_panel, viewport_stack) =
            panel_and_stack_by_kind(&workspace, PanelKind::Viewport);
        let (_, outliner_stack) = panel_and_stack_by_kind(&workspace, PanelKind::Outliner);
        let stale = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelBetweenTabStacks {
                panel_id: viewport_panel,
                source_tab_stack_id: viewport_stack,
                destination_tab_stack_id: outliner_stack,
                destination_index: 0,
                activate_panel: true,
            },
        )
        .expect("stale graph should still be structurally valid");
        let profile = scene_profile();
        let saved = legacy_layout_result(stale);

        assert!(!workspace_layout_matches_profile(&saved, &profile));
    }

    #[test]
    fn tagged_profile_layout_accepts_matching_template_metadata() {
        let workspace = default_scene_workspace();
        let profile = scene_profile();
        let saved = WorkspaceLayoutReadResult {
            workspace_state: workspace,
            workspace_profile_id: Some(SCENE_WORKSPACE_PROFILE_ID),
            layout_template: Some(profile.default_layout_template.contract_id().to_string()),
            layout_template_version: Some(profile.default_layout_template.contract_version()),
            last_saved_at_unix_seconds: Some(1),
        };

        assert!(workspace_layout_matches_profile(&saved, &profile));
    }

    #[test]
    fn scene_derived_modelling_template_metadata_is_rejected_as_stale() {
        let workspace = default_scene_workspace();
        let profile = modelling_profile();
        let saved = WorkspaceLayoutReadResult {
            workspace_state: workspace,
            workspace_profile_id: Some(MODELLING_WORKSPACE_PROFILE_ID),
            layout_template: Some("scene-derived.modelling".to_string()),
            layout_template_version: Some(profile.default_layout_template.contract_version()),
            last_saved_at_unix_seconds: Some(1),
        };

        assert!(!workspace_layout_matches_profile(&saved, &profile));
    }

    #[test]
    fn fallback_default_workspace_uses_registry_when_available() {
        let app = RunenwerkEditorApp::new();
        let profile = scene_profile();

        let workspace = build_default_workspace_for_profile(&app, &profile)
            .expect("profile fallback should build through hosted registry");

        let report = workspace.validate_tool_surface_registry_compatibility(
            app.workbench_host().tool_surface_registry(),
        );
        assert!(report.is_fully_compatible());
    }

    #[test]
    fn workspace_layout_load_rejection_classifies_unsupported_saved_schema() {
        let error = anyhow::anyhow!("persisted workspace version 4 is unsupported")
            .context("failed to validate persisted workspace layout");

        let rejection = workspace_layout_load_rejection(&error);

        assert_eq!(
            rejection.message,
            "failed to load workspace layout: saved schema unsupported"
        );
        assert!(error_chain_summary(&error).contains("persisted workspace version 4"));
    }

    #[test]
    fn workspace_layout_load_fallback_preserves_legacy_behavior() {
        let app = RunenwerkEditorApp::new();
        let profile = scene_profile();
        let legacy_workspace = default_scene_workspace();

        let registry_workspace = build_default_workspace_for_profile(&app, &profile)
            .expect("profile fallback should build through hosted registry");

        assert_eq!(
            workspace_surface_order(&registry_workspace),
            workspace_surface_order(&legacy_workspace)
        );
        assert_eq!(
            profile
                .default_layout_template
                .default_graph_matches(&registry_workspace),
            profile
                .default_layout_template
                .default_graph_matches(&legacy_workspace)
        );
    }
}
