//! File: apps/runenwerk_editor/src/runtime/systems/material_preview.rs
//! Purpose: Material Lab prepared renderer handoff and preview render producer.

use std::collections::BTreeMap;

use editor_scene::SceneMaterialAssignmentState;
use editor_viewport::ViewportSurfacePresentationSlot;
use engine::plugins::render::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedFlowInvocationId,
    PreparedFlowInvocationRequest, PreparedMaterialFeatureResource,
    PreparedRenderFrameRequestResource, PreparedTargetBinding, PreparedViewFrame, RenderFlowId,
    RenderFlowRegistryResource, ShaderRegistryResource, compile_scene_material_table_shader,
};
use engine::runtime::{Res, ResMut};

use crate::material_lab::workflow::write_scene_material_table_shader_bundle;
use crate::material_lab::{
    EditorMaterialPreviewProduct, EditorSceneMaterialTableShaderBundle, SceneMaterialSlotProduct,
    prepared_material_resource_for_preview,
    prepared_material_resource_for_preview_with_resolved_scene_materials_and_bundle,
    scene_material_table_shader_build_request_for_preview,
};
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
    let scene_material_assignments = host.app.runtime().scene_material_assignments().clone();
    let slot_product_values = {
        let slot_products = collect_scene_material_slot_products(
            &scene_material_assignments,
            host.app.material_lab_runtime(),
        );
        slot_products
            .into_iter()
            .map(|slot_product| (slot_product.slot_id, slot_product.preview.clone()))
            .collect::<Vec<_>>()
    };
    let slot_products = slot_product_values
        .iter()
        .map(|(slot_id, preview)| SceneMaterialSlotProduct {
            slot_id: *slot_id,
            preview,
        })
        .collect::<Vec<_>>();
    let scene_table_bundle = match ensure_scene_material_table_shader_bundle(
        &mut host.app,
        preview.as_ref(),
        Some(&scene_material_assignments),
        &slot_products,
    ) {
        Ok(bundle) => bundle,
        Err(diagnostic) => {
            *material_feature = PreparedMaterialFeatureResource {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
                payload: Default::default(),
            };
            host.app
                .material_lab_runtime_mut()
                .record_diagnostic(diagnostic.clone());
            return;
        }
    };
    if let Some(diagnostic) = prepare_material_preview_render_resource_with_scene_materials(
        preview.as_ref(),
        Some(&scene_material_assignments),
        &slot_products,
        scene_table_bundle.as_ref(),
        &mut material_feature,
        &mut shader_registry,
    ) {
        host.app
            .material_lab_runtime_mut()
            .record_diagnostic(diagnostic);
    }
}

#[cfg(test)]
pub(crate) fn prepare_material_preview_render_resource(
    preview: Option<&EditorMaterialPreviewProduct>,
    material_feature: &mut PreparedMaterialFeatureResource,
    shader_registry: &mut ShaderRegistryResource,
) -> Option<asset::AssetDiagnosticRecord> {
    prepare_material_preview_render_resource_with_scene_materials(
        preview,
        None,
        &[],
        None,
        material_feature,
        shader_registry,
    )
}

pub(crate) fn prepare_material_preview_render_resource_with_scene_materials(
    preview: Option<&EditorMaterialPreviewProduct>,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
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
        if let Some(bundle) = scene_table_bundle {
            shader_registry
                .register_shader_with_id(bundle.shader_path.clone(), bundle.shader_path.clone());
        } else {
            for slot_product in slot_products {
                shader_registry.register_shader_with_id(
                    slot_product.preview.scene_shader_path.clone(),
                    slot_product.preview.scene_shader_path.clone(),
                );
            }
        }
        if shader_registry.is_loaded(EDITOR_MATERIAL_PREVIEW_SHADER_ID)
            && scene_shader_bundle_is_loaded(
                preview,
                slot_products,
                scene_table_bundle,
                shader_registry,
            )
        {
            match prepared_material_resource_for_preview_with_resolved_scene_materials_and_bundle(
                Some(preview),
                scene_material_assignments,
                &slot_products,
                scene_table_bundle,
            ) {
                Ok(resource) => {
                    *material_feature = resource;
                }
                Err(diagnostic) => {
                    *material_feature = PreparedMaterialFeatureResource {
                        status: FeatureContributionStatus::Missing,
                        fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
                        payload: Default::default(),
                    };
                    return Some(diagnostic);
                }
            }
        } else {
            if material_feature.status != FeatureContributionStatus::Ready {
                *material_feature = PreparedMaterialFeatureResource {
                    status: FeatureContributionStatus::Missing,
                    fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
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

fn scene_shader_bundle_is_loaded(
    preview: &EditorMaterialPreviewProduct,
    slot_products: &[SceneMaterialSlotProduct<'_>],
    scene_table_bundle: Option<&EditorSceneMaterialTableShaderBundle>,
    shader_registry: &ShaderRegistryResource,
) -> bool {
    if let Some(bundle) = scene_table_bundle {
        return shader_registry.is_loaded(bundle.shader_path.as_str());
    }
    shader_registry.is_loaded(preview.scene_shader_path.as_str())
        && slot_products.iter().all(|slot_product| {
            shader_registry.is_loaded(slot_product.preview.scene_shader_path.as_str())
        })
}

fn ensure_scene_material_table_shader_bundle(
    app: &mut crate::editor_app::RunenwerkEditorApp,
    preview: Option<&EditorMaterialPreviewProduct>,
    scene_material_assignments: Option<&SceneMaterialAssignmentState>,
    slot_products: &[SceneMaterialSlotProduct<'_>],
) -> Result<Option<EditorSceneMaterialTableShaderBundle>, asset::AssetDiagnosticRecord> {
    let Some(preview) = preview else {
        app.material_lab_runtime_mut()
            .clear_scene_material_table_shader_bundle();
        return Ok(None);
    };
    let Some(request) = scene_material_table_shader_build_request_for_preview(
        preview,
        scene_material_assignments,
        slot_products,
    )?
    else {
        app.material_lab_runtime_mut()
            .clear_scene_material_table_shader_bundle();
        return Ok(None);
    };
    if let Some(bundle) = app
        .material_lab_runtime()
        .scene_material_table_shader_bundle()
        && bundle.matches_scene_table(
            request.material_table_identity.as_str(),
            request.resource_layout_identity.as_str(),
        )
    {
        return Ok(Some(bundle.clone()));
    }
    let compiled =
        compile_scene_material_table_shader(request.compile_request).map_err(|error| {
            asset::AssetDiagnosticRecord::error(
                asset::AssetDiagnosticCode::RatificationRejected,
                format!("scene material table shader compilation failed: {error}"),
            )
        })?;
    let project_root = scene_material_table_shader_project_root(app);
    let bundle = write_scene_material_table_shader_bundle(
        project_root.as_path(),
        request.material_table_identity.as_str(),
        request.resource_layout_identity.as_str(),
        &compiled,
    )
    .map_err(|error| {
        asset::AssetDiagnosticRecord::error(
            asset::AssetDiagnosticCode::RatificationRejected,
            format!("scene material table shader artifact generation failed: {error}"),
        )
    })?;
    app.material_lab_runtime_mut()
        .set_scene_material_table_shader_bundle(bundle.clone());
    Ok(Some(bundle))
}

fn scene_material_table_shader_project_root(
    app: &crate::editor_app::RunenwerkEditorApp,
) -> std::path::PathBuf {
    app.asset_project_session()
        .map(|session| session.project_root().to_path_buf())
        .unwrap_or_else(|| {
            std::env::temp_dir()
                .join("runenwerk")
                .join("material-scene-table-runtime-cache")
        })
}

fn collect_scene_material_slot_products<'a>(
    scene_material_assignments: &'a SceneMaterialAssignmentState,
    material_runtime: &'a crate::material_lab::MaterialLabRuntime,
) -> Vec<SceneMaterialSlotProduct<'a>> {
    scene_material_assignments
        .palette()
        .slots
        .iter()
        .filter_map(|slot| {
            if slot.is_default && slot.material_asset_id.is_none() && slot.source_ref.is_none() {
                return None;
            }
            let preview = slot
                .material_asset_id
                .and_then(|asset_id| material_runtime.preview_product_for_asset(asset_id))
                .or_else(|| {
                    slot.source_ref.as_ref().and_then(|source_ref| {
                        material_runtime
                            .preview_product_for_asset(source_ref.asset_id)
                            .filter(|preview| preview.source_id == source_ref.source_id)
                    })
                })?;
            Some(SceneMaterialSlotProduct {
                slot_id: slot.slot_id,
                preview,
            })
        })
        .collect()
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
    use asset::{ArtifactCacheKey, AssetKind, asset_artifact_id, asset_id, asset_source_id};
    use editor_core::RealityVersion;
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition,
        NodeId, PortDefinition, PortDirection, PortId, PortTypeId,
    };
    use material_graph::{
        FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocument, MaterialGraphDocumentId,
        MaterialIr, MaterialNodeCatalog, MaterialOutputTarget, MaterialProductId,
        lower_material_graph,
    };
    use resource_ref::ResourceRef;
    use texture::{TextureDescriptor, TextureDimension, TextureExtent, TextureProductId};

    use crate::material_lab::{MaterialRendererParameterProfile, ResolvedMaterialResource};
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

    #[test]
    fn material_preview_uses_generated_scene_table_shader_bundle_when_required() {
        let root = unique_temp_dir("material_scene_table_shader_bundle");
        let preview_path = root.join("material-preview.wgsl");
        std::fs::write(&preview_path, "fn shader_preview_marker() {}\n")
            .expect("preview shader should write");
        let default_preview = texture_preview(
            asset_id(1),
            3,
            4,
            5,
            6,
            "default",
            "texture.default",
            preview_path.to_string_lossy().to_string(),
        );
        let assigned_preview = texture_preview(
            asset_id(8),
            9,
            10,
            11,
            12,
            "rock",
            "texture.rock",
            preview_path.to_string_lossy().to_string(),
        );
        let assigned_slot = editor_scene::SceneMaterialSlot::new(
            editor_scene::SceneMaterialSlotId::new(2),
            "Assigned",
        )
        .with_material_asset(asset_id(8));
        let assignments = SceneMaterialAssignmentState::new(
            editor_scene::SceneMaterialPalette::new([
                editor_scene::SceneMaterialSlot::default_generated(),
                assigned_slot,
            ])
            .expect("valid palette"),
            [],
        )
        .expect("valid assignments");
        let slot_products = [SceneMaterialSlotProduct {
            slot_id: editor_scene::SceneMaterialSlotId::new(2),
            preview: &assigned_preview,
        }];
        let mut app = crate::editor_app::RunenwerkEditorApp::new();
        let bundle = ensure_scene_material_table_shader_bundle(
            &mut app,
            Some(&default_preview),
            Some(&assignments),
            &slot_products,
        )
        .expect("scene table shader generation should succeed")
        .expect("multi-slot table requires generated bundle");
        assert!(
            std::path::Path::new(&bundle.shader_path).exists(),
            "generated scene table shader bundle must be written before renderer handoff"
        );
        let mut shader_registry = ShaderRegistryResource::new();
        let mut material_feature = PreparedMaterialFeatureResource::default();

        let diagnostic = prepare_material_preview_render_resource_with_scene_materials(
            Some(&default_preview),
            Some(&assignments),
            &slot_products,
            Some(&bundle),
            &mut material_feature,
            &mut shader_registry,
        );
        assert!(diagnostic.is_none());
        assert_eq!(material_feature.status, FeatureContributionStatus::Missing);
        let _ = shader_registry.poll_updates();
        let diagnostic = prepare_material_preview_render_resource_with_scene_materials(
            Some(&default_preview),
            Some(&assignments),
            &slot_products,
            Some(&bundle),
            &mut material_feature,
            &mut shader_registry,
        );
        assert!(diagnostic.is_none());

        assert_eq!(material_feature.status, FeatureContributionStatus::Ready);
        let scene_bundle = material_feature
            .payload
            .scene_bundle
            .as_ref()
            .expect("generated scene table bundle should be handed off");
        assert_eq!(scene_bundle.shader_path, bundle.shader_path);
        assert_eq!(
            scene_bundle.material_table_identity,
            bundle.material_table_identity
        );
        assert_eq!(
            scene_bundle.resource_layout_identity,
            bundle.resource_layout_identity
        );
        let _ = std::fs::remove_dir_all(root);
    }

    fn texture_preview(
        asset_id: asset::AssetId,
        product_id: u64,
        artifact_id: u64,
        shader_artifact_id: u64,
        scene_shader_artifact_id: u64,
        label: &str,
        texture_stable_id: &str,
        shader_path: String,
    ) -> EditorMaterialPreviewProduct {
        let ir = texture_ir(product_id + 100, texture_stable_id);
        let mut product = FormedMaterialProduct::new(
            MaterialProductId::new(product_id),
            MaterialGraphDocumentId::new(product_id + 100),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new(format!("material-cache-{label}")),
        );
        product.executable_ir = Some(ir);
        EditorMaterialPreviewProduct::new(
            asset_id,
            asset_source_id(product_id + 200),
            asset_artifact_id(artifact_id),
            ArtifactCacheKey::new(format!("asset-cache-{label}")),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(shader_artifact_id),
            ArtifactCacheKey::new(format!("shader-cache-{label}")),
            shader_path,
            format!("shader-identity-{label}"),
            asset_artifact_id(scene_shader_artifact_id),
            ArtifactCacheKey::new(format!("scene-shader-cache-{label}")),
            format!(".runenwerk/artifacts/material-scene-shader-{label}.wgsl"),
            format!("scene-shader-identity-{label}"),
            vec![resolved_texture_resource(product_id, texture_stable_id)],
        )
    }

    fn texture_ir(document_id: u64, texture_stable_id: &str) -> MaterialIr {
        let color = PortTypeId::new(1);
        let vec2 = PortTypeId::new(3);
        let texture_ref =
            ResourceRef::new("asset.catalog.texture2d", texture_stable_id).expect("resource ref");
        let document = MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(document_id),
            "texture_slot",
            GraphDefinition::new(
                GraphId::new(1),
                "texture_slot",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(1),
                        "texture.sample_2d",
                        [
                            PortDefinition::new(PortId::new(1), "uv", PortDirection::Input, vec2),
                            PortDefinition::new(
                                PortId::new(2),
                                "color",
                                PortDirection::Output,
                                color,
                            ),
                        ],
                    )
                    .with_values([GraphMetadataEntry::new(
                        material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                        GraphValue::resource(texture_ref),
                    )]),
                    NodeDefinition::new(
                        NodeId::new(2),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(3),
                            "base_color",
                            PortDirection::Input,
                            color,
                        )],
                    ),
                ],
                [graph::EdgeDefinition::new(
                    graph::EdgeId::new(1),
                    PortId::new(2),
                    PortId::new(3),
                )],
            ),
            MaterialOutputTarget::RenderMaterial,
        );
        let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
        assert!(
            !lowering.report.has_blocking_issues(),
            "{:?}",
            lowering.report.issues()
        );
        lowering.product.expect("formed").executable_ir.expect("ir")
    }

    fn resolved_texture_resource(index: u64, texture_stable_id: &str) -> ResolvedMaterialResource {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(index + 1000),
            format!("Texture {index}"),
            TextureDimension::Texture2D,
            TextureExtent::new(1, 1, 1),
        );
        ResolvedMaterialResource {
            node_id: graph::NodeId::new(1),
            binding_key: material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF.to_string(),
            reference: ResourceRef::new("asset.catalog.texture2d", texture_stable_id)
                .expect("resource ref"),
            artifact_id: asset_artifact_id(index + 100),
            artifact_path: format!(".runenwerk/artifacts/texture-{index}.ktx2"),
            kind: AssetKind::Texture2D,
            cache_key: ArtifactCacheKey::new(format!("texture-cache-{index}")),
            descriptor,
            artifact_revision: "1".to_string(),
            dimension: "2d".to_string(),
            color_space: "linear".to_string(),
            sampler_policy: "linear_repeat".to_string(),
            residency_identity: format!("ktx2:texture:{index}"),
        }
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
