use super::*;

pub(super) struct TextureViewerProvider;

impl EditorSurfaceProvider for TextureViewerProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            TEXTURE_VIEWER_PROVIDER_ID,
            "Texture Viewer",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::TextureViewer
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let mut lines = vec![
            "texture viewer: descriptor-first provider".to_string(),
            surface_document_context_line(&request.document_context),
            "domain/texture owns sampler, color-space, compression, mip, and channel metadata"
                .to_string(),
        ];
        lines.extend(texture_preview_lines(
            context
                .app
                .asset_catalog_runtime()
                .texture_preview_descriptor(),
        ));
        lines.extend(context.app.asset_catalog_runtime().texture_product_lines());
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
            title: "Texture Viewer".to_string(),
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

pub(super) fn texture_preview_lines(
    descriptor: Option<texture::TexturePreviewDescriptor>,
) -> Vec<String> {
    match descriptor {
        Some(descriptor) => vec![format!(
            "preview descriptor: product={} mip={} slice={} channel={:?} color_space={:?}",
            descriptor.product_id.raw(),
            descriptor.mip_level,
            descriptor.slice_index,
            descriptor.channel,
            descriptor.color_space_override
        )],
        None => vec![
            "preview descriptor: unavailable until a typed formed texture product is selected"
                .to_string(),
        ],
    }
}
