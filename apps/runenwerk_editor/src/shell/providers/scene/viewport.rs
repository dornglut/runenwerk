//! File: apps/runenwerk_editor/src/shell/providers/scene/viewport.rs
//! Purpose: Scene viewport provider.

use super::super::*;

use crate::runtime::viewport::viewport_id_for_tool_surface;

pub struct SceneViewportProvider;

impl EditorSurfaceProvider for SceneViewportProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_VIEWPORT_PROVIDER_ID,
            "Scene Viewport",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::Viewport
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let bound_products = context
            .tool_surface_bindings
            .and_then(|bindings| {
                bindings.binding_for_tool_surface(request.tool_surface_instance_id)
            })
            .and_then(|binding| {
                context
                    .viewport_observations
                    .and_then(|observations| observations.frame_for(binding.viewport_id))
            });
        let expected_viewport_id = viewport_id_for_tool_surface(request.tool_surface_instance_id);
        let products = bound_products.or_else(|| {
            context
                .viewport_observations
                .and_then(|observations| observations.frame_for(expected_viewport_id))
        });
        let tool_state = context.app.viewport_tool_state();
        let frame = build_viewport_observation_frame(
            products,
            session.viewport_details_visible,
            session.viewport_statistics_visible,
            session.viewport_options_menu_open,
            context.app.runtime().selected_entity(),
            session.viewport_interaction_state.drag_in_progress(),
            tool_state,
            context.app.runtime().current_scene_reality_version(),
            Some(expected_viewport_id),
        );
        let view_model = build_viewport_view_model(&frame);
        let root = remap_surface_node_ids(
            build_viewport_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::ToggleViewportDetails),
        );
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::ToggleViewportStatistics),
        );
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::ToggleViewportOptionsMenu),
        );
        for (index, choice) in view_model.product_choices.iter().enumerate() {
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    viewport_product_button_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::SelectViewportProduct {
                    viewport_id: choice.viewport_id,
                    product_id: choice.product_id,
                    enabled: choice.enabled,
                }),
            );
        }
        Ok(ProviderSurfaceFrame {
            title: "Viewport".to_string(),
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
        match action {
            SurfaceLocalAction::SelectViewportProduct {
                viewport_id,
                product_id,
                enabled,
            } if enabled => Ok(Some(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::SelectViewportProduct {
                    viewport_id,
                    product_id,
                },
            ))),
            SurfaceLocalAction::ToggleViewportDetails => Ok(Some(surface_session_proposal(
                request,
                context.projection_epoch,
                SurfaceSessionMutation::ToggleViewportDetails,
            ))),
            SurfaceLocalAction::ToggleViewportStatistics => Ok(Some(surface_session_proposal(
                request,
                context.projection_epoch,
                SurfaceSessionMutation::ToggleViewportStatistics,
            ))),
            SurfaceLocalAction::ToggleViewportOptionsMenu => Ok(Some(surface_session_proposal(
                request,
                context.projection_epoch,
                SurfaceSessionMutation::ToggleViewportOptionsMenu,
            ))),
            _ => Ok(None),
        }
    }
}
