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
        let mut lines = vec![
            "material preview: descriptor-first adapter boundary".to_string(),
            surface_document_context_line(&request.document_context),
            "preview targets: sdf_sphere, sdf_box, plane, formed_field_product".to_string(),
            "render adapter: not registered; P3 owns GPU/render-expression handoff".to_string(),
            "preview output fails closed when no formed material product is available".to_string(),
        ];
        lines.extend(context.app.asset_catalog_runtime().material_product_lines());
        lines.extend(context.app.asset_catalog_runtime().texture_product_lines());
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            Vec::new(),
        );

        Ok(ProviderSurfaceFrame {
            title: "Material Preview".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        Ok(None)
    }
}
