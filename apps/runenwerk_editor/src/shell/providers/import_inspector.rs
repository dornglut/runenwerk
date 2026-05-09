use super::*;

pub(super) struct ImportInspectorProvider;

impl EditorSurfaceProvider for ImportInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            IMPORT_INSPECTOR_PROVIDER_ID,
            "Import Inspector",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::ImportInspector
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let mut lines = context
            .app
            .asset_catalog_runtime()
            .import_diagnostic_lines();
        lines.push(format!(
            "dirty assets: {}",
            context.app.asset_catalog_runtime().dirty_assets().count()
        ));
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            Vec::new(),
        );
        Ok(ProviderSurfaceFrame {
            title: "Import Inspector".to_string(),
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
