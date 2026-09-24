//! Runtime producer for catalog-backed texture preview product surfaces.

use editor_shell::TextureViewerSurfaceKind;
use engine::plugins::render::{
    RenderDynamicTextureTargetRequestRegistryResource, RenderDynamicTextureUploadRegistryResource,
    RenderFrameProducerId, RenderProductSurfaceManifest,
};
use engine::runtime::{Res, ResMut};
use ui_render_data::ProductSurfaceTextureBindingSource;

use crate::runtime::resources::EditorHostResource;
use crate::texture_preview::{TexturePreviewPreparedUpload, prepare_texture_preview};

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
    let mut previews = Vec::new();
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
            previews.push(preview);
        }
    }

    if previews.is_empty() {
        let _ = dynamic_target_requests.remove_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID);
        let _ = texture_uploads.remove_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID);
        return;
    }
    let manifest = texture_preview_product_surface_manifest(previews);
    debug_assert!(
        !manifest.has_error_diagnostics(),
        "texture preview product-surface manifest should be structurally valid"
    );
    let (targets, uploads, _, _) = manifest.into_render_parts();

    dynamic_target_requests
        .replace_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID, targets)
        .expect("editor texture preview dynamic target contribution must be valid and unique");
    texture_uploads
        .replace_contribution(EDITOR_TEXTURE_PREVIEW_PRODUCER_ID, uploads)
        .expect("editor texture preview upload contribution must be valid and unique");
}

pub(crate) fn texture_preview_product_surface_manifest(
    previews: impl IntoIterator<Item = TexturePreviewPreparedUpload>,
) -> RenderProductSurfaceManifest {
    let mut manifest = RenderProductSurfaceManifest::new(
        EDITOR_TEXTURE_PREVIEW_PRODUCER_ID,
        "editor.texture_preview",
    );
    for preview in previews {
        let target_key = preview.proof.target_key.clone();
        manifest = manifest
            .with_dynamic_target(preview.target)
            .with_dynamic_upload(preview.upload)
            .with_upload_backed_product_surface_binding(
                target_key.to_string(),
                ProductSurfaceTextureBindingSource::dynamic_texture(
                    target_key.namespace,
                    target_key.target_id,
                ),
            );
    }
    manifest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture_preview::TexturePreviewProofMetadata;
    use engine::plugins::render::{
        RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
        RenderDynamicTextureTargetKey, RenderDynamicTextureUploadDescriptor,
        RenderTextureSampleMode, RenderTextureTargetFormat, RenderTextureUploadAlphaMode,
    };

    fn prepared_upload(target_id: &str) -> TexturePreviewPreparedUpload {
        let target_key = RenderDynamicTextureTargetKey::new("editor.texture_preview", target_id);
        TexturePreviewPreparedUpload {
            target: RenderDynamicTextureTargetDescriptor::color_sampled(
                target_key.clone(),
                2,
                2,
                RenderTextureTargetFormat::Rgba8Unorm,
                RenderTextureSampleMode::FilterableFloat,
                RenderDynamicTextureRetention::RetainWhileRequested,
            ),
            upload: RenderDynamicTextureUploadDescriptor::rgba8(
                target_key.clone(),
                0,
                0,
                2,
                2,
                RenderTextureUploadAlphaMode::Straight,
                1,
                vec![255; 16],
            ),
            proof: TexturePreviewProofMetadata {
                texture_product_id: 1,
                descriptor_hash: "hash".to_string(),
                artifact_uri: "artifact://texture".to_string(),
                upload_format: format!("{:?}", RenderTextureTargetFormat::Rgba8Unorm),
                mip_count: 1,
                selected_mip: 0,
                selected_slice: 0,
                selected_channel: "all".to_string(),
                sampler_identity: "sampler".to_string(),
                bind_group_identity: "bind_group".to_string(),
                residency_state: "Resident".to_string(),
                residency_class: "test".to_string(),
                target_key,
            },
        }
    }

    #[test]
    fn texture_preview_product_surface_manifest_traces_upload_backed_surface() {
        let manifest = texture_preview_product_surface_manifest([prepared_upload("texture.2d")]);

        assert_eq!(manifest.product_family(), "editor.texture_preview");
        assert_eq!(manifest.dynamic_targets().len(), 1);
        assert_eq!(manifest.dynamic_uploads().len(), 1);
        assert_eq!(manifest.product_bindings().len(), 1);
        assert!(manifest.product_bindings()[0].upload_required);
        assert!(
            manifest.diagnostics().is_empty(),
            "texture preview manifest should declare target, upload, and UI binding"
        );
    }
}
