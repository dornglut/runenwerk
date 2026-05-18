use super::*;
use crate::texture_preview::texture_preview_view_model;

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
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_or_legacy_kind_support(
            request,
            TEXTURE_VIEWER_3D_SURFACE_KEY,
            ToolSurfaceKind::VolumeTextureViewer,
        )
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let controls = context
            .app
            .texture_preview_runtime()
            .controls(TextureViewerSurfaceKind::VolumeTexture3D);
        let view_model = texture_preview_view_model(
            context.app.asset_catalog_runtime().catalog(),
            context.app.asset_catalog_runtime().selected_asset_id(),
            context.app.texture_preview_runtime(),
            TextureViewerSurfaceKind::VolumeTexture3D,
        );
        let (root, routes) = super::texture_viewer::build_texture_preview_panel(
            context.theme,
            request,
            &view_model,
            vec![
                (
                    "Slice -".to_string(),
                    TextureSurfaceAction::SetPreviewSlice {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        slice_index: controls.slice_index.saturating_sub(1),
                    },
                ),
                (
                    "Slice +".to_string(),
                    TextureSurfaceAction::SetPreviewSlice {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        slice_index: controls.slice_index.saturating_add(1),
                    },
                ),
                (
                    "Mip -".to_string(),
                    TextureSurfaceAction::SetPreviewMip {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        mip_level: controls.mip_level.saturating_sub(1),
                    },
                ),
                (
                    "Mip +".to_string(),
                    TextureSurfaceAction::SetPreviewMip {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        mip_level: controls.mip_level.saturating_add(1),
                    },
                ),
                (
                    "Mip 0".to_string(),
                    TextureSurfaceAction::SetPreviewMip {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        mip_level: 0,
                    },
                ),
                (
                    "All".to_string(),
                    TextureSurfaceAction::SetPreviewChannel {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        channel: TexturePreviewChannelSelection::All,
                    },
                ),
                (
                    "R".to_string(),
                    TextureSurfaceAction::SetPreviewChannel {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        channel: TexturePreviewChannelSelection::R,
                    },
                ),
                (
                    "G".to_string(),
                    TextureSurfaceAction::SetPreviewChannel {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        channel: TexturePreviewChannelSelection::G,
                    },
                ),
                (
                    "B".to_string(),
                    TextureSurfaceAction::SetPreviewChannel {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        channel: TexturePreviewChannelSelection::B,
                    },
                ),
                (
                    "A".to_string(),
                    TextureSurfaceAction::SetPreviewChannel {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                        channel: TexturePreviewChannelSelection::A,
                    },
                ),
                (
                    "Reset".to_string(),
                    TextureSurfaceAction::ResetPreview {
                        surface: TextureViewerSurfaceKind::VolumeTexture3D,
                    },
                ),
            ],
        );

        Ok(ProviderSurfaceFrame {
            title: "Volume Texture Viewer".to_string(),
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
        super::texture_viewer::texture_surface_action_command(action, context.projection_epoch)
    }
}
