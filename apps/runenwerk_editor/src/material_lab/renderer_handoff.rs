use asset::{AssetDiagnosticCode, AssetDiagnosticRecord};
use engine::plugins::render::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedMaterialBindingSlot,
    PreparedMaterialBindingTable, PreparedMaterialFeatureContribution,
    PreparedMaterialFeatureResource, PreparedMaterialInstanceInput, PreparedMaterialOutputTarget,
    PreparedMaterialParameterInput, PreparedMaterialParameterKind,
    PreparedMaterialParameterPayloadV1, PreparedMaterialParameterProfile,
    PreparedMaterialTextureBinding, PreparedMaterialTextureKind, PreparedSceneMaterialBundle,
};
use material_graph::{MaterialOutputTarget, MaterialParameterKind};

use crate::material_lab::{EditorMaterialPreviewProduct, MaterialRendererParameterProfile};

pub fn prepared_material_contribution_for_preview(
    preview: &EditorMaterialPreviewProduct,
) -> PreparedMaterialFeatureContribution {
    PreparedMaterialFeatureContribution {
        instances: vec![PreparedMaterialInstanceInput {
            material_instance_id: format!("material.product.{}", preview.product.product_id.raw()),
            specialization_key_fragment: preview.product.specialization_fragment.0.clone(),
            parameter_payload: material_parameter_payload(preview),
            texture_bindings: prepared_texture_bindings(preview),
        }],
        binding_table: PreparedMaterialBindingTable::fixed_capacity([
            PreparedMaterialBindingSlot::new(
                0,
                format!("material.product.{}", preview.product.product_id.raw()),
                preview.artifact_id.raw().to_string(),
                preview.shader_artifact_id.raw().to_string(),
                preview.artifact_cache_key.as_str().to_string(),
                preview.shader_cache_key.as_str().to_string(),
            ),
        ])
        .expect("single material preview slot is within portable binding table limits"),
        scene_bundle: Some(PreparedSceneMaterialBundle::new(
            preview.scene_shader_artifact_id.raw().to_string(),
            preview.scene_shader_cache_key.as_str().to_string(),
            preview.scene_shader_path.clone(),
            preview.scene_shader_identity.clone(),
            format!(
                "fixed64:slot0:{}:{}:{}",
                preview.product.product_id.raw(),
                preview.artifact_cache_key.as_str(),
                preview.shader_cache_key.as_str()
            ),
        )),
    }
}

pub fn prepared_material_resource_for_preview(
    preview: Option<&EditorMaterialPreviewProduct>,
) -> Result<PreparedMaterialFeatureResource, AssetDiagnosticRecord> {
    match preview {
        Some(preview) => {
            let payload = prepared_material_contribution_for_preview(preview);
            payload.validate_portable_limits().map_err(|error| {
                AssetDiagnosticRecord::error(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "material renderer handoff rejected portable binding limits: {}",
                        error
                    ),
                )
            })?;
            Ok(PreparedMaterialFeatureResource {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
                payload,
            })
        }
        None => Ok(PreparedMaterialFeatureResource {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
            payload: PreparedMaterialFeatureContribution::default(),
        }),
    }
}

pub fn material_parameter_payload(
    preview: &EditorMaterialPreviewProduct,
) -> PreparedMaterialParameterPayloadV1 {
    PreparedMaterialParameterPayloadV1::new(
        prepared_parameter_profile(preview.renderer_parameter_profile),
        prepared_material_output_target(preview.product.output_target),
        preview.product.parameters.iter().map(|parameter| {
            PreparedMaterialParameterInput::new(
                parameter.key.clone(),
                prepared_parameter_kind(parameter.kind),
            )
        }),
    )
}

fn prepared_parameter_profile(
    profile: MaterialRendererParameterProfile,
) -> PreparedMaterialParameterProfile {
    match profile {
        MaterialRendererParameterProfile::PbrPreview => {
            PreparedMaterialParameterProfile::PbrPreview
        }
        MaterialRendererParameterProfile::RenderMaterial => {
            PreparedMaterialParameterProfile::RenderMaterial
        }
    }
}

fn prepared_material_output_target(
    output_target: MaterialOutputTarget,
) -> PreparedMaterialOutputTarget {
    match output_target {
        MaterialOutputTarget::PbrPreview => PreparedMaterialOutputTarget::PbrPreview,
        MaterialOutputTarget::FieldMaterialChannel => {
            PreparedMaterialOutputTarget::FieldMaterialChannel
        }
        MaterialOutputTarget::RenderMaterial => PreparedMaterialOutputTarget::RenderMaterial,
    }
}

fn prepared_parameter_kind(kind: MaterialParameterKind) -> PreparedMaterialParameterKind {
    match kind {
        MaterialParameterKind::Scalar => PreparedMaterialParameterKind::Scalar,
        MaterialParameterKind::Vector2 => PreparedMaterialParameterKind::Vector2,
        MaterialParameterKind::Vector3 => PreparedMaterialParameterKind::Vector3,
        MaterialParameterKind::Vector4 => PreparedMaterialParameterKind::Vector4,
        MaterialParameterKind::Texture2D => PreparedMaterialParameterKind::Texture2D,
        MaterialParameterKind::Texture3D => PreparedMaterialParameterKind::Texture3D,
    }
}

fn prepared_texture_bindings(
    preview: &EditorMaterialPreviewProduct,
) -> Vec<PreparedMaterialTextureBinding> {
    preview
        .resolved_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| {
            let mut binding = PreparedMaterialTextureBinding::new(
                resource.node_id.raw(),
                resource.binding_key.clone(),
                resource.artifact_id.raw().to_string(),
                resource.artifact_path.clone(),
                match resource.kind {
                    asset::AssetKind::Texture3DVolume => PreparedMaterialTextureKind::Texture3D,
                    _ => PreparedMaterialTextureKind::Texture2D,
                },
                resource.cache_key.as_str().to_string(),
            )
            .with_resource_slot_index(index as u32)
            .with_texture_dimension(resource.dimension.clone())
            .with_extent(
                resource.descriptor.extent.width,
                resource.descriptor.extent.height,
                resource.descriptor.extent.depth,
            )
            .with_residency_identity(resource.residency_identity.clone())
            .with_artifact_revision(resource.artifact_revision.clone())
            .with_descriptor_hash(resource.descriptor.descriptor_hash().to_string())
            .with_ktx2_contract(
                format!("{:?}", resource.descriptor.ktx2_metadata().pixel_format),
                format!("{:?}", resource.descriptor.ktx2_metadata().supercompression),
                resource.descriptor.ktx2_metadata().byte_length,
            );
            binding.sampler_policy = resource.sampler_policy.clone();
            binding
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material_lab::MaterialRendererParameterProfile;
    use asset::{
        ArtifactCacheKey, AssetDiagnosticCode, AssetKind, asset_artifact_id, asset_id,
        asset_source_id,
    };
    use material_graph::{
        FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocumentId, MaterialOutputTarget,
        MaterialParameterDescriptor, MaterialParameterKind, MaterialProductId,
    };
    use resource_ref::ResourceRef;
    use texture::{TextureDescriptor, TextureDimension, TextureExtent, TextureProductId};

    #[test]
    fn material_handoff_prepared_resource_uses_formed_product_specialization_and_parameters() {
        let mut product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        product.parameters = vec![MaterialParameterDescriptor::new(
            "roughness",
            MaterialParameterKind::Scalar,
        )];
        let preview = EditorMaterialPreviewProduct::new(
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
        );

        let prepared =
            prepared_material_resource_for_preview(Some(&preview)).expect("handoff should prepare");

        assert_eq!(prepared.status, FeatureContributionStatus::Ready);
        assert_eq!(prepared.payload.instances.len(), 1);
        assert_eq!(
            prepared.payload.instances[0].specialization_key_fragment,
            "material.first_slice"
        );
        let encoded = prepared.payload.instances[0].parameter_payload.encode_v1();
        let payload =
            PreparedMaterialParameterPayloadV1::decode_v1(&encoded).expect("payload should decode");
        assert_eq!(
            payload.profile,
            PreparedMaterialParameterProfile::RenderMaterial
        );
        assert_eq!(
            payload.output_target,
            PreparedMaterialOutputTarget::RenderMaterial
        );
        assert_eq!(payload.parameters.len(), 1);
        assert_eq!(payload.parameters[0].key, "roughness");
        assert_eq!(prepared.payload.binding_table.slots.len(), 1);
        let scene_bundle = prepared
            .payload
            .scene_bundle
            .as_ref()
            .expect("material handoff should carry the scene bundle as feature data");
        assert_eq!(
            scene_bundle.shader_path,
            ".runenwerk/artifacts/material-scene-shader-6.wgsl"
        );
        assert_eq!(scene_bundle.shader_identity, "scene-shader-identity");
        let blob = std::str::from_utf8(&encoded).expect("payload should be utf8");
        assert!(blob.contains("format=32:runenwerk.material-parameters.v1"));
        assert!(blob.contains("profile=15:render_material"));
        assert!(blob.contains("parameter_kind=6:scalar"));
        assert!(
            !blob.contains("Scalar"),
            "prepared material payload must not use Rust debug formatting"
        );
    }

    #[test]
    fn material_handoff_reports_portable_limit_diagnostics() {
        let mut product = FormedMaterialProduct::new(
            MaterialProductId::new(3),
            MaterialGraphDocumentId::new(2),
            MaterialOutputTarget::RenderMaterial,
            MaterialCacheKey::new("material-cache"),
        );
        product.parameters = vec![MaterialParameterDescriptor::new(
            "albedo",
            MaterialParameterKind::Texture2D,
        )];
        let mut preview = EditorMaterialPreviewProduct::new(
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
            (0..129)
                .map(test_resolved_texture_resource)
                .collect::<Vec<_>>(),
        );

        let diagnostic = prepared_material_resource_for_preview(Some(&preview))
            .expect_err("portable texture binding limit must be a visible diagnostic");

        assert_eq!(diagnostic.code, AssetDiagnosticCode::RatificationRejected);
        assert!(
            diagnostic.message.contains("portable binding limits"),
            "portable limit failure should not collapse into a generic missing feature"
        );

        preview.resolved_resources.truncate(128);
        assert!(prepared_material_resource_for_preview(Some(&preview)).is_ok());
    }

    fn test_resolved_texture_resource(index: u64) -> crate::material_lab::ResolvedMaterialResource {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(index + 1),
            format!("Texture {index}"),
            TextureDimension::Texture2D,
            TextureExtent::new(1, 1, 1),
        );
        crate::material_lab::ResolvedMaterialResource {
            node_id: graph::NodeId::new(index + 1),
            binding_key: format!("albedo_{index}"),
            reference: ResourceRef::new("texture", format!("texture.{index}"))
                .expect("test resource ref should be valid"),
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
}
