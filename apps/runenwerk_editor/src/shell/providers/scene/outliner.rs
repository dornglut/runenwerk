//! File: apps/runenwerk_editor/src/shell/providers/scene/outliner.rs
//! Purpose: Scene outliner provider.

use super::super::*;

pub struct SceneOutlinerProvider;

impl EditorSurfaceProvider for SceneOutlinerProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_OUTLINER_PROVIDER_ID,
            "Scene Outliner",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        if !matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) {
            return SurfaceProviderSupportMode::Unsupported;
        }
        stable_key_or_legacy_kind_support(
            request,
            SCENE_OUTLINER_SURFACE_KEY,
            ToolSurfaceKind::Outliner,
        )
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let state = context.app.outliner_state();
        let frame = build_outliner_observation_frame(
            &state,
            context.app.runtime().current_scene_reality_version(),
        );
        let view_model = build_outliner_view_model(&frame);
        let root = build_outliner_panel(
            &view_model,
            context.theme,
            request.panel_instance_id,
            Some(request.tool_surface_instance_id),
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            surface_widget_id(request.tool_surface_instance_id, OUTLINER_LIST_WIDGET_ID),
            SurfaceLocalRoute::new(SurfaceLocalAction::Outliner(
                OutlinerSurfaceAction::SelectRow {
                    entities: view_model.rows.iter().map(|row| row.entity).collect(),
                },
            )),
        );
        Ok(ProviderSurfaceFrame {
            title: "Outliner".to_string(),
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
            SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity { entity }) => {
                Ok(Some(editor_domain_proposal(
                    request,
                    context.projection_epoch,
                    EditorDomainMutation::Outliner(OutlinerDomainMutation::SelectEntity { entity }),
                )))
            }
            _ => Ok(None),
        }
    }
}
