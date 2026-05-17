//! File: apps/runenwerk_editor/src/runtime/systems/material_preview.rs
//! Purpose: Material Lab prepared renderer handoff and preview render producer.

use std::collections::BTreeMap;

use editor_viewport::ViewportSurfacePresentationSlot;
use engine::plugins::render::{
    FeatureContributionStatus, PreparedFlowInvocationId, PreparedFlowInvocationRequest,
    PreparedMaterialFeatureResource, PreparedRenderFrameRequestResource, PreparedTargetBinding,
    PreparedViewFrame, RenderFlowId, RenderFlowRegistryResource, ShaderRegistryResource,
};
use engine::runtime::{Res, ResMut};

use crate::material_lab::{EditorMaterialPreviewProduct, prepared_material_resource_for_preview};
use crate::runtime::app::{EDITOR_MATERIAL_PREVIEW_FLOW_ID, EDITOR_MATERIAL_PREVIEW_SHADER_ID};
use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::{
    EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID, VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW,
    ViewportProductTargetRegistryResource, ViewportProductTargetStatus,
};

pub fn prepare_material_preview_render_resource_system(
    mut host: ResMut<EditorHostResource>,
    mut material_feature: ResMut<PreparedMaterialFeatureResource>,
    mut shader_registry: ResMut<ShaderRegistryResource>,
) {
    let preview = host.app.material_lab_runtime().active_preview().cloned();
    if let Some(diagnostic) = prepare_material_preview_render_resource(
        preview.as_ref(),
        &mut material_feature,
        &mut shader_registry,
    ) {
        host.app
            .material_lab_runtime_mut()
            .record_diagnostic(diagnostic);
    }
}

pub(crate) fn prepare_material_preview_render_resource(
    preview: Option<&EditorMaterialPreviewProduct>,
    material_feature: &mut PreparedMaterialFeatureResource,
    shader_registry: &mut ShaderRegistryResource,
) -> Option<asset::AssetDiagnosticRecord> {
    if let Some(preview) = preview {
        shader_registry.register_shader_with_id(
            EDITOR_MATERIAL_PREVIEW_SHADER_ID,
            preview.shader_path.clone(),
        );
        shader_registry.register_shader_with_id(
            preview.scene_shader_path.clone(),
            preview.scene_shader_path.clone(),
        );
        if shader_registry.is_loaded(EDITOR_MATERIAL_PREVIEW_SHADER_ID)
            && shader_registry.is_loaded(preview.scene_shader_path.as_str())
        {
            match prepared_material_resource_for_preview(Some(preview)) {
                Ok(resource) => {
                    *material_feature = resource;
                }
                Err(diagnostic) => {
                    *material_feature = PreparedMaterialFeatureResource {
                        status: FeatureContributionStatus::Missing,
                        fallback_policy:
                            engine::plugins::render::FeatureFallbackPolicy::SkipFeaturePasses,
                        payload: Default::default(),
                    };
                    return Some(diagnostic);
                }
            }
        } else {
            if material_feature.status != FeatureContributionStatus::Ready {
                *material_feature = PreparedMaterialFeatureResource {
                    status: FeatureContributionStatus::Missing,
                    fallback_policy:
                        engine::plugins::render::FeatureFallbackPolicy::SkipFeaturePasses,
                    payload: Default::default(),
                };
            }
        }
    } else {
        *material_feature = prepared_material_resource_for_preview(None)
            .expect("missing material preview cannot violate portable handoff limits");
    }
    None
}

pub fn produce_material_preview_dynamic_uploads_system(
    material_feature: Res<PreparedMaterialFeatureResource>,
    flow_registry: Res<RenderFlowRegistryResource>,
    product_targets: Res<ViewportProductTargetRegistryResource>,
    host: Res<EditorHostResource>,
    mut prepared_frame_requests: ResMut<PreparedRenderFrameRequestResource>,
) {
    let Some(flow_id) = material_preview_flow_id(&flow_registry) else {
        let _ = prepared_frame_requests
            .remove_contribution(EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID);
        return;
    };
    let Some(preview) = host.app.material_lab_runtime().active_preview() else {
        let _ = prepared_frame_requests
            .remove_contribution(EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID);
        return;
    };
    if material_feature.status != FeatureContributionStatus::Ready {
        let _ = prepared_frame_requests
            .remove_contribution(EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID);
        return;
    }

    let requests = material_preview_flow_requests(preview, flow_id, &product_targets);
    if requests.is_empty() {
        let _ = prepared_frame_requests
            .remove_contribution(EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID);
        return;
    }
    let views = requests
        .iter()
        .map(|(_, view, _)| view.clone())
        .collect::<Vec<_>>();
    let invocations = requests
        .into_iter()
        .map(|(_, _, request)| request)
        .collect::<Vec<_>>();
    prepared_frame_requests
        .replace_contribution(
            EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID,
            views,
            invocations,
        )
        .expect("material preview render producer contribution must be unique");
}

pub(crate) fn material_preview_flow_requests(
    preview: &EditorMaterialPreviewProduct,
    flow_id: RenderFlowId,
    product_targets: &ViewportProductTargetRegistryResource,
) -> Vec<(
    engine::plugins::render::RenderDynamicTextureTargetKey,
    PreparedViewFrame,
    PreparedFlowInvocationRequest,
)> {
    product_targets
        .records()
        .filter(|record| {
            record.key.presentation_slot == ViewportSurfacePresentationSlot::Primary
                && record.key.product_id == preview.viewport_product_id
                && record.status == ViewportProductTargetStatus::Requested
        })
        .map(|record| {
            let target = record.dynamic_key();
            let view_id = format!("editor.material.preview.{}.view", record.key.viewport_id.0);
            let view = PreparedViewFrame::offscreen_product(
                view_id.clone(),
                (record.width, record.height),
            );
            let request = PreparedFlowInvocationRequest {
                invocation_id: PreparedFlowInvocationId::new(format!(
                    "editor.material.preview.{}.{}",
                    record.key.viewport_id.0, preview.shader_identity
                )),
                flow_id,
                view_id,
                target_alias_bindings: BTreeMap::from([(
                    VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW.to_string(),
                    PreparedTargetBinding::DynamicTexture(target.clone()),
                )]),
                uniform_overrides: BTreeMap::new(),
                history_signature: Some(preview.shader_identity.clone()),
            };
            (target, view, request)
        })
        .collect()
}

fn material_preview_flow_id(flow_registry: &RenderFlowRegistryResource) -> Option<RenderFlowId> {
    flow_registry
        .compiled_flows()
        .iter()
        .find(|flow| flow.flow_label == EDITOR_MATERIAL_PREVIEW_FLOW_ID)
        .map(|flow| flow.flow_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{ArtifactCacheKey, asset_artifact_id, asset_id, asset_source_id};
    use editor_core::RealityVersion;
    use material_graph::{
        FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocumentId, MaterialOutputTarget,
        MaterialProductId,
    };

    use crate::material_lab::MaterialRendererParameterProfile;
    use crate::runtime::viewport::{
        ViewportProductTargetRegistryResource, material_preview_descriptor,
    };

    fn preview() -> EditorMaterialPreviewProduct {
        let product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        EditorMaterialPreviewProduct::new(
            asset_id(1),
            asset_source_id(2),
            asset_artifact_id(4),
            ArtifactCacheKey::new("asset-cache"),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(5),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material-shader-5.wgsl",
            "shader-identity",
            asset_artifact_id(6),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/material-scene-shader-6.wgsl",
            "scene-shader-identity",
            Vec::new(),
        )
    }

    #[test]
    fn material_preview_requests_write_selected_preview_target() {
        let preview = preview();
        let mut targets = ViewportProductTargetRegistryResource::default();
        targets.replace_records(
            ViewportProductTargetRegistryResource::from_descriptors_for_viewport(
                editor_viewport::ViewportId(1),
                &[material_preview_descriptor(
                    preview.viewport_product_id,
                    editor_viewport::ExpressionDimensions::new(64, 64),
                    RealityVersion(3),
                    preview.product.specialization_fragment.0.clone(),
                )],
            )
            .records()
            .cloned()
            .collect(),
        );

        let requests = material_preview_flow_requests(
            &preview,
            RenderFlowId::try_from_raw(99).unwrap(),
            &targets,
        );

        assert_eq!(requests.len(), 1);
        let target = requests[0].0.clone();
        assert_eq!(
            requests[0]
                .2
                .target_alias_bindings
                .get(VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW),
            Some(&PreparedTargetBinding::DynamicTexture(target))
        );
    }

    #[test]
    fn material_feature_preserves_prior_ready_bundle_while_new_shaders_load() {
        let preview = preview();
        let mut shader_registry = ShaderRegistryResource::new();
        let mut material_feature = PreparedMaterialFeatureResource {
            status: FeatureContributionStatus::Ready,
            fallback_policy: engine::plugins::render::FeatureFallbackPolicy::ReuseLastGood,
            payload: Default::default(),
        };

        let diagnostic = prepare_material_preview_render_resource(
            Some(&preview),
            &mut material_feature,
            &mut shader_registry,
        );

        assert!(diagnostic.is_none());
        assert_eq!(material_feature.status, FeatureContributionStatus::Ready);
    }

    #[test]
    fn material_feature_waits_for_preview_shader_and_carries_scene_bundle_without_global_rewrite() {
        let root = unique_temp_dir("material_generated_shader_load_gate");
        let preview_path = root.join("material-preview.wgsl");
        let scene_path = root.join("material-scene.wgsl");
        std::fs::write(&preview_path, "fn shader_preview_marker() {}\n")
            .expect("preview shader should write");
        std::fs::write(&scene_path, "fn shader_scene_marker() {}\n")
            .expect("scene shader should write");

        let mut preview = preview();
        preview.shader_path = preview_path.to_string_lossy().to_string();
        preview.scene_shader_path = scene_path.to_string_lossy().to_string();
        let mut shader_registry = ShaderRegistryResource::new();
        let mut material_feature = PreparedMaterialFeatureResource::default();

        let diagnostic = prepare_material_preview_render_resource(
            Some(&preview),
            &mut material_feature,
            &mut shader_registry,
        );
        assert!(diagnostic.is_none());
        assert_eq!(material_feature.status, FeatureContributionStatus::Missing);

        let lines = shader_registry.poll_updates();
        assert!(
            lines.iter().any(|line| line.contains("material-preview")),
            "the exact generated preview shader must load before preview producer handoff is ready"
        );
        let diagnostic = prepare_material_preview_render_resource(
            Some(&preview),
            &mut material_feature,
            &mut shader_registry,
        );
        assert!(diagnostic.is_none());

        assert_eq!(material_feature.status, FeatureContributionStatus::Ready);
        assert!(
            shader_registry
                .handle("editor_viewport_scene_product")
                .is_none(),
            "material preview preparation must not replace the global scene shader id"
        );
        assert_eq!(
            material_feature
                .payload
                .scene_bundle
                .as_ref()
                .map(|bundle| bundle.shader_path.as_str()),
            Some(preview.scene_shader_path.as_str())
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        root.push(format!("{label}_{nanos}"));
        std::fs::create_dir_all(&root).expect("temp dir should be creatable");
        root
    }
}
