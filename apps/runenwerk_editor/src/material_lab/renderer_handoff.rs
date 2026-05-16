use engine::plugins::render::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedMaterialFeatureContribution,
    PreparedMaterialFeatureResource, PreparedMaterialInstanceInput,
};

use crate::material_lab::EditorMaterialPreviewProduct;

pub fn prepared_material_contribution_for_preview(
    preview: &EditorMaterialPreviewProduct,
) -> PreparedMaterialFeatureContribution {
    PreparedMaterialFeatureContribution {
        instances: vec![PreparedMaterialInstanceInput {
            material_instance_id: format!("material.product.{}", preview.product.product_id.raw()),
            specialization_key_fragment: preview.product.specialization_fragment.0.clone(),
            parameter_blob: material_parameter_blob(preview),
        }],
    }
}

pub fn prepared_material_resource_for_preview(
    preview: Option<&EditorMaterialPreviewProduct>,
) -> PreparedMaterialFeatureResource {
    match preview {
        Some(preview) => PreparedMaterialFeatureResource {
            status: FeatureContributionStatus::Ready,
            fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
            payload: prepared_material_contribution_for_preview(preview),
        },
        None => PreparedMaterialFeatureResource {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
            payload: PreparedMaterialFeatureContribution::default(),
        },
    }
}

pub fn material_parameter_blob(preview: &EditorMaterialPreviewProduct) -> Vec<u8> {
    preview
        .product
        .parameters
        .iter()
        .map(|parameter| format!("{}:{:?}", parameter.key, parameter.kind))
        .collect::<Vec<_>>()
        .join(";")
        .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{ArtifactCacheKey, asset_artifact_id, asset_id, asset_source_id};
    use material_graph::{
        FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocumentId, MaterialOutputTarget,
        MaterialParameterDescriptor, MaterialParameterKind, MaterialProductId,
    };

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
        );

        let prepared = prepared_material_resource_for_preview(Some(&preview));

        assert_eq!(prepared.status, FeatureContributionStatus::Ready);
        assert_eq!(prepared.payload.instances.len(), 1);
        assert_eq!(
            prepared.payload.instances[0].specialization_key_fragment,
            "material.first_slice"
        );
        assert!(
            std::str::from_utf8(&prepared.payload.instances[0].parameter_blob)
                .expect("blob should be utf8")
                .contains("roughness")
        );
    }
}
