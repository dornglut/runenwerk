use product::{
    ProductAuthorityClass, ProductDeterminismClass, ProductIdentity, ProductJobAffinity,
    ProductJobBudgetClass, ProductJobDescriptor, ProductJobFailurePolicy, ProductJobId,
    ProductKind, ProductScaleBand, ProductScope,
};
use serde::{Deserialize, Serialize};

use crate::{
    ArtifactCacheKey, AssetDiagnosticRecord, AssetId, AssetKind, AssetSourceDescriptor,
    AssetSourceId, ImportJobId, ImportSettings, SourceHash,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedArtifact {
    pub kind: AssetKind,
    pub cache_key: ArtifactCacheKey,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportValidationRequirement {
    pub code: String,
    pub description: String,
}

impl ImportValidationRequirement {
    pub fn new(code: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPlan {
    pub job_id: ImportJobId,
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub source_hash: Option<SourceHash>,
    pub settings: ImportSettings,
    pub expected_artifacts: Vec<ExpectedArtifact>,
    pub dependencies: Vec<AssetId>,
    pub cache_key: ArtifactCacheKey,
    pub validation_requirements: Vec<ImportValidationRequirement>,
    pub expected_diagnostics: Vec<AssetDiagnosticRecord>,
    pub product_job: Option<ProductJobDescriptor>,
}

impl ImportPlan {
    pub fn deterministic(
        job_id: ImportJobId,
        source: &AssetSourceDescriptor,
        settings: ImportSettings,
        expected_artifact_kind: AssetKind,
    ) -> Self {
        let cache_key = deterministic_cache_key(source, &settings);
        let product_job =
            product_job_descriptor_for_import(job_id, source, &settings, expected_artifact_kind);
        Self {
            job_id,
            asset_id: source.asset_id,
            source_id: source.source_id,
            source_hash: source.source_hash.clone(),
            settings,
            expected_artifacts: vec![ExpectedArtifact {
                kind: expected_artifact_kind,
                cache_key: cache_key.clone(),
                required: true,
            }],
            dependencies: Vec::new(),
            cache_key,
            validation_requirements: vec![ImportValidationRequirement::new(
                "source_hash_matches_plan",
                "source hash must match the deterministic import plan",
            )],
            expected_diagnostics: Vec::new(),
            product_job,
        }
    }

    pub fn with_dependency(mut self, asset_id: AssetId) -> Self {
        self.dependencies.push(asset_id);
        self.dependencies.sort();
        self.dependencies.dedup();
        self
    }
}

pub fn product_job_descriptor_for_import(
    job_id: ImportJobId,
    source: &AssetSourceDescriptor,
    settings: &ImportSettings,
    expected_artifact_kind: AssetKind,
) -> Option<ProductJobDescriptor> {
    if !expected_artifact_kind.is_formed_product() {
        return None;
    }

    let output_product = ProductIdentity::new(job_id.raw());
    let mut descriptor = ProductJobDescriptor::new(
        ProductJobId::new(job_id.raw()),
        ProductKind::new(product_job_kind(settings, expected_artifact_kind)),
        "asset.import_plan",
        output_product,
        ProductScope::non_spatial(format!(
            "asset:{}:source:{}",
            source.asset_id.raw(),
            source.source_id.raw()
        )),
        product_scale_band_for_import(settings),
    );
    descriptor.budget_class = ProductJobBudgetClass::Background;
    descriptor.affinity = ProductJobAffinity::Worker;
    descriptor.determinism = ProductDeterminismClass::DeterministicLocal;
    descriptor.authority_class = ProductAuthorityClass::DeterministicDerived;
    descriptor.failure_policy = ProductJobFailurePolicy::PreserveLastValidWithDiagnostic;
    descriptor.priority = product_job_priority(settings);
    Some(descriptor)
}

fn product_job_kind(settings: &ImportSettings, expected_artifact_kind: AssetKind) -> String {
    format!(
        "asset_import:{}:{}",
        settings.stable_kind_label(),
        asset_kind_label(expected_artifact_kind)
    )
}

fn asset_kind_label(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Scene => "scene",
        AssetKind::Prefab => "prefab",
        AssetKind::SdfGraph => "sdf_graph",
        AssetKind::SdfBrushLayer => "sdf_brush_layer",
        AssetKind::FieldWorldDefinition => "field_world_definition",
        AssetKind::WorldEditLog => "world_edit_log",
        AssetKind::FieldMaterialChannelSet => "field_material_channel_set",
        AssetKind::FormedFieldProduct => "formed_field_product",
        AssetKind::WorldSdfChunkPageArtifact => "world_sdf_chunk_page_artifact",
        AssetKind::ClipmapBrickmapProduct => "clipmap_brickmap_product",
        AssetKind::MaterialGraph => "material_graph",
        AssetKind::Material => "material",
        AssetKind::ProceduralMaterial => "procedural_material",
        AssetKind::ProceduralTexture => "procedural_texture",
        AssetKind::Texture2D => "texture_2d",
        AssetKind::Texture3DVolume => "texture_3d_volume",
        AssetKind::GameplayGraph => "gameplay_graph",
        AssetKind::GameplayRuleTrigger => "gameplay_rule_trigger",
        AssetKind::GameplayAbility => "gameplay_ability",
        AssetKind::GameplayQuest => "gameplay_quest",
        AssetKind::GameplayAtrIrProduct => "gameplay_atr_ir_product",
        AssetKind::GameplayEcsLoweringProduct => "gameplay_ecs_lowering_product",
        AssetKind::ParticleGraph => "particle_graph",
        AssetKind::ParticleEmitter => "particle_emitter",
        AssetKind::PhysicsConfig => "physics_config",
        AssetKind::AnimationClip => "animation_clip",
        AssetKind::AnimationGraph => "animation_graph",
        AssetKind::ProcgenGraph => "procgen_graph",
        AssetKind::UiLayout => "ui_layout",
        AssetKind::UiDefinition => "ui_definition",
        AssetKind::Graph => "graph",
        AssetKind::Script => "script",
        AssetKind::Shader => "shader",
        AssetKind::Theme => "theme",
        AssetKind::Menu => "menu",
        AssetKind::Shortcut => "shortcut",
        AssetKind::WorkspaceDefinition => "workspace_definition",
        AssetKind::EditorDefinition => "editor_definition",
        AssetKind::DiagnosticsCapture => "diagnostics_capture",
        AssetKind::ForeignMeshReferenceSource => "foreign_mesh_reference_source",
        AssetKind::ForeignMeshReferenceArtifact => "foreign_mesh_reference_artifact",
    }
}

fn product_scale_band_for_import(settings: &ImportSettings) -> ProductScaleBand {
    match settings {
        ImportSettings::WorldSdfProduct { scale_band, .. } => match scale_band.as_str() {
            "near" => ProductScaleBand::Near,
            "mid" => ProductScaleBand::Mid,
            "far" => ProductScaleBand::Far,
            "summary" => ProductScaleBand::Summary,
            "collision_strict_query" => ProductScaleBand::CollisionStrictQuery,
            "offline" => ProductScaleBand::Offline,
            "preview" => ProductScaleBand::Preview,
            _ => ProductScaleBand::FamilySpecific,
        },
        ImportSettings::SdfGraph { .. }
        | ImportSettings::SdfBrushLayer { .. }
        | ImportSettings::FieldWorldDefinition { .. }
        | ImportSettings::MaterialGraph { .. }
        | ImportSettings::Material { .. }
        | ImportSettings::ProceduralTexture { .. }
        | ImportSettings::Texture2D { .. }
        | ImportSettings::Texture3DVolume { .. } => ProductScaleBand::Preview,
        _ => ProductScaleBand::FamilySpecific,
    }
}

fn product_job_priority(settings: &ImportSettings) -> i32 {
    match settings {
        ImportSettings::WorldSdfProduct { .. } => 20,
        ImportSettings::SdfGraph { .. }
        | ImportSettings::SdfBrushLayer { .. }
        | ImportSettings::FieldWorldDefinition { .. } => 10,
        _ => 0,
    }
}

pub fn deterministic_cache_key(
    source: &AssetSourceDescriptor,
    settings: &ImportSettings,
) -> ArtifactCacheKey {
    let hash = source
        .source_hash
        .as_ref()
        .map(|hash| format!("{}:{}", hash.algorithm, hash.value))
        .unwrap_or_else(|| "unhashed".to_string());
    ArtifactCacheKey::new(format!(
        "asset-{}-source-{}-{}-{}",
        source.asset_id.raw(),
        source.source_id.raw(),
        settings.stable_kind_label(),
        hash
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetKind, FieldProductResolution, ImportSettings, SourceHash, asset_id, asset_source_id,
        import_job_id,
    };

    #[test]
    fn import_plan_cache_key_is_deterministic() {
        let source = AssetSourceDescriptor::new(
            asset_source_id(2),
            asset_id(1),
            AssetKind::SdfGraph,
            "assets/fields/test.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let settings = ImportSettings::SdfGraph {
            resolution: FieldProductResolution::new(64, 64, 1),
        };

        let first = ImportPlan::deterministic(
            import_job_id(3),
            &source,
            settings.clone(),
            AssetKind::FormedFieldProduct,
        );
        let second = ImportPlan::deterministic(
            import_job_id(3),
            &source,
            settings,
            AssetKind::FormedFieldProduct,
        );

        assert_eq!(first.cache_key, second.cache_key);
        assert_eq!(first.expected_artifacts, second.expected_artifacts);
        assert_eq!(first.product_job, second.product_job);
        assert_eq!(
            first
                .product_job
                .as_ref()
                .expect("formed product imports should declare a product job")
                .output_products,
            vec![product::ProductIdentity::new(3)]
        );
    }
}
