use material_graph::MaterialNodeCatalog;

use super::*;

pub(super) struct MaterialGraphCanvasProvider;

impl EditorSurfaceProvider for MaterialGraphCanvasProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
            "Material Graph Canvas",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::MaterialGraphCanvas
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let catalog = MaterialNodeCatalog::first_slice();
        let catalog_sample = catalog
            .descriptors()
            .take(8)
            .map(|node| node.key.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let mut lines = vec![
            "material graph canvas: descriptor-first provider".to_string(),
            surface_document_context_line(&request.document_context),
            format!("catalog nodes: {}", catalog.len()),
            format!("catalog sample: {catalog_sample}"),
            "canvas state is not material truth; authored graph documents must ratify in domain/material_graph".to_string(),
            "ratification: waiting for source-backed MaterialGraphDocument content".to_string(),
        ];
        lines.extend(
            context
                .app
                .asset_catalog_runtime()
                .material_graph_asset_lines(),
        );
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
            title: "Material Graph Canvas".to_string(),
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
