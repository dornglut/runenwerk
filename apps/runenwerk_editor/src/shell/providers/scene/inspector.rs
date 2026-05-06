//! File: apps/runenwerk_editor/src/shell/providers/scene/inspector.rs
//! Purpose: Scene inspector provider.

use super::super::*;

pub struct SceneInspectorProvider;

impl EditorSurfaceProvider for SceneInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_INSPECTOR_PROVIDER_ID,
            "Scene Inspector",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::Inspector
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let panel_model = InspectorPanelPresenter::build_view_model(
            context.app.runtime(),
            &session.inspector_ui_state,
        );
        let frame = build_inspector_observation_frame(
            &panel_model,
            context.app.runtime().current_scene_reality_version(),
        );
        let view_model = build_inspector_view_model(&frame);
        let root = remap_surface_node_ids(
            build_inspector_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        for (index, field) in view_model.fields.iter().enumerate() {
            let action = if field.editable {
                SurfaceLocalAction::EditInspectorFieldText {
                    index,
                    text: String::new(),
                }
            } else {
                SurfaceLocalAction::ActivateInspectorField { index }
            };
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    inspector_field_widget_id(index),
                ),
                SurfaceLocalRoute::new(action),
            );
            if field.editable {
                routes.insert(
                    remap_widget_id(
                        request.tool_surface_instance_id,
                        inspector_field_focus_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::FocusInspectorField { index }),
                );
            }
        }
        Ok(ProviderSurfaceFrame {
            title: "Inspector".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let projection_epoch = context.projection_epoch;
        match action {
            SurfaceLocalAction::ActivateInspectorField { index } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::ActivateInspectorField { index },
                )))
            }
            SurfaceLocalAction::FocusInspectorField { index } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::FocusInspectorField { index },
                )))
            }
            SurfaceLocalAction::EditInspectorFieldText { index, text } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::AppendInspectorFieldText { index, text },
                )))
            }
            SurfaceLocalAction::BackspaceInspectorFieldText { index } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::BackspaceInspectorFieldText { index },
                )))
            }
            SurfaceLocalAction::CommitInspectorFieldText { index } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::CommitInspectorFieldText { index },
                )))
            }
            SurfaceLocalAction::CancelInspectorFieldText { index } => {
                Ok(Some(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::CancelInspectorFieldText { index },
                )))
            }
            _ => Ok(None),
        }
    }
}
