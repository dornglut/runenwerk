use std::time::Instant;

use editor_core::EditorMutationError;
use editor_definition::{
    EditorLabOperation, EditorLabOperationKind, EditorLabOperationReport, EditorLabOperationStatus,
    EditorWorkspaceSplitAxisDefinition,
};
use editor_shell::{
    EditorFreshTargetRequest, ShellCommand, TabStackPopupMenuKind, ToolbarCommandKind,
    UI_DESIGNER_WORKBENCH_TARGET_PROFILE, WorkspaceProfileLayoutSource,
    editor_design_system_recipe_library,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{redo_last_scene_change, undo_last_scene_change};
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
    EditorCommandAvailabilityContext, RunenwerkEditorShellState, RunenwerkWorkbenchComposition,
    editor_command_catalog,
};
use ui_theme::ThemeTokens;

mod composition;
mod workspace_io;

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
            workspace_io::switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ShellCommand::CloseWorkspaceProfile { profile_id } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace close command",
                    ))?;
            workspace_io::close_workspace_profile(app, shell_state, profile_id)?;
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
            composition::undo_composition_layout(app, shell_state.as_deref_mut())?;
        }
        ShellCommand::RedoCompositionLayout => {
            composition::redo_composition_layout(app, shell_state.as_deref_mut())?;
        }
        ShellCommand::SaveScene => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for save command",
                    ))?;
            workspace_io::save_scene_to_default_path(app, shell_state)?;
        }
        ShellCommand::LoadScene => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for load command",
                    ))?;
            workspace_io::load_scene_from_default_path(app, shell_state)?;
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
            composition::set_tab_stack_active_panel(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                panel_instance_id,
                projection_epoch,
            )?;
        }
        ShellCommand::CommitTabDrop {
            panel_instance_id,
            source_tab_stack_id,
            destination,
            projection_epoch,
        } => {
            composition::commit_tab_drop(
                shell_state.as_deref_mut(),
                panel_instance_id,
                source_tab_stack_id,
                destination,
                projection_epoch,
            )?;
        }
        ShellCommand::CommitCompositionDock {
            intent,
            projection_epoch,
        } => {
            composition::commit_composition_dock(
                shell_state.as_deref_mut(),
                intent,
                projection_epoch,
            )?;
        }
        ShellCommand::ResizeCompositionSplit {
            split,
            fraction,
            expected_revision,
            projection_epoch,
        } => {
            composition::resize_composition_split(
                app,
                shell_state.as_deref_mut(),
                split,
                fraction,
                expected_revision,
                projection_epoch,
            )?;
        }
        ShellCommand::CreatePanelTabStableKey {
            tab_stack_id,
            stable_surface_key,
            projection_epoch,
        } => {
            composition::create_panel_tab_stable_key(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                stable_surface_key,
                projection_epoch,
            )?;
        }
        ShellCommand::ClosePanelTab {
            tab_stack_id,
            panel_instance_id,
            projection_epoch,
        } => {
            composition::close_panel_tab(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                panel_instance_id,
                projection_epoch,
            )?;
        }
        ShellCommand::CloseOtherPanelTabs {
            tab_stack_id,
            keep_panel_instance_id,
            projection_epoch,
        } => {
            composition::close_other_panel_tabs(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                keep_panel_instance_id,
                projection_epoch,
            )?;
        }
        ShellCommand::SplitTabStackAreaStableKey {
            tab_stack_id,
            axis,
            stable_surface_key,
            projection_epoch,
        } => {
            composition::split_tab_stack_area_stable_key(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                axis,
                stable_surface_key,
                projection_epoch,
            )?;
        }
        ShellCommand::DuplicateTabStackArea {
            tab_stack_id,
            projection_epoch,
        } => {
            composition::duplicate_tab_stack_area(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                projection_epoch,
            )?;
        }
        ShellCommand::CloseTabStackArea {
            tab_stack_id,
            projection_epoch,
        } => {
            composition::close_tab_stack_area(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                projection_epoch,
            )?;
        }
        ShellCommand::ResetTabStackAreaStableKey {
            tab_stack_id,
            stable_surface_key,
            projection_epoch,
        } => {
            composition::reset_tab_stack_area_stable_key(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                stable_surface_key,
                projection_epoch,
            )?;
        }
        ShellCommand::LockTabStackAreaStableKey {
            tab_stack_id,
            locked_stable_surface_key,
            projection_epoch,
        } => {
            composition::lock_tab_stack_area_stable_key(
                app,
                shell_state.as_deref_mut(),
                tab_stack_id,
                locked_stable_surface_key,
                projection_epoch,
            )?;
        }
        ShellCommand::ActivateDocumentTab { document_id } => {
            app.runtime_mut()
                .session_mut()
                .activate_document(document_id)
                .map_err(|_| EditorMutationError::runtime_rejected("activate document failed"))?;
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
            projection_epoch,
        } => {
            crate::shell::dispatch::dispatch_surface_session_mutation(
                app,
                shell_state.as_deref_mut(),
                target,
                projection_epoch,
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
            workspace_io::save_scene_to_default_path(app, shell_state)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::OpenScene => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for open command",
            ))?;
            workspace_io::load_scene_from_default_path(app, shell_state)?;
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
            let profile_id = workspace_io::adjacent_workspace_profile(
                app.workbench_host().workspace_profile_registry(),
                shell_state.active_workspace_profile_id(),
                1,
            )
            .ok_or(EditorMutationError::runtime_rejected(
                "workspace profile missing",
            ))?;
            workspace_io::switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ToolbarCommandKind::PreviousWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            let profile_id = workspace_io::adjacent_workspace_profile(
                app.workbench_host().workspace_profile_registry(),
                shell_state.active_workspace_profile_id(),
                -1,
            )
            .ok_or(EditorMutationError::runtime_rejected(
                "workspace profile missing",
            ))?;
            workspace_io::switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ToolbarCommandKind::SaveWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            workspace_io::save_workspace_layout_for_active_profile(app, shell_state)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::LoadWorkspaceProfile(profile_id) => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            workspace_io::load_workspace_profile_layout(app, shell_state, profile_id)?;
            shell_state.close_toolbar_menu();
        }
        ToolbarCommandKind::NewWindow => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for new window command",
            ))?;
            if !workbench_allows_new_window(app.workbench_host().composition()) {
                return Err(EditorMutationError::runtime_rejected(
                    "new editor windows are unavailable in this workbench composition",
                ));
            }
            let profile_id = shell_state.active_workspace_profile_id();
            let profile = app.workbench_host().workspace_profile(profile_id).ok_or(
                EditorMutationError::runtime_rejected("active workspace profile missing"),
            )?;
            let WorkspaceProfileLayoutSource::AuthoredLayout { layout, .. } =
                &profile.layout_source
            else {
                return Err(EditorMutationError::runtime_rejected(
                    "active workspace profile has no normalized authored layout",
                ));
            };
            shell_state
                .queue_fresh_target_request(EditorFreshTargetRequest::new(
                    profile_id,
                    layout.clone(),
                ))
                .map_err(|_| {
                    EditorMutationError::runtime_rejected(
                        "fresh editor target request rejected by composition coordination",
                    )
                })?;
            shell_state.close_toolbar_menu();
            app.append_console_line(format!(
                "[composition] queued fresh editor target for profile {}",
                profile_id.raw()
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

fn workbench_allows_new_window(composition: RunenwerkWorkbenchComposition) -> bool {
    !matches!(
        composition,
        RunenwerkWorkbenchComposition::HeadlessValidation
            | RunenwerkWorkbenchComposition::Constrained
    )
}

