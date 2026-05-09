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
        let mut lines = vec![
            "material inspector: formed-product descriptor surface".to_string(),
            surface_document_context_line(&request.document_context),
            "domain/material_graph owns parameters, source maps, cache keys, and specialization fragments".to_string(),
            "inspector routes are read-only until material commands exist".to_string(),
        ];
        lines.extend(context.app.asset_catalog_runtime().material_product_lines());
        lines.extend(
            context
                .app
                .asset_catalog_runtime()
                .import_diagnostic_lines(),
        );
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            Vec::new(),
        );

        Ok(ProviderSurfaceFrame {
            title: "Material Inspector".to_string(),
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
