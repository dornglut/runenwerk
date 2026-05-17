use super::*;

pub(super) struct MaterialPreviewProvider;

impl EditorSurfaceProvider for MaterialPreviewProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            MATERIAL_PREVIEW_PROVIDER_ID,
            "Material Preview",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::MaterialPreview
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model = context.app.material_lab_runtime().preview_view_model();
        let mut lines = vec![
            "material preview: catalog-backed product and prepared renderer handoff".to_string(),
            surface_document_context_line(&request.document_context),
            "preview output fails closed when no formed material product is available".to_string(),
        ];
        if let Some(product_id) = view_model.active_product_id {
            lines.push(format!("active material product: {}", product_id.raw()));
        }
        if let Some(artifact_id) = view_model.artifact_id {
            lines.push(format!("artifact: {}", artifact_id.raw()));
        }
        if let Some(viewport_product_id) = view_model.viewport_product_id {
            lines.push(format!("viewport product: {}", viewport_product_id.0));
        }
        if let Some(fragment) = &view_model.specialization_fragment {
            lines.push(format!("specialization: {fragment}"));
        }
        lines.push(format!(
            "prepared parameter payload bytes: {}",
            view_model.prepared_parameter_payload_bytes
        ));
        lines.extend(view_model.preview_status_lines.clone());
        lines.extend(view_model.diagnostic_lines.clone());
        lines.extend(crate::material_lab::material_artifact_lines(
            context.app.asset_catalog_runtime().catalog(),
        ));
        lines.extend(context.app.asset_catalog_runtime().texture_product_lines());
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
            title: "Material Preview".to_string(),
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
