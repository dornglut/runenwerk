use super::*;

pub(super) struct SdfBrushBrowserProvider;

impl EditorSurfaceProvider for SdfBrushBrowserProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SDF_BRUSH_BROWSER_PROVIDER_ID,
            "SDF Brush Browser",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_or_legacy_kind_support(
            request,
            SDF_BRUSH_BROWSER_SURFACE_KEY,
            ToolSurfaceKind::SdfBrushBrowser,
        )
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            context.app.asset_catalog_runtime().sdf_brush_lines(),
            Vec::new(),
        );
        Ok(ProviderSurfaceFrame {
            title: "SDF Brushes".to_string(),
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
