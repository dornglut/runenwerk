//! App-owned preview scene product contracts.
//!
//! This module describes the resolved Material Lab preview scene product. It
//! carries stable product identities and resource layout references only; engine
//! renderer prepared structs are built later by the renderer handoff path.

use asset::ArtifactCacheKey;
use editor_viewport::ExpressionProductId;
use material_graph::MaterialProductId;

const PREVIEW_SCENE_PRODUCT_IDENTITY_VERSION: &str = "preview-scene-product-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreviewSceneProductMode {
    SingleMaterial,
    SceneMaterialTable,
}

impl PreviewSceneProductMode {
    const fn identity_tag(self) -> &'static str {
        match self {
            Self::SingleMaterial => "single_material",
            Self::SceneMaterialTable => "scene_material_table",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneShaderProductRef {
    pub shader_artifact_id: String,
    pub shader_cache_key: ArtifactCacheKey,
    pub shader_identity: String,
    pub shader_path: String,
    pub material_table_identity: String,
    pub resource_layout_identity: String,
}

impl PreviewSceneShaderProductRef {
    pub fn new(
        shader_artifact_id: impl Into<String>,
        shader_cache_key: ArtifactCacheKey,
        shader_identity: impl Into<String>,
        shader_path: impl Into<String>,
        material_table_identity: impl Into<String>,
        resource_layout_identity: impl Into<String>,
    ) -> Self {
        Self {
            shader_artifact_id: shader_artifact_id.into(),
            shader_cache_key,
            shader_identity: shader_identity.into(),
            shader_path: shader_path.into(),
            material_table_identity: material_table_identity.into(),
            resource_layout_identity: resource_layout_identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneResourceSlotMapping {
    pub local_resource_slot: u32,
    pub table_resource_slot: u32,
}

impl PreviewSceneResourceSlotMapping {
    pub const fn new(local_resource_slot: u32, table_resource_slot: u32) -> Self {
        Self {
            local_resource_slot,
            table_resource_slot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneMaterialSlot {
    pub material_slot_index: u32,
    pub scene_material_slot_id: String,
    pub material_product_id: MaterialProductId,
    pub material_artifact_cache_key: ArtifactCacheKey,
    pub scene_shader_identity: String,
    pub resource_slot_mappings: Vec<PreviewSceneResourceSlotMapping>,
}

impl PreviewSceneMaterialSlot {
    pub fn new(
        material_slot_index: u32,
        scene_material_slot_id: impl Into<String>,
        material_product_id: MaterialProductId,
        material_artifact_cache_key: ArtifactCacheKey,
        scene_shader_identity: impl Into<String>,
        resource_slot_mappings: impl IntoIterator<Item = PreviewSceneResourceSlotMapping>,
    ) -> Self {
        Self {
            material_slot_index,
            scene_material_slot_id: scene_material_slot_id.into(),
            material_product_id,
            material_artifact_cache_key,
            scene_shader_identity: scene_shader_identity.into(),
            resource_slot_mappings: resource_slot_mappings.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneResourceSlot {
    pub table_resource_slot: u32,
    pub resource_product_identity: String,
    pub resource_kind: String,
    pub texture_dimension: String,
    pub format_usage_contract: String,
    pub sampler_contract: String,
    pub artifact_identity: String,
    pub artifact_cache_key: ArtifactCacheKey,
}

impl PreviewSceneResourceSlot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        table_resource_slot: u32,
        resource_product_identity: impl Into<String>,
        resource_kind: impl Into<String>,
        texture_dimension: impl Into<String>,
        format_usage_contract: impl Into<String>,
        sampler_contract: impl Into<String>,
        artifact_identity: impl Into<String>,
        artifact_cache_key: ArtifactCacheKey,
    ) -> Self {
        Self {
            table_resource_slot,
            resource_product_identity: resource_product_identity.into(),
            resource_kind: resource_kind.into(),
            texture_dimension: texture_dimension.into(),
            format_usage_contract: format_usage_contract.into(),
            sampler_contract: sampler_contract.into(),
            artifact_identity: artifact_identity.into(),
            artifact_cache_key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneProduct {
    pub product_identity: String,
    pub mode: PreviewSceneProductMode,
    pub viewport_product_id: ExpressionProductId,
    pub active_material_product_id: MaterialProductId,
    pub active_material_artifact_cache_key: ArtifactCacheKey,
    pub material_table_identity: String,
    pub resource_layout_identity: String,
    pub shader: PreviewSceneShaderProductRef,
    pub slots: Vec<PreviewSceneMaterialSlot>,
    pub resources: Vec<PreviewSceneResourceSlot>,
}

impl PreviewSceneProduct {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: PreviewSceneProductMode,
        viewport_product_id: ExpressionProductId,
        active_material_product_id: MaterialProductId,
        active_material_artifact_cache_key: ArtifactCacheKey,
        material_table_identity: impl Into<String>,
        resource_layout_identity: impl Into<String>,
        shader: PreviewSceneShaderProductRef,
        slots: impl IntoIterator<Item = PreviewSceneMaterialSlot>,
        resources: impl IntoIterator<Item = PreviewSceneResourceSlot>,
    ) -> Self {
        let material_table_identity = material_table_identity.into();
        let resource_layout_identity = resource_layout_identity.into();
        let slots = slots.into_iter().collect::<Vec<_>>();
        let resources = resources.into_iter().collect::<Vec<_>>();
        let product_identity = preview_scene_product_identity(
            mode,
            viewport_product_id,
            active_material_product_id,
            &active_material_artifact_cache_key,
            &material_table_identity,
            &resource_layout_identity,
            &shader,
            &slots,
            &resources,
        );

        Self {
            product_identity,
            mode,
            viewport_product_id,
            active_material_product_id,
            active_material_artifact_cache_key,
            material_table_identity,
            resource_layout_identity,
            shader,
            slots,
            resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewSceneProductBuildStatus {
    Ready,
    WaitingForShaderLoad {
        product_identity: String,
    },
    FailedClosed {
        diagnostics: Vec<PreviewSceneProductDiagnostic>,
    },
    PriorValidPreserved {
        preserved_product_identity: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneProductDiagnostic {
    pub code: String,
    pub message: String,
}

impl PreviewSceneProductDiagnostic {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSceneProductBuildOutcome {
    pub product: Option<PreviewSceneProduct>,
    pub status: PreviewSceneProductBuildStatus,
}

impl PreviewSceneProductBuildOutcome {
    pub fn ready(product: PreviewSceneProduct) -> Self {
        Self {
            product: Some(product),
            status: PreviewSceneProductBuildStatus::Ready,
        }
    }

    pub fn failed_closed(
        diagnostics: impl IntoIterator<Item = PreviewSceneProductDiagnostic>,
    ) -> Self {
        Self {
            product: None,
            status: PreviewSceneProductBuildStatus::FailedClosed {
                diagnostics: diagnostics.into_iter().collect(),
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn preview_scene_product_identity(
    mode: PreviewSceneProductMode,
    viewport_product_id: ExpressionProductId,
    active_material_product_id: MaterialProductId,
    active_material_artifact_cache_key: &ArtifactCacheKey,
    material_table_identity: &str,
    resource_layout_identity: &str,
    shader: &PreviewSceneShaderProductRef,
    slots: &[PreviewSceneMaterialSlot],
    resources: &[PreviewSceneResourceSlot],
) -> String {
    let mut hasher = blake3::Hasher::new();
    update_identity(
        &mut hasher,
        "version",
        PREVIEW_SCENE_PRODUCT_IDENTITY_VERSION,
    );
    update_identity(&mut hasher, "mode", mode.identity_tag());
    update_u64(&mut hasher, "viewport_product_id", viewport_product_id.0);
    update_u64(
        &mut hasher,
        "active_material_product_id",
        active_material_product_id.raw(),
    );
    update_identity(
        &mut hasher,
        "active_material_artifact_cache_key",
        active_material_artifact_cache_key.as_str(),
    );
    update_identity(
        &mut hasher,
        "material_table_identity",
        material_table_identity,
    );
    update_identity(
        &mut hasher,
        "resource_layout_identity",
        resource_layout_identity,
    );
    update_shader_identity(&mut hasher, shader);

    update_u64(&mut hasher, "slot_count", slots.len() as u64);
    for slot in slots {
        update_u32(
            &mut hasher,
            "slot.material_slot_index",
            slot.material_slot_index,
        );
        update_identity(
            &mut hasher,
            "slot.scene_material_slot_id",
            &slot.scene_material_slot_id,
        );
        update_u64(
            &mut hasher,
            "slot.material_product_id",
            slot.material_product_id.raw(),
        );
        update_identity(
            &mut hasher,
            "slot.material_artifact_cache_key",
            slot.material_artifact_cache_key.as_str(),
        );
        update_identity(
            &mut hasher,
            "slot.scene_shader_identity",
            &slot.scene_shader_identity,
        );
        update_u64(
            &mut hasher,
            "slot.resource_mapping_count",
            slot.resource_slot_mappings.len() as u64,
        );
        for mapping in &slot.resource_slot_mappings {
            update_u32(
                &mut hasher,
                "slot.mapping.local_resource_slot",
                mapping.local_resource_slot,
            );
            update_u32(
                &mut hasher,
                "slot.mapping.table_resource_slot",
                mapping.table_resource_slot,
            );
        }
    }

    update_u64(&mut hasher, "resource_count", resources.len() as u64);
    for resource in resources {
        update_u32(
            &mut hasher,
            "resource.table_resource_slot",
            resource.table_resource_slot,
        );
        update_identity(
            &mut hasher,
            "resource.product_identity",
            &resource.resource_product_identity,
        );
        update_identity(&mut hasher, "resource.kind", &resource.resource_kind);
        update_identity(
            &mut hasher,
            "resource.texture_dimension",
            &resource.texture_dimension,
        );
        update_identity(
            &mut hasher,
            "resource.format_usage_contract",
            &resource.format_usage_contract,
        );
        update_identity(
            &mut hasher,
            "resource.sampler_contract",
            &resource.sampler_contract,
        );
        update_identity(
            &mut hasher,
            "resource.artifact_identity",
            &resource.artifact_identity,
        );
        update_identity(
            &mut hasher,
            "resource.artifact_cache_key",
            resource.artifact_cache_key.as_str(),
        );
    }

    format!(
        "{}:{}",
        PREVIEW_SCENE_PRODUCT_IDENTITY_VERSION,
        hasher.finalize().to_hex()
    )
}

fn update_shader_identity(hasher: &mut blake3::Hasher, shader: &PreviewSceneShaderProductRef) {
    update_identity(
        hasher,
        "shader.shader_artifact_id",
        &shader.shader_artifact_id,
    );
    update_identity(
        hasher,
        "shader.shader_cache_key",
        shader.shader_cache_key.as_str(),
    );
    update_identity(hasher, "shader.shader_identity", &shader.shader_identity);
    update_identity(
        hasher,
        "shader.material_table_identity",
        &shader.material_table_identity,
    );
    update_identity(
        hasher,
        "shader.resource_layout_identity",
        &shader.resource_layout_identity,
    );
}

fn update_identity(hasher: &mut blake3::Hasher, key: &str, value: &str) {
    hasher.update(key.as_bytes());
    hasher.update(&[0]);
    hasher.update(value.as_bytes());
    hasher.update(&[0xff]);
}

fn update_u64(hasher: &mut blake3::Hasher, key: &str, value: u64) {
    hasher.update(key.as_bytes());
    hasher.update(&[0]);
    hasher.update(&value.to_le_bytes());
    hasher.update(&[0xff]);
}

fn update_u32(hasher: &mut blake3::Hasher, key: &str, value: u32) {
    hasher.update(key.as_bytes());
    hasher.update(&[0]);
    hasher.update(&value.to_le_bytes());
    hasher.update(&[0xff]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shader_ref() -> PreviewSceneShaderProductRef {
        PreviewSceneShaderProductRef::new(
            "shader-artifact",
            ArtifactCacheKey::new("shader-cache"),
            "shader-identity",
            "generated/material-scene-table.wgsl",
            "table-identity",
            "resource-layout",
        )
    }

    fn material_slot(
        material_slot_index: u32,
        scene_material_slot_id: &str,
    ) -> PreviewSceneMaterialSlot {
        PreviewSceneMaterialSlot::new(
            material_slot_index,
            scene_material_slot_id,
            MaterialProductId::new(u64::from(material_slot_index) + 10),
            ArtifactCacheKey::new(format!("material-cache-{material_slot_index}")),
            format!("scene-shader-{material_slot_index}"),
            [PreviewSceneResourceSlotMapping::new(0, material_slot_index)],
        )
    }

    fn resource_slot(table_resource_slot: u32) -> PreviewSceneResourceSlot {
        PreviewSceneResourceSlot::new(
            table_resource_slot,
            format!("texture-product-{table_resource_slot}"),
            "texture_2d",
            "2d",
            "rgba8_unorm_srgb|sampled",
            "linear-repeat",
            format!("texture-artifact-{table_resource_slot}"),
            ArtifactCacheKey::new(format!("texture-cache-{table_resource_slot}")),
        )
    }

    fn product_with_parts(
        shader: PreviewSceneShaderProductRef,
        slots: Vec<PreviewSceneMaterialSlot>,
        resources: Vec<PreviewSceneResourceSlot>,
        resource_layout_identity: &str,
    ) -> PreviewSceneProduct {
        PreviewSceneProduct::new(
            PreviewSceneProductMode::SceneMaterialTable,
            ExpressionProductId(10_012),
            MaterialProductId::new(12),
            ArtifactCacheKey::new("active-material-cache"),
            "table-identity",
            resource_layout_identity,
            shader,
            slots,
            resources,
        )
    }

    fn baseline_product() -> PreviewSceneProduct {
        product_with_parts(
            shader_ref(),
            vec![material_slot(0, "default"), material_slot(1, "assigned")],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        )
    }

    #[test]
    fn product_identity_changes_when_material_slot_order_changes() {
        let baseline = baseline_product();
        let reordered = product_with_parts(
            shader_ref(),
            vec![material_slot(1, "assigned"), material_slot(0, "default")],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        assert_ne!(baseline.product_identity, reordered.product_identity);
    }

    #[test]
    fn product_identity_changes_when_resource_layout_identity_changes() {
        let baseline = baseline_product();
        let changed = product_with_parts(
            shader_ref(),
            vec![material_slot(0, "default"), material_slot(1, "assigned")],
            vec![resource_slot(0), resource_slot(1)],
            "changed-resource-layout",
        );

        assert_ne!(baseline.product_identity, changed.product_identity);
    }

    #[test]
    fn product_identity_changes_when_shader_identity_changes() {
        let baseline = baseline_product();
        let mut shader = shader_ref();
        shader.shader_identity = "changed-shader-identity".to_string();
        let changed = product_with_parts(
            shader,
            vec![material_slot(0, "default"), material_slot(1, "assigned")],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        assert_ne!(baseline.product_identity, changed.product_identity);
    }

    #[test]
    fn product_identity_changes_when_shader_artifact_or_cache_identity_changes() {
        let baseline = baseline_product();
        let mut changed_artifact = shader_ref();
        changed_artifact.shader_artifact_id = "changed-shader-artifact".to_string();
        let changed_artifact = product_with_parts(
            changed_artifact,
            vec![material_slot(0, "default"), material_slot(1, "assigned")],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        let mut changed_cache = shader_ref();
        changed_cache.shader_cache_key = ArtifactCacheKey::new("changed-shader-cache");
        let changed_cache = product_with_parts(
            changed_cache,
            vec![material_slot(0, "default"), material_slot(1, "assigned")],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        assert_ne!(baseline.product_identity, changed_artifact.product_identity);
        assert_ne!(baseline.product_identity, changed_cache.product_identity);
    }

    #[test]
    fn product_identity_changes_when_slot_to_table_resource_mapping_changes() {
        let baseline = baseline_product();
        let changed = product_with_parts(
            shader_ref(),
            vec![
                material_slot(0, "default"),
                PreviewSceneMaterialSlot::new(
                    1,
                    "assigned",
                    MaterialProductId::new(11),
                    ArtifactCacheKey::new("material-cache-1"),
                    "scene-shader-1",
                    [PreviewSceneResourceSlotMapping::new(0, 0)],
                ),
            ],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        assert_ne!(baseline.product_identity, changed.product_identity);
    }

    #[test]
    fn product_identity_changes_when_material_product_or_cache_identity_changes() {
        let baseline = baseline_product();
        let changed_product = product_with_parts(
            shader_ref(),
            vec![
                PreviewSceneMaterialSlot::new(
                    0,
                    "default",
                    MaterialProductId::new(99),
                    ArtifactCacheKey::new("material-cache-0"),
                    "scene-shader-0",
                    [PreviewSceneResourceSlotMapping::new(0, 0)],
                ),
                material_slot(1, "assigned"),
            ],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );
        let changed_cache = product_with_parts(
            shader_ref(),
            vec![
                PreviewSceneMaterialSlot::new(
                    0,
                    "default",
                    MaterialProductId::new(10),
                    ArtifactCacheKey::new("changed-material-cache"),
                    "scene-shader-0",
                    [PreviewSceneResourceSlotMapping::new(0, 0)],
                ),
                material_slot(1, "assigned"),
            ],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        assert_ne!(baseline.product_identity, changed_product.product_identity);
        assert_ne!(baseline.product_identity, changed_cache.product_identity);
    }

    #[test]
    fn product_identity_ignores_diagnostic_message_and_status_text() {
        let product = baseline_product();
        let ready = PreviewSceneProductBuildOutcome::ready(product.clone());
        let waiting = PreviewSceneProductBuildOutcome {
            product: Some(product.clone()),
            status: PreviewSceneProductBuildStatus::WaitingForShaderLoad {
                product_identity: "different-status-text".to_string(),
            },
        };
        let failed = PreviewSceneProductBuildOutcome::failed_closed([
            PreviewSceneProductDiagnostic::new(
                "material.preview_scene.stale_bundle",
                "first wording",
            ),
            PreviewSceneProductDiagnostic::new(
                "material.preview_scene.stale_bundle",
                "different wording",
            ),
        ]);

        assert_eq!(
            ready.product.as_ref().unwrap().product_identity,
            product.product_identity
        );
        assert_eq!(
            waiting.product.as_ref().unwrap().product_identity,
            product.product_identity
        );
        assert_eq!(failed.product, None);
        assert_eq!(
            product.product_identity,
            baseline_product().product_identity
        );
    }

    #[test]
    fn resource_slots_are_table_wide_and_do_not_use_local_slot_identity() {
        let product = baseline_product();

        assert_eq!(product.resources[0].table_resource_slot, 0);
        assert_eq!(product.resources[1].table_resource_slot, 1);
        assert_ne!(
            product.resources[0].resource_product_identity,
            product.resources[1].resource_product_identity
        );
        assert_eq!(
            product.slots[0].resource_slot_mappings[0].local_resource_slot,
            0
        );
        assert_eq!(
            product.slots[1].resource_slot_mappings[0].local_resource_slot,
            0
        );
        assert_ne!(
            product.slots[0].resource_slot_mappings[0].table_resource_slot,
            product.slots[1].resource_slot_mappings[0].table_resource_slot
        );
    }

    #[test]
    fn preview_scene_product_does_not_store_material_graph_source_documents() {
        let product = baseline_product();
        let debug = format!("{product:?}");

        assert!(!debug.contains("MaterialGraphDocument"));
        assert!(!debug.contains("source_document"));
    }

    #[test]
    fn preview_scene_product_does_not_use_engine_prepared_scene_material_bundle() {
        let product_type = std::any::type_name::<PreviewSceneProduct>();
        let shader_ref_type = std::any::type_name::<PreviewSceneShaderProductRef>();

        assert!(!product_type.contains("PreparedSceneMaterialBundle"));
        assert!(shader_ref_type.contains("PreviewSceneShaderProductRef"));
    }

    #[test]
    fn shader_paths_are_stored_but_do_not_drive_product_identity() {
        let baseline = baseline_product();
        let mut shader = shader_ref();
        shader.shader_path = "different/generated/path.wgsl".to_string();
        let changed_path = product_with_parts(
            shader,
            vec![material_slot(0, "default"), material_slot(1, "assigned")],
            vec![resource_slot(0), resource_slot(1)],
            "resource-layout",
        );

        assert_eq!(baseline.product_identity, changed_path.product_identity);
    }
}
