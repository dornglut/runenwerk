//! File: apps/runenwerk_editor/src/shell/dispatch/sdf_operations.rs
//! Purpose: Dispatch SDF operation surface mutations through app-held domain state.

use editor_core::EditorMutationError;
use editor_shell::{
    SdfOperationDomainMutation, SdfOperationSessionMutation, StructuralCommandTarget,
    ToolSurfaceKind,
};
use ui_surface::SurfaceCapability;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::RunenwerkEditorShellState;
use crate::shell::dispatch::resolve_legacy_surface_command_contract;

pub(crate) fn dispatch_session_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: SdfOperationSessionMutation,
) -> Result<(), EditorMutationError> {
    ensure_sdf_surface(
        shell_state,
        target,
        &[SurfaceCapability::Observe, SurfaceCapability::Interact],
    )?;
    match mutation {
        SdfOperationSessionMutation::SelectLayer { layer_id } => {
            app.sdf_operation_workspace_mut().select_layer(layer_id)
        }
        SdfOperationSessionMutation::SelectOperation { operation_id } => app
            .sdf_operation_workspace_mut()
            .select_operation(operation_id),
    }
}

pub(crate) fn dispatch_domain_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: SdfOperationDomainMutation,
) -> Result<(), EditorMutationError> {
    ensure_sdf_surface(
        shell_state,
        target,
        &[
            SurfaceCapability::Observe,
            SurfaceCapability::Interact,
            SurfaceCapability::RequestMutation,
        ],
    )?;
    match mutation {
        SdfOperationDomainMutation::ApplyCommand { intent } => {
            app.sdf_operation_workspace_mut().apply_command(intent)?;
            Ok(())
        }
        SdfOperationDomainMutation::ApplyGraphCommand { intent } => {
            app.sdf_operation_workspace_mut()
                .apply_graph_command(intent)?;
            Ok(())
        }
        SdfOperationDomainMutation::LowerGraphToOperationDocument => {
            app.sdf_operation_workspace_mut()
                .lower_graph_to_operation_document()?;
            Ok(())
        }
        SdfOperationDomainMutation::CommitOperationWindow => {
            let (candidate, selected_product) = {
                let workspace = app.sdf_operation_workspace_mut();
                let candidate = workspace.commit_operation_window()?;
                let selected_product = workspace
                    .selected_field_preview_product()
                    .map(|product| product.descriptor.clone());
                (candidate, selected_product)
            };
            app.asset_catalog_runtime_mut()
                .set_selected_field_product(selected_product);
            app.append_console_line(format!(
                "[sdf] committed operation window ops={} chunks={} previews={}",
                candidate.operation_count(),
                candidate.touched_chunks.len(),
                app.sdf_operation_workspace().field_preview_products().len()
            ));
            Ok(())
        }
    }
}

fn ensure_sdf_surface(
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    required_capabilities: &[SurfaceCapability],
) -> Result<(), EditorMutationError> {
    let Some(contract) = resolve_legacy_surface_command_contract(
        shell_state,
        target,
        ToolSurfaceKind::FieldLayerStack,
    ) else {
        return Err(EditorMutationError::session_rejected(
            "missing SDF operation surface command target",
        ));
    };
    if matches!(
        contract.tool_surface_kind,
        ToolSurfaceKind::FieldLayerStack | ToolSurfaceKind::SdfGraphCanvas
    ) {
    } else {
        return Err(EditorMutationError::session_rejected(
            "SDF operation mutation targeted a non-SDF surface",
        ));
    }
    for required_capability in required_capabilities {
        if !contract.capabilities.allows(*required_capability) {
            return Err(EditorMutationError::session_rejected(
                "SDF operation mutation missing required surface capability",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_scene::{SdfBooleanIntent, SdfPrimitiveKind, SdfPrimitiveSpec};
    use editor_shell::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId};

    fn target() -> StructuralCommandTarget {
        StructuralCommandTarget {
            panel_instance_id: PanelInstanceId::try_from_raw(1).unwrap(),
            active_tool_surface: Some(ToolSurfaceInstanceId::try_from_raw(2).unwrap()),
            tab_stack_id: TabStackId::try_from_raw(3).unwrap(),
        }
    }

    fn target_for_surface_kind(
        shell_state: &RunenwerkEditorShellState,
        kind: ToolSurfaceKind,
    ) -> StructuralCommandTarget {
        let (panel_instance_id, tool_surface_id) = shell_state
            .workspace_state()
            .panels()
            .filter_map(|panel| {
                let tool_surface_id = panel.active_tool_surface?;
                let surface = shell_state
                    .workspace_state()
                    .tool_surface(tool_surface_id)?;
                (surface.legacy_tool_surface_kind() == Some(kind))
                    .then_some((panel.id, tool_surface_id))
            })
            .next()
            .expect("default shell state should contain requested surface kind");

        StructuralCommandTarget {
            panel_instance_id,
            active_tool_surface: Some(tool_surface_id),
            tab_stack_id: TabStackId::try_from_raw(1).unwrap(),
        }
    }

    #[test]
    fn sdf_operation_domain_dispatch_applies_command_to_app_state() {
        let mut app = RunenwerkEditorApp::new();
        let layer_id = app.sdf_operation_workspace().document().layers()[0].id;

        dispatch_domain_mutation(
            &mut app,
            None,
            target(),
            SdfOperationDomainMutation::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Sphere Add".to_string(),
                    primitive: SdfPrimitiveSpec::new(
                        SdfPrimitiveKind::Sphere,
                        SdfBooleanIntent::Add,
                    ),
                    material_channel: 0,
                },
            },
        )
        .expect("SDF domain mutation should dispatch");

        assert_eq!(
            app.sdf_operation_workspace().document().layers()[0]
                .operations
                .len(),
            1
        );
    }

    #[test]
    fn sdf_operation_session_dispatch_selects_layer() {
        let mut app = RunenwerkEditorApp::new();
        let layer_id = app.sdf_operation_workspace().document().layers()[0].id;

        dispatch_session_mutation(
            &mut app,
            None,
            target(),
            SdfOperationSessionMutation::SelectLayer { layer_id },
        )
        .expect("SDF session mutation should dispatch");

        assert_eq!(
            app.sdf_operation_workspace().selected_layer_id(),
            Some(layer_id)
        );
    }

    #[test]
    fn sdf_operation_commit_appends_log_dirty_chunks_and_preview_products() {
        let mut app = RunenwerkEditorApp::new();
        let layer_id = app.sdf_operation_workspace().document().layers()[0].id;
        dispatch_domain_mutation(
            &mut app,
            None,
            target(),
            SdfOperationDomainMutation::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Sphere Add".to_string(),
                    primitive: SdfPrimitiveSpec::new(
                        SdfPrimitiveKind::Sphere,
                        SdfBooleanIntent::Add,
                    ),
                    material_channel: 1,
                },
            },
        )
        .expect("add SDF operation");

        dispatch_domain_mutation(
            &mut app,
            None,
            target(),
            SdfOperationDomainMutation::CommitOperationWindow,
        )
        .expect("commit SDF operation window");

        assert_eq!(
            app.sdf_operation_workspace()
                .committed_operation_log()
                .operations
                .len(),
            1
        );
        assert!(
            !app.sdf_operation_workspace()
                .dirty_chunks()
                .by_chunk
                .is_empty()
        );
        assert!(
            !app.sdf_operation_workspace()
                .field_preview_products()
                .is_empty()
        );
        assert_eq!(
            app.sdf_operation_workspace().field_preview_products().len() % 4,
            0
        );
        assert!(
            app.asset_catalog_runtime()
                .selected_field_product()
                .is_some()
        );
    }

    #[test]
    fn sdf_operation_dispatch_rejects_non_sdf_surface_target() {
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let console_target = target_for_surface_kind(&shell_state, ToolSurfaceKind::Console);
        let layer_id = app.sdf_operation_workspace().document().layers()[0].id;

        let error = dispatch_session_mutation(
            &mut app,
            Some(&shell_state),
            console_target,
            SdfOperationSessionMutation::SelectLayer { layer_id },
        )
        .expect_err("SDF dispatch must reject non-SDF surface targets");

        assert_eq!(
            error.message,
            "SDF operation mutation targeted a non-SDF surface",
        );
    }
}
