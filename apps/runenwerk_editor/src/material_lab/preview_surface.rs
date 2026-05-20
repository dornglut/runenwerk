//! Material Preview provider product-surface target contracts.

use editor_shell::MaterialPreviewProductSurfaceViewModel;
use engine::plugins::render::{
    RenderDynamicTextureRetention, RenderDynamicTextureTargetDescriptor,
    RenderDynamicTextureTargetKey, RenderTextureSampleMode, RenderTextureTargetFormat,
    RenderTextureTargetUsage,
};
use ui_render_data::ProductSurfaceTextureBindingSource;

use crate::material_lab::state::EditorMaterialPreviewProduct;

pub const MATERIAL_PREVIEW_SCENE_SURFACE_NAMESPACE: &str =
    "runenwerk.editor.material_lab.preview_scene";
pub const MATERIAL_PREVIEW_SCENE_SURFACE_WIDTH: u32 = 512;
pub const MATERIAL_PREVIEW_SCENE_SURFACE_HEIGHT: u32 = 512;

pub fn material_preview_scene_surface_target_key(
    preview: &EditorMaterialPreviewProduct,
) -> RenderDynamicTextureTargetKey {
    RenderDynamicTextureTargetKey::new(
        MATERIAL_PREVIEW_SCENE_SURFACE_NAMESPACE,
        format!("material-product-{}-scene", preview.product_id().raw()),
    )
}

pub fn material_preview_scene_surface_descriptor(
    preview: &EditorMaterialPreviewProduct,
) -> RenderDynamicTextureTargetDescriptor {
    RenderDynamicTextureTargetDescriptor::new(
        material_preview_scene_surface_target_key(preview),
        MATERIAL_PREVIEW_SCENE_SURFACE_WIDTH,
        MATERIAL_PREVIEW_SCENE_SURFACE_HEIGHT,
        RenderTextureTargetFormat::Rgba8Unorm,
        RenderTextureTargetUsage::color_sampled(),
        RenderTextureSampleMode::FilterableFloat,
        RenderDynamicTextureRetention::RetainWhileRequested,
    )
}

pub fn material_preview_scene_surface_view_model(
    preview: &EditorMaterialPreviewProduct,
) -> MaterialPreviewProductSurfaceViewModel {
    let target_key = material_preview_scene_surface_target_key(preview);
    MaterialPreviewProductSurfaceViewModel {
        source: ProductSurfaceTextureBindingSource::dynamic_texture(
            target_key.namespace.clone(),
            target_key.target_id.clone(),
        ),
        width: MATERIAL_PREVIEW_SCENE_SURFACE_WIDTH,
        height: MATERIAL_PREVIEW_SCENE_SURFACE_HEIGHT,
        target_label: target_key.label(),
        bind_group_identity: format!(
            "engine_ui_product_surface_bind_group:{}",
            target_key.label()
        ),
    }
}
