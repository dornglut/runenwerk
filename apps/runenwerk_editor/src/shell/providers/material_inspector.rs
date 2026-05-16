use super::*;

pub(super) struct MaterialInspectorProvider;

impl EditorSurfaceProvider for MaterialInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            MATERIAL_INSPECTOR_PROVIDER_ID,
            "Material Inspector",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::MaterialInspector
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model = context.app.material_lab_runtime().inspector_view_model();
        let mut lines = vec![
            "material inspector: formed material product surface".to_string(),
            surface_document_context_line(&request.document_context),
            "domain/material_graph owns parameters, source maps, cache keys, and specialization fragments".to_string(),
            format!(
                "selected material asset: {:?}",
                view_model.selected_asset_id.map(|asset_id| asset_id.raw())
            ),
        ];
        if let Some(product_id) = view_model.active_product_id {
            lines.push(format!("active material product: {}", product_id.raw()));
        }
        if let Some(artifact_id) = view_model.artifact_id {
            lines.push(format!("artifact: {}", artifact_id.raw()));
        }
        if let Some(output_target) = view_model.output_target {
            lines.push(format!("output target: {output_target:?}"));
        }
        lines.extend(view_model.parameter_lines.clone());
        lines.extend(view_model.source_map_lines.clone());
        lines.extend(view_model.diagnostic_lines.clone());
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        let mut actions = vec![(
            "Clear material diagnostics".to_string(),
            SurfaceLocalAction::Material(MaterialSurfaceAction::ClearMaterialDiagnostics),
        )];
        if view_model.selected_asset_id.is_some() {
            actions.push((
                "Build selected preview".to_string(),
                SurfaceLocalAction::Material(MaterialSurfaceAction::BuildSelectedMaterialPreview),
            ));
        }

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );

        Ok(ProviderSurfaceFrame {
            title: "Material Inspector".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let SurfaceLocalAction::Material(action) = action else {
            return Ok(None);
        };
        Ok(
            super::material_graph_canvas::material_surface_action_command(
                action,
                context.projection_epoch,
            )
            .map(SurfaceCommandProposal::Shell),
        )
    }
}
