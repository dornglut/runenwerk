use super::*;

pub(super) struct FieldProductViewerProvider;

impl EditorSurfaceProvider for FieldProductViewerProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            FIELD_PRODUCT_VIEWER_PROVIDER_ID,
            "Field Product Viewer",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_support(request, FIELD_PRODUCT_VIEWER_SURFACE_KEY)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let mut lines = context.app.asset_catalog_runtime().field_product_lines();
        lines.extend(context.app.sdf_operation_workspace().field_preview_lines());
        lines.extend(context.app.procgen_runtime().field_preview_lines());
        if context
            .app
            .asset_catalog_runtime()
            .has_stale_field_product()
        {
            lines.push("selected field product is potentially stale".to_string());
        }
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            Vec::new(),
        );
        Ok(ProviderSurfaceFrame {
            title: "Field Products".to_string(),
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
