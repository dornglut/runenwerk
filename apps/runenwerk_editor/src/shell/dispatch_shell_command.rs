use std::path::PathBuf;

use editor_core::{DirtyDocumentClosePolicy, EditorMutationError};
use editor_shell::{
    FloatingHostBounds, ShellCommand, TabDropDestination, TabStackPopupMenuKind,
    ToolbarCommandKind, WorkspaceMutation, default_workspace_profile_registry,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{redo_last_scene_change, undo_last_scene_change};
use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
use crate::persistence::{
    WorkspaceLayoutReadResult, default_workspace_layout_path_for_profile,
    legacy_workspace_layout_path_for_scene, load_scene_file_into_runtime_classified,
    read_retained_change_log, read_workspace_layout_with_metadata,
    retained_change_log_path_for_scene, write_retained_change_log, write_scene_file,
    write_workspace_layout_for_profile,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource, ViewportRenderStateCommandQueueResource,
};
use crate::shell::dispatch::panel_kind_for_tool_surface_kind;
use crate::shell::{
    ROTATE_TOOL_ID, RunenwerkEditorShellState, SCALE_TOOL_ID, SELECT_TOOL_ID, TRANSLATE_TOOL_ID,
};

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
                        app.queue_editor_definition_activation(document);
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
        ShellCommand::SetTabStackActivePanel {
            tab_stack_id,
            panel_instance_id,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state
                .apply_workspace_mutation(WorkspaceMutation::SetTabStackActivePanel {
                    tab_stack_id,
                    active_panel: Some(panel_instance_id),
                })
                .map_err(|_| EditorMutationError::runtime_rejected("workspace mutation failed"))?;
        }
        ShellCommand::CommitTabDrop {
            panel_instance_id,
            source_tab_stack_id,
            destination,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            apply_tab_drop(
                shell_state,
                panel_instance_id,
                source_tab_stack_id,
                destination,
            )
            .map_err(|_| EditorMutationError::runtime_rejected("workspace tab drop failed"))?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::SwitchPanelToolSurfaceKind {
            panel_instance_id,
            tool_surface_kind,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for surface switch command",
                    ))?;
            shell_state
                .switch_panel_tool_surface_kind(panel_instance_id, tool_surface_kind)
                .map_err(|_| EditorMutationError::runtime_rejected("surface switch failed"))?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::CreatePanelTab {
            tab_stack_id,
            tool_surface_kind,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state.close_tab_stack_popup_menu();
            shell_state
                .try_apply_workspace_mutation_with_allocations(|allocator| {
                    let panel_id = allocator.allocate_panel_instance_id();
                    let tool_surface_id = allocator.allocate_tool_surface_instance_id();
                    Ok((
                        WorkspaceMutation::add_panel_tab_legacy(
                            tab_stack_id,
                            panel_id,
                            panel_kind_for_tool_surface_kind(tool_surface_kind),
                            tool_surface_id,
                            tool_surface_kind,
                            true,
                        )?,
                        (),
                    ))
                })
                .map_err(|_| EditorMutationError::runtime_rejected("create panel tab failed"))?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::ClosePanelTab {
            tab_stack_id,
            panel_instance_id,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            let tab_count = shell_state
                .workspace_state()
                .tab_stack(tab_stack_id)
                .map(|stack| stack.ordered_panels.len())
                .unwrap_or(0);
            if tab_count <= 1 {
                shell_state
                    .apply_workspace_mutation(WorkspaceMutation::CloseTabStackArea { tab_stack_id })
                    .map_err(|_| EditorMutationError::runtime_rejected("close area failed"))?;
            } else {
                shell_state
                    .apply_workspace_mutation(WorkspaceMutation::ClosePanelTab {
                        tab_stack_id,
                        panel_id: panel_instance_id,
                    })
                    .map_err(|_| EditorMutationError::runtime_rejected("close panel tab failed"))?;
            }
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::CloseOtherPanelTabs {
            tab_stack_id,
            keep_panel_instance_id,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state
                .apply_workspace_mutation(WorkspaceMutation::CloseOtherPanelTabs {
                    tab_stack_id,
                    keep_panel_id: keep_panel_instance_id,
                })
                .map_err(|_| {
                    EditorMutationError::runtime_rejected("close other panel tabs failed")
                })?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::SplitTabStackArea {
            tab_stack_id,
            axis,
            tool_surface_kind,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state.close_tab_stack_popup_menu();
            shell_state
                .try_apply_workspace_mutation_with_allocations(|allocator| {
                    let split_host_id = allocator.allocate_panel_host_id();
                    let first_child_host_id = allocator.allocate_panel_host_id();
                    let second_child_host_id = allocator.allocate_panel_host_id();
                    let new_tab_stack_id = allocator.allocate_tab_stack_id();
                    let new_panel_id = allocator.allocate_panel_instance_id();
                    let new_tool_surface_id = allocator.allocate_tool_surface_instance_id();
                    Ok((
                        WorkspaceMutation::split_tab_stack_area_legacy(
                            tab_stack_id,
                            axis,
                            split_host_id,
                            first_child_host_id,
                            second_child_host_id,
                            new_tab_stack_id,
                            new_panel_id,
                            panel_kind_for_tool_surface_kind(tool_surface_kind),
                            new_tool_surface_id,
                            tool_surface_kind,
                            0.5,
                        )?,
                        (),
                    ))
                })
                .map_err(|_| {
                    EditorMutationError::runtime_rejected("split tab stack area failed")
                })?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::DuplicateTabStackArea {
            tab_stack_id,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state.close_tab_stack_popup_menu();
            shell_state
                .apply_workspace_mutation_with_allocations(|allocator| {
                    let new_panel_id = allocator.allocate_panel_instance_id();
                    let new_tool_surface_id = allocator.allocate_tool_surface_instance_id();
                    (
                        WorkspaceMutation::DuplicateTabStackArea {
                            tab_stack_id,
                            new_panel_id,
                            new_tool_surface_id,
                        },
                        (),
                    )
                })
                .map_err(|_| {
                    EditorMutationError::runtime_rejected("duplicate tab stack area failed")
                })?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::CloseTabStackArea {
            tab_stack_id,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state.close_tab_stack_popup_menu();
            shell_state
                .apply_workspace_mutation(WorkspaceMutation::CloseTabStackArea { tab_stack_id })
                .map_err(|_| {
                    EditorMutationError::runtime_rejected("close tab stack area failed")
                })?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::ResetTabStackArea {
            tab_stack_id,
            tool_surface_kind,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state.close_tab_stack_popup_menu();
            shell_state
                .try_apply_workspace_mutation_with_allocations(|allocator| {
                    let panel_id = allocator.allocate_panel_instance_id();
                    let tool_surface_id = allocator.allocate_tool_surface_instance_id();
                    Ok((
                        WorkspaceMutation::reset_tab_stack_area_legacy(
                            tab_stack_id,
                            panel_id,
                            tool_surface_id,
                            tool_surface_kind,
                        )?,
                        (),
                    ))
                })
                .map_err(|_| {
                    EditorMutationError::runtime_rejected("reset tab stack area failed")
                })?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::LockTabStackAreaType {
            tab_stack_id,
            locked_tool_surface_kind,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state.close_tab_stack_popup_menu();
            let lock_mutation = WorkspaceMutation::lock_tab_stack_area_type_legacy(
                tab_stack_id,
                locked_tool_surface_kind,
            )
            .map_err(|_| EditorMutationError::runtime_rejected("lock tab stack area failed"))?;
            shell_state
                .apply_workspace_mutation(lock_mutation)
                .map_err(|_| EditorMutationError::runtime_rejected("lock tab stack area failed"))?;
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
            match shell_state
                .self_authoring_mut()
                .rename_selected(display_name.clone())
            {
                Ok(()) => app.append_console_line(format!(
                    "[editor-definition] renamed selected definition to {display_name}"
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] rename blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition rename blocked",
                    ));
                }
            }
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
        ShellCommand::SetSelectedEditorDefinitionUiNodeText { node_id, text } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor definition node edit",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .set_selected_ui_node_text(&node_id, text.clone())
            {
                Ok(()) => app.append_console_line(format!(
                    "[editor-definition] set text on UI node {node_id}"
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] text edit blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor definition text edit blocked",
                    ));
                }
            }
        }
        ShellCommand::SetSelectedEditorThemeColor { token, value } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor theme edit",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .set_selected_theme_color(&token, value.clone())
            {
                Ok(()) => app.append_console_line(format!(
                    "[editor-definition] set theme color {token}={value}"
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] theme edit blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor theme edit blocked",
                    ));
                }
            }
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
            match shell_state
                .self_authoring_mut()
                .add_selected_workspace_layout_tab(label.clone(), tool_surface.clone())
            {
                Ok(tab_id) => app.append_console_line(format!(
                    "[editor-definition] added workspace layout tab {tab_id}"
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] workspace layout tab edit blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor workspace layout tab edit blocked",
                    ));
                }
            }
        }
        ShellCommand::SplitSelectedEditorWorkspaceLayoutRoot { axis } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor workspace layout split",
                    ))?;
            let axis = match axis.as_str() {
                "horizontal" => editor_definition::EditorWorkspaceSplitAxisDefinition::Horizontal,
                "vertical" => editor_definition::EditorWorkspaceSplitAxisDefinition::Vertical,
                _ => {
                    app.append_console_line(format!(
                        "[editor-definition] workspace split blocked: unsupported axis {axis}"
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor workspace layout split axis unsupported",
                    ));
                }
            };
            match shell_state
                .self_authoring_mut()
                .split_selected_workspace_layout_root(axis)
            {
                Ok(()) => app.append_console_line(
                    "[editor-definition] split workspace layout root".to_string(),
                ),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] workspace split blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor workspace layout split blocked",
                    ));
                }
            }
        }
        ShellCommand::CloseSelectedEditorWorkspaceLayoutLastTab => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for editor workspace layout close tab",
                    ))?;
            match shell_state
                .self_authoring_mut()
                .close_selected_workspace_layout_last_tab()
            {
                Ok(tab) => app.append_console_line(format!(
                    "[editor-definition] closed workspace layout tab {}",
                    tab.id
                )),
                Err(diagnostic) => {
                    app.append_console_line(format!(
                        "[editor-definition] workspace close tab blocked: {}",
                        diagnostic.message
                    ));
                    return Err(EditorMutationError::runtime_rejected(
                        "editor workspace layout close tab blocked",
                    ));
                }
            }
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
        ShellCommand::SetTabStackActivePanel { .. } => "SetTabStackActivePanel",
        ShellCommand::CommitTabDrop { .. } => "CommitTabDrop",
        ShellCommand::SwitchPanelToolSurfaceKind { .. } => "SwitchPanelToolSurfaceKind",
        ShellCommand::CreatePanelTab { .. } => "CreatePanelTab",
        ShellCommand::ClosePanelTab { .. } => "ClosePanelTab",
        ShellCommand::CloseOtherPanelTabs { .. } => "CloseOtherPanelTabs",
        ShellCommand::SplitTabStackArea { .. } => "SplitTabStackArea",
        ShellCommand::DuplicateTabStackArea { .. } => "DuplicateTabStackArea",
        ShellCommand::CloseTabStackArea { .. } => "CloseTabStackArea",
        ShellCommand::ResetTabStackArea { .. } => "ResetTabStackArea",
        ShellCommand::LockTabStackAreaType { .. } => "LockTabStackAreaType",
        ShellCommand::ActivateDocumentTab { .. } => "ActivateDocumentTab",
        ShellCommand::CloseDocumentTab { .. } => "CloseDocumentTab",
        ShellCommand::SaveDocumentTab { .. } => "SaveDocumentTab",
        ShellCommand::SelectEditorDefinitionDocument { .. } => "SelectEditorDefinitionDocument",
        ShellCommand::DuplicateSelectedEditorDefinition => "DuplicateSelectedEditorDefinition",
        ShellCommand::RenameSelectedEditorDefinition { .. } => "RenameSelectedEditorDefinition",
        ShellCommand::DeleteSelectedEditorDefinition => "DeleteSelectedEditorDefinition",
        ShellCommand::ExportSelectedEditorDefinition => "ExportSelectedEditorDefinition",
        ShellCommand::ApplySelectedEditorDefinition => "ApplySelectedEditorDefinition",
        ShellCommand::RollbackSelectedEditorDefinition => "RollbackSelectedEditorDefinition",
        ShellCommand::SelectEditorDefinitionUiNode { .. } => "SelectEditorDefinitionUiNode",
        ShellCommand::SetSelectedEditorDefinitionUiNodeText { .. } => {
            "SetSelectedEditorDefinitionUiNodeText"
        }
        ShellCommand::SetSelectedEditorThemeColor { .. } => "SetSelectedEditorThemeColor",
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

fn apply_tab_drop(
    shell_state: &mut RunenwerkEditorShellState,
    panel_instance_id: editor_shell::PanelInstanceId,
    source_tab_stack_id: editor_shell::TabStackId,
    destination: TabDropDestination,
) -> Result<(), editor_shell::WorkspaceStateError> {
    match destination {
        TabDropDestination::TabStack {
            tab_stack_id,
            insert_index,
        } => shell_state.apply_workspace_mutation(WorkspaceMutation::MovePanelBetweenTabStacks {
            panel_id: panel_instance_id,
            source_tab_stack_id,
            destination_tab_stack_id: tab_stack_id,
            destination_index: insert_index,
            activate_panel: true,
        }),
        TabDropDestination::SplitIntoArea {
            target_tab_stack_id,
            side,
        } => shell_state.apply_workspace_mutation_with_allocations(|allocator| {
            let split_host_id = allocator.allocate_panel_host_id();
            let target_child_host_id = allocator.allocate_panel_host_id();
            let new_child_host_id = allocator.allocate_panel_host_id();
            let new_tab_stack_id = allocator.allocate_tab_stack_id();
            (
                WorkspaceMutation::MovePanelToNewSplitArea {
                    panel_id: panel_instance_id,
                    source_tab_stack_id,
                    target_tab_stack_id,
                    split_host_id,
                    target_child_host_id,
                    new_child_host_id,
                    new_tab_stack_id,
                    axis: side.axis(),
                    target_is_first_child: side.target_is_first_child(),
                    fraction: 0.5,
                },
                (),
            )
        }),
        TabDropDestination::SplitIntoHost {
            target_host_id,
            side,
        } => shell_state.apply_workspace_mutation_with_allocations(|allocator| {
            let split_host_id = allocator.allocate_panel_host_id();
            let new_child_host_id = allocator.allocate_panel_host_id();
            let new_tab_stack_id = allocator.allocate_tab_stack_id();
            (
                WorkspaceMutation::MovePanelToNewHostSplitArea {
                    panel_id: panel_instance_id,
                    source_tab_stack_id,
                    target_host_id,
                    split_host_id,
                    new_child_host_id,
                    new_tab_stack_id,
                    axis: side.axis(),
                    target_is_first_child: side.target_is_first_child(),
                    fraction: 0.5,
                },
                (),
            )
        }),
        TabDropDestination::SplitIntoRoot { side } => {
            let target_host_id = shell_state.workspace_state().root_host_id();
            shell_state.apply_workspace_mutation_with_allocations(|allocator| {
                let split_host_id = allocator.allocate_panel_host_id();
                let new_child_host_id = allocator.allocate_panel_host_id();
                let new_tab_stack_id = allocator.allocate_tab_stack_id();
                (
                    WorkspaceMutation::MovePanelToNewHostSplitArea {
                        panel_id: panel_instance_id,
                        source_tab_stack_id,
                        target_host_id,
                        split_host_id,
                        new_child_host_id,
                        new_tab_stack_id,
                        axis: side.axis(),
                        target_is_first_child: side.target_is_first_child(),
                        fraction: 0.5,
                    },
                    (),
                )
            })
        }
        TabDropDestination::NewFloatingHost => {
            let bounds = default_floating_host_bounds(shell_state);
            shell_state.apply_workspace_mutation_with_allocations(|allocator| {
                let floating_host_id = allocator.allocate_panel_host_id();
                let floating_tab_stack_id = allocator.allocate_tab_stack_id();
                (
                    WorkspaceMutation::MovePanelToNewFloatingHost {
                        panel_id: panel_instance_id,
                        source_tab_stack_id,
                        floating_host_id,
                        floating_tab_stack_id,
                        bounds,
                    },
                    (),
                )
            })
        }
    }
}

fn default_floating_host_bounds(shell_state: &RunenwerkEditorShellState) -> FloatingHostBounds {
    let bounds = shell_state
        .last_bounds()
        .unwrap_or(ui_math::UiRect::new(0.0, 0.0, 1280.0, 720.0));
    let width = (bounds.width * 0.46).clamp(360.0, 920.0);
    let height = (bounds.height * 0.42).clamp(240.0, 680.0);
    let x = (bounds.width - width).max(0.0) * 0.5;
    let y = (bounds.height - height).max(0.0) * 0.33;
    FloatingHostBounds::new(x, y, width, height)
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
            let profile_id =
                adjacent_workspace_profile(shell_state.active_workspace_profile_id(), 1).ok_or(
                    EditorMutationError::runtime_rejected("workspace profile missing"),
                )?;
            switch_workspace_profile(app, shell_state, profile_id)?;
        }
        ToolbarCommandKind::PreviousWorkspace => {
            let shell_state = shell_state.ok_or(EditorMutationError::runtime_rejected(
                "missing shell state for workspace command",
            ))?;
            let profile_id =
                adjacent_workspace_profile(shell_state.active_workspace_profile_id(), -1).ok_or(
                    EditorMutationError::runtime_rejected("workspace profile missing"),
                )?;
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
        ToolbarCommandKind::SaveSceneAs
        | ToolbarCommandKind::OpenRecent
        | ToolbarCommandKind::EditPreferences
        | ToolbarCommandKind::NewWindow
        | ToolbarCommandKind::LoadCustomWorkspace
        | ToolbarCommandKind::AddWorkspace => {
            app.append_console_line("[ui] command unavailable".to_string());
        }
    }
    Ok(())
}

fn adjacent_workspace_profile(
    active_profile_id: editor_shell::WorkspaceProfileId,
    delta: isize,
) -> Option<editor_shell::WorkspaceProfileId> {
    let registry = default_workspace_profile_registry();
    let profiles = registry
        .profiles()
        .map(|profile| profile.id)
        .collect::<Vec<_>>();
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
    let registry = default_workspace_profile_registry();
    let profile = registry
        .profile(profile_id)
        .ok_or(EditorMutationError::runtime_rejected(
            "workspace profile missing",
        ))?;
    let workspace_layout_path = default_workspace_layout_path_for_profile(profile_id);
    let workspace_state = if workspace_layout_path.exists() {
        let saved_workspace =
            read_workspace_layout_with_metadata(&workspace_layout_path).map_err(|_| {
                EditorMutationError::runtime_rejected("failed to load workspace layout")
            })?;
        if workspace_layout_matches_profile(&saved_workspace, profile) {
            saved_workspace.workspace_state
        } else {
            app.append_console_line(format!(
                "[workspace] ignored stale/incompatible saved {} workspace layout; rebuilt default",
                profile.label
            ));
            build_default_workspace_for_profile(app, profile)?
        }
    } else {
        build_default_workspace_for_profile(app, profile)?
    };
    shell_state.set_active_workspace_profile_id(profile_id);
    shell_state.replace_workspace_state(workspace_state);
    app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
    app.append_console_line(format!("[workspace] loaded {} workspace", profile.label));
    Ok(())
}

fn build_default_workspace_for_profile(
    app: &RunenwerkEditorApp,
    profile: &editor_shell::WorkspaceProfile,
) -> Result<editor_shell::WorkspaceState, EditorMutationError> {
    let mut allocator = editor_shell::WorkspaceIdentityAllocator::new();
    let workspace_id = allocator.allocate_workspace_id();
    profile
        .build_default_workspace_state_with_registry(
            workspace_id,
            &mut allocator,
            app.workbench_host().tool_surface_registry(),
        )
        .map_err(|_| {
            EditorMutationError::runtime_rejected(
                "workspace profile is incompatible with tool-surface registry",
            )
        })
}

fn workspace_layout_matches_profile(
    saved_workspace: &WorkspaceLayoutReadResult,
    profile: &editor_shell::WorkspaceProfile,
) -> bool {
    if saved_workspace
        .workspace_profile_id
        .is_some_and(|profile_id| profile_id != profile.id)
    {
        return false;
    }
    if !profile.required_tool_surfaces_are_present(&saved_workspace.workspace_state) {
        return false;
    }

    match (
        saved_workspace.layout_template.as_deref(),
        saved_workspace.layout_template_version,
    ) {
        (Some(template), Some(version)) => {
            template == profile.default_layout_template.contract_id()
                && version == profile.default_layout_template.contract_version()
        }
        _ => profile
            .default_layout_template
            .default_graph_matches(&saved_workspace.workspace_state),
    }
}

fn save_workspace_layout_for_active_profile(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    let workspace_layout_path =
        default_workspace_layout_path_for_profile(shell_state.active_workspace_profile_id());
    if let Some(parent) = workspace_layout_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create workspace layout folder")
        })?;
    }
    write_workspace_layout_for_profile(
        &workspace_layout_path,
        shell_state.workspace_state(),
        shell_state.active_workspace_profile_id(),
    )
    .map_err(|_| EditorMutationError::runtime_rejected("failed to save workspace layout"))?;
    app.append_console_line(format!(
        "[workspace] saved layout {}",
        workspace_layout_path.display()
    ));
    Ok(())
}

fn save_scene_to_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    let path = default_scene_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create editor scene folder")
        })?;
    }

    write_scene_file(&path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save editor scene"))?;
    let retained_path = retained_change_log_path_for_scene(&path);
    let workspace_layout_path =
        default_workspace_layout_path_for_profile(shell_state.active_workspace_profile_id());
    let entry_count = write_retained_change_log(&retained_path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save retained change log"))?;
    if let Some(parent) = workspace_layout_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create workspace layout folder")
        })?;
    }
    write_workspace_layout_for_profile(
        &workspace_layout_path,
        shell_state.workspace_state(),
        shell_state.active_workspace_profile_id(),
    )
    .map_err(|_| EditorMutationError::runtime_rejected("failed to save workspace layout"))?;
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
        "[io] saved workspace layout {}",
        workspace_layout_path.display()
    ));
    Ok(())
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
    let workspace_layout_path =
        default_workspace_layout_path_for_profile(shell_state.active_workspace_profile_id());
    let legacy_workspace_layout_path = legacy_workspace_layout_path_for_scene(&path);
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
    let layout_path_to_load = if workspace_layout_path.exists() {
        Some(workspace_layout_path.clone())
    } else if legacy_workspace_layout_path.exists() {
        Some(legacy_workspace_layout_path.clone())
    } else {
        None
    };

    if let Some(layout_path_to_load) = layout_path_to_load {
        match read_workspace_layout_with_metadata(&layout_path_to_load) {
            Ok(saved_workspace) => {
                let registry = default_workspace_profile_registry();
                let active_profile = registry.profile(shell_state.active_workspace_profile_id());
                if active_profile.is_some_and(|profile| {
                    workspace_layout_matches_profile(&saved_workspace, profile)
                }) {
                    shell_state.replace_workspace_state(saved_workspace.workspace_state);
                    app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
                    app.append_console_line(format!(
                        "[io] loaded workspace layout {}",
                        layout_path_to_load.display()
                    ));
                } else {
                    app.append_console_line(format!(
                        "[io] ignored workspace layout for inactive profile: {}",
                        layout_path_to_load.display()
                    ));
                }
            }
            Err(error) => {
                app.append_console_line(format!(
                    "[io] workspace layout load failed, keeping current layout: {} ({error})",
                    layout_path_to_load.display()
                ));
            }
        }
    } else {
        app.append_console_line(format!(
            "[io] workspace layout missing, keeping current layout: {}",
            workspace_layout_path.display()
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
mod tests {
    use super::*;
    use editor_shell::{
        MODELLING_WORKSPACE_PROFILE_ID, PanelKind, SCENE_WORKSPACE_PROFILE_ID,
        WorkspaceIdentityAllocator, reduce_workspace,
    };

    fn scene_profile() -> editor_shell::WorkspaceProfile {
        default_workspace_profile_registry()
            .profile(SCENE_WORKSPACE_PROFILE_ID)
            .expect("scene profile should exist")
            .clone()
    }

    fn modelling_profile() -> editor_shell::WorkspaceProfile {
        default_workspace_profile_registry()
            .profile(MODELLING_WORKSPACE_PROFILE_ID)
            .expect("modelling profile should exist")
            .clone()
    }

    fn default_scene_workspace() -> editor_shell::WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        scene_profile().build_default_workspace_state(workspace_id, &mut allocator)
    }

    fn workspace_surface_order(
        workspace: &editor_shell::WorkspaceState,
    ) -> Vec<editor_shell::ToolSurfaceKind> {
        workspace
            .tab_stacks()
            .flat_map(|stack| stack.ordered_panels.iter())
            .filter_map(|panel_id| workspace.panel(*panel_id))
            .filter_map(|panel| panel.active_tool_surface)
            .filter_map(|surface_id| workspace.tool_surface(surface_id))
            .filter_map(|surface| surface.legacy_tool_surface_kind())
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
            app.material_lab_runtime().active_source_document().is_none(),
            "missing source failure must not synthesize Material Lab source state"
        );
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
