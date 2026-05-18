//! Runtime producer for catalog-backed texture preview product surfaces.

use editor_shell::TextureViewerSurfaceKind;
use engine::plugins::render::{
    RenderDynamicTextureTargetRequestRegistryResource, RenderDynamicTextureUploadRegistryResource,
    RenderFrameProducerId,
};
use engine::runtime::{Res, ResMut};

use crate::runtime::resources::EditorHostResource;
use crate::texture_preview::prepare_texture_preview;

pub const EDITOR_TEXTURE_PREVIEW_PRODUCER_ID: RenderFrameProducerId =
    match RenderFrameProducerId::try_from_raw(2_028) {
        Ok(id) => id,
        Err(_) => panic!("texture preview producer id must be non-zero"),
    };

pub fn produce_texture_preview_dynamic_uploads_system(
    host: Res<EditorHostResource>,
    mut dynamic_target_requests: ResMut<RenderDynamicTextureTargetRequestRegistryResource>,
    mut texture_uploads: ResMut<RenderDynamicTextureUploadRegistryResource>,
) {
    let mut targets = Vec::new();
    let mut uploads = Vec::new();
    for surface in [
        TextureViewerSurfaceKind::Texture2D,
        TextureViewerSurfaceKind::VolumeTexture3D,
    ] {
        if let Ok(preview) = prepare_texture_preview(
            host.app.asset_catalog_runtime().catalog(),
            host.app.asset_catalog_runtime().selected_asset_id(),
            host.app.texture_preview_runtime(),
            surface,
        ) {
            targets.push(preview.target);
            uploads.push(preview.upload);
        }
    }

    if targets.is_empty() {
        let _ = dynamic_target_requests.remove_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID);
        let _ = texture_uploads.remove_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID);
        return;
    }

    dynamic_target_requests
        .replace_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID, targets)
        .expect("editor texture preview dynamic target contribution must be valid and unique");
    texture_uploads
        .replace_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID, uploads)
        .expect("editor texture preview upload contribution must be valid and unique");
}
