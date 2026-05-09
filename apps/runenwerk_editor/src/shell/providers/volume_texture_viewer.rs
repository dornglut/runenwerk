use super::*;

pub(super) struct VolumeTextureViewerProvider;

impl EditorSurfaceProvider for VolumeTextureViewerProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            VOLUME_TEXTURE_VIEWER_PROVIDER_ID,
            "Volume Texture Viewer",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::VolumeTextureViewer
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let mut lines = vec![
            "volume texture viewer: descriptor-first provider".to_string(),
            surface_document_context_line(&request.document_context),
            "Texture3D preview is slice/mip/channel inspection; GPU upload remains adapter-owned"
                .to_string(),
        ];
        lines.extend(super::texture_viewer::texture_preview_lines(
            context
                .app
                .asset_catalog_runtime()
                .volume_texture_preview_descriptor(),
        ));
        lines.extend(
            context
                .app
                .asset_catalog_runtime()
                .volume_texture_product_lines(),
        );
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
            title: "Volume Texture Viewer".to_string(),
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
