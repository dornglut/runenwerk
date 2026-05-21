use super::{PreparedFeatureContributionDiagnostic, PreparedRegisteredFeaturePayload};
use crate::plugins::render::api::ids::RenderFeatureId;
use crate::plugins::render::features::{
    CAVE_INTERIOR_RENDER_FEATURE_ID, DEFORMATION_RENDER_FEATURE_ID, DETAIL_RENDER_FEATURE_ID,
    FeatureContributionStatus, FeatureFallbackPolicy, MATERIAL_RENDER_FEATURE_ID,
    PROCEDURAL_WORLD_RENDER_FEATURE_ID, PreparedUiFrameContribution, SCENE_ROUTE_RENDER_FEATURE_ID,
    UI_RENDER_FEATURE_ID, WIND_FIELDS_RENDER_FEATURE_ID, WORLD_DRAW_RENDER_FEATURE_ID,
};
use spatial::ChunkId;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct PreparedFrameContributions {
    pub by_feature: BTreeMap<RenderFeatureId, PreparedFeatureContribution>,
    pub diagnostics: Vec<PreparedFeatureContributionDiagnostic>,
}

impl PreparedFrameContributions {
    pub fn feature(&self, id: &RenderFeatureId) -> Option<&PreparedFeatureContribution> {
        self.by_feature.get(id)
    }

    pub fn insert(&mut self, id: RenderFeatureId, contribution: PreparedFeatureContribution) {
        self.by_feature.insert(id, contribution);
    }

    pub fn push_diagnostic(&mut self, diagnostic: PreparedFeatureContributionDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[PreparedFeatureContributionDiagnostic] {
        &self.diagnostics
    }

    pub fn insert_missing(&mut self, id: RenderFeatureId, fallback_policy: FeatureFallbackPolicy) {
        self.by_feature
            .entry(id)
            .or_insert_with(|| PreparedFeatureContribution {
                status: FeatureContributionStatus::Missing,
                fallback_policy,
                payload: PreparedFeaturePayload::Empty,
            });
    }

    pub fn insert_ui(
        &mut self,
        payload: PreparedUiFrameContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            UI_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Ui(payload),
            },
        );
    }

    pub fn insert_scene_route(
        &mut self,
        world_scene_label: String,
        overlay_scene_label: String,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            SCENE_ROUTE_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::SceneRoute(PreparedSceneRouteContribution {
                    world_scene_label,
                    overlay_scene_label,
                }),
            },
        );
    }

    pub fn insert_draw(
        &mut self,
        payload: PreparedDrawFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            WORLD_DRAW_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Draw(payload),
            },
        );
    }

    pub fn insert_world(
        &mut self,
        payload: PreparedWorldFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            WORLD_DRAW_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::World(payload),
            },
        );
    }

    pub fn insert_caves(
        &mut self,
        payload: PreparedCaveFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            CAVE_INTERIOR_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Caves(payload),
            },
        );
    }

    pub fn insert_detail(
        &mut self,
        payload: PreparedDetailFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            DETAIL_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Detail(payload),
            },
        );
    }

    pub fn insert_procedural_world(
        &mut self,
        payload: PreparedProceduralWorldFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            PROCEDURAL_WORLD_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::ProceduralWorld(payload),
            },
        );
    }

    pub fn insert_wind_fields(
        &mut self,
        payload: PreparedWindFieldFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            WIND_FIELDS_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::WindFields(payload),
            },
        );
    }

    pub fn insert_material(
        &mut self,
        payload: PreparedMaterialFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            MATERIAL_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Material(payload),
            },
        );
    }

    pub fn insert_deformation(
        &mut self,
        payload: PreparedDeformationFeatureContribution,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            DEFORMATION_RENDER_FEATURE_ID,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Deformation(payload),
            },
        );
    }

    pub fn insert_registered(
        &mut self,
        id: RenderFeatureId,
        payload: PreparedRegisteredFeaturePayload,
        status: FeatureContributionStatus,
        fallback_policy: FeatureFallbackPolicy,
    ) {
        self.insert(
            id,
            PreparedFeatureContribution {
                status,
                fallback_policy,
                payload: PreparedFeaturePayload::Registered(payload),
            },
        );
    }

    pub fn ui(&self) -> Option<&PreparedUiFrameContribution> {
        let contribution = self.by_feature.get(&UI_RENDER_FEATURE_ID)?;
        match contribution.payload {
            PreparedFeaturePayload::Ui(ref value)
                if !matches!(
                    contribution.status,
                    FeatureContributionStatus::Disabled | FeatureContributionStatus::Missing
                ) =>
            {
                Some(value)
            }
            _ => None,
        }
    }

    pub fn scene_route_labels(&self) -> Option<(&str, &str)> {
        let contribution = self.by_feature.get(&SCENE_ROUTE_RENDER_FEATURE_ID)?;
        match contribution.payload {
            PreparedFeaturePayload::SceneRoute(ref value)
                if !matches!(
                    contribution.status,
                    FeatureContributionStatus::Disabled | FeatureContributionStatus::Missing
                ) =>
            {
                Some((
                    value.world_scene_label.as_str(),
                    value.overlay_scene_label.as_str(),
                ))
            }
            _ => None,
        }
    }

    pub fn feature_gate(&self, id: &RenderFeatureId) -> Option<PreparedFeatureGate> {
        let contribution = self.by_feature.get(id)?;
        Some(contribution.gate())
    }
}

#[derive(Debug, Clone)]
pub struct PreparedFeatureContribution {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedFeaturePayload,
}

impl Default for PreparedFeatureContribution {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Ready,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedFeaturePayload::Empty,
        }
    }
}

impl PreparedFeatureContribution {
    pub fn gate(&self) -> PreparedFeatureGate {
        PreparedFeatureGate {
            status: self.status,
            fallback_policy: self.fallback_policy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedFeatureGate {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
}

impl Default for PreparedFeatureGate {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum PreparedFeaturePayload {
    #[default]
    Empty,
    Ui(PreparedUiFrameContribution),
    SceneRoute(PreparedSceneRouteContribution),
    Draw(PreparedDrawFeatureContribution),
    World(PreparedWorldFeatureContribution),
    Caves(PreparedCaveFeatureContribution),
    Detail(PreparedDetailFeatureContribution),
    ProceduralWorld(PreparedProceduralWorldFeatureContribution),
    WindFields(PreparedWindFieldFeatureContribution),
    Material(PreparedMaterialFeatureContribution),
    Deformation(PreparedDeformationFeatureContribution),
    Registered(PreparedRegisteredFeaturePayload),
}

#[derive(Debug, Clone, Default)]
pub struct PreparedSceneRouteContribution {
    pub world_scene_label: String,
    pub overlay_scene_label: String,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDrawFeatureContribution {
    pub batches: Vec<PreparedDrawBatch>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedWorldFeatureContribution {
    pub visible_chunks: Vec<PreparedWorldChunkContribution>,
    pub residency_intents: Vec<PreparedWorldResidencyIntent>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedWorldChunkContribution {
    pub chunk_id: ChunkId,
    pub chunk_revision: u64,
    pub chunk_generation: u64,
    pub draw_batch_ref: PreparedWorldDrawBatchRef,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct PreparedWorldDrawBatchRef {
    pub chunk_id: ChunkId,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedWorldResidencyIntent {
    pub chunk_id: ChunkId,
    pub priority: i32,
    pub hard_pin: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedCaveFeatureContribution {
    pub visible_sector_ids: Vec<u32>,
    pub scoped_light_volume_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDetailFeatureContribution {
    pub cells: Vec<PreparedDetailCellContribution>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDetailCellContribution {
    pub cell_id: String,
    pub chunk_id: ChunkId,
    pub instance_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedProceduralWorldFeatureContribution {
    pub overlays: Vec<PreparedProceduralOverlayContribution>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedProceduralOverlayContribution {
    pub overlay_id: String,
    pub source_revision: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedWindFieldFeatureContribution {
    pub fields: Vec<PreparedWindFieldContribution>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedWindFieldContribution {
    pub field_id: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDrawBatch {
    pub batch_id: String,
    pub mesh_ref: String,
    pub material_ref: String,
    pub instance_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreparedModelMeshMaterialSourceIdentity {
    pub asset_id: u64,
    pub source_id: u64,
    pub source_revision_id: Option<u64>,
    pub source_revision: Option<String>,
}

impl PreparedModelMeshMaterialSourceIdentity {
    pub fn new(asset_id: u64, source_id: u64) -> Result<Self, PreparedModelMeshMaterialError> {
        if asset_id == 0 {
            return Err(PreparedModelMeshMaterialError::new(
                "model/mesh material source asset id must be non-zero",
            ));
        }
        if source_id == 0 {
            return Err(PreparedModelMeshMaterialError::new(
                "model/mesh material source id must be non-zero",
            ));
        }
        Ok(Self {
            asset_id,
            source_id,
            source_revision_id: None,
            source_revision: None,
        })
    }

    pub fn with_source_revision_id(mut self, source_revision_id: u64) -> Self {
        self.source_revision_id = (source_revision_id != 0).then_some(source_revision_id);
        self
    }

    pub fn with_source_revision(mut self, source_revision: impl Into<String>) -> Self {
        let source_revision = source_revision.into();
        if !source_revision.is_empty() {
            self.source_revision = Some(source_revision);
        }
        self
    }

    pub fn identity_key(&self) -> String {
        format!(
            "asset={}:source={}:source_revision_id={}:source_revision={}",
            self.asset_id,
            self.source_id,
            self.source_revision_id
                .map(|revision| revision.to_string())
                .unwrap_or_default(),
            self.source_revision.as_deref().unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreparedModelMeshMaterialRegionIdentity {
    pub source: PreparedModelMeshMaterialSourceIdentity,
    pub region_key: String,
}

impl PreparedModelMeshMaterialRegionIdentity {
    pub fn new(
        source: PreparedModelMeshMaterialSourceIdentity,
        region_key: impl Into<String>,
    ) -> Result<Self, PreparedModelMeshMaterialError> {
        let region_key = region_key.into();
        validate_model_mesh_material_region_key(&region_key)?;
        Ok(Self { source, region_key })
    }

    pub fn identity_key(&self) -> String {
        format!("{}:region={}", self.source.identity_key(), self.region_key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedModelMeshMaterialSelection {
    pub surface: PreparedModelMeshMaterialRegionIdentity,
    pub requested_material_slot_id: u64,
    pub resolved_material_slot_id: u64,
    pub material_table_index: u32,
    pub used_default_fallback: bool,
}

impl PreparedModelMeshMaterialSelection {
    pub fn new(
        surface: PreparedModelMeshMaterialRegionIdentity,
        requested_material_slot_id: u64,
        resolved_material_slot_id: u64,
        material_table_index: u32,
        used_default_fallback: bool,
    ) -> Result<Self, PreparedModelMeshMaterialError> {
        if requested_material_slot_id == 0 {
            return Err(PreparedModelMeshMaterialError::new(
                "requested model/mesh material slot id must be non-zero",
            ));
        }
        if resolved_material_slot_id == 0 {
            return Err(PreparedModelMeshMaterialError::new(
                "resolved model/mesh material slot id must be non-zero",
            ));
        }
        Ok(Self {
            surface,
            requested_material_slot_id,
            resolved_material_slot_id,
            material_table_index,
            used_default_fallback,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedModelMeshMaterialError {
    message: String,
}

impl PreparedModelMeshMaterialError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PreparedModelMeshMaterialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for PreparedModelMeshMaterialError {}

fn validate_model_mesh_material_region_key(
    value: &str,
) -> Result<(), PreparedModelMeshMaterialError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PreparedModelMeshMaterialError::new(
            "model/mesh material region key must not be empty",
        ));
    }
    if trimmed != value {
        return Err(PreparedModelMeshMaterialError::new(
            "model/mesh material region key must already be normalized",
        ));
    }
    if is_transient_model_mesh_material_region_key(trimmed) {
        return Err(PreparedModelMeshMaterialError::new(format!(
            "model/mesh material region key must not use transient renderer identity '{trimmed}'"
        )));
    }
    Ok(())
}

fn is_transient_model_mesh_material_region_key(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    [
        "renderable_index:",
        "draw_order:",
        "mesh_table_index:",
        "residency_slot:",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

#[derive(Debug, Clone, Default)]
pub struct PreparedMaterialFeatureContribution {
    pub instances: Vec<PreparedMaterialInstanceInput>,
    pub binding_table: PreparedMaterialBindingTable,
    pub scene_bundle: Option<PreparedSceneMaterialBundle>,
    pub model_mesh_material_selections: Vec<PreparedModelMeshMaterialSelection>,
}

pub const PREPARED_MATERIAL_TEXTURE_RESOURCE_PORTABLE_SLOT_LIMIT: usize = 128;

impl PreparedMaterialFeatureContribution {
    pub fn validate_portable_limits(&self) -> Result<(), PreparedMaterialBindingTableError> {
        let texture_slots = self
            .instances
            .iter()
            .map(|instance| instance.texture_bindings.len())
            .sum::<usize>();
        if texture_slots > PREPARED_MATERIAL_TEXTURE_RESOURCE_PORTABLE_SLOT_LIMIT {
            return Err(PreparedMaterialBindingTableError::new(format!(
                "material texture binding table has {texture_slots} resource slots, portable limit is {PREPARED_MATERIAL_TEXTURE_RESOURCE_PORTABLE_SLOT_LIMIT}"
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        for instance in &self.instances {
            for binding in &instance.texture_bindings {
                if binding.resource_slot_index as usize
                    >= PREPARED_MATERIAL_TEXTURE_RESOURCE_PORTABLE_SLOT_LIMIT
                {
                    return Err(PreparedMaterialBindingTableError::new(format!(
                        "material texture resource slot {} exceeds portable limit {}",
                        binding.resource_slot_index,
                        PREPARED_MATERIAL_TEXTURE_RESOURCE_PORTABLE_SLOT_LIMIT
                    )));
                }
                if !seen.insert(binding.resource_slot_index) {
                    return Err(PreparedMaterialBindingTableError::new(format!(
                        "duplicate material texture resource slot {}",
                        binding.resource_slot_index
                    )));
                }
            }
        }
        let mut seen_model_mesh_regions = std::collections::BTreeSet::new();
        for selection in &self.model_mesh_material_selections {
            if selection.material_table_index as usize
                >= PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT
            {
                return Err(PreparedMaterialBindingTableError::new(format!(
                    "model/mesh material table slot {} exceeds portable limit {}",
                    selection.material_table_index,
                    PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT
                )));
            }
            if selection.requested_material_slot_id == 0 {
                return Err(PreparedMaterialBindingTableError::new(
                    "requested model/mesh material slot id must be non-zero",
                ));
            }
            if selection.resolved_material_slot_id == 0 {
                return Err(PreparedMaterialBindingTableError::new(
                    "resolved model/mesh material slot id must be non-zero",
                ));
            }
            if !seen_model_mesh_regions.insert(selection.surface.identity_key()) {
                return Err(PreparedMaterialBindingTableError::new(format!(
                    "duplicate model/mesh material region '{}'",
                    selection.surface.identity_key()
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedSceneMaterialBundle {
    pub shader_artifact_id: String,
    pub shader_cache_key: String,
    pub shader_path: String,
    pub shader_identity: String,
    pub material_table_identity: String,
    pub resource_layout_identity: String,
}

impl PreparedSceneMaterialBundle {
    pub fn new(
        shader_artifact_id: impl Into<String>,
        shader_cache_key: impl Into<String>,
        shader_path: impl Into<String>,
        shader_identity: impl Into<String>,
        material_table_identity: impl Into<String>,
    ) -> Self {
        Self::new_with_resource_layout(
            shader_artifact_id,
            shader_cache_key,
            shader_path,
            shader_identity,
            material_table_identity,
            "",
        )
    }

    pub fn new_with_resource_layout(
        shader_artifact_id: impl Into<String>,
        shader_cache_key: impl Into<String>,
        shader_path: impl Into<String>,
        shader_identity: impl Into<String>,
        material_table_identity: impl Into<String>,
        resource_layout_identity: impl Into<String>,
    ) -> Self {
        Self {
            shader_artifact_id: shader_artifact_id.into(),
            shader_cache_key: shader_cache_key.into(),
            shader_path: shader_path.into(),
            shader_identity: shader_identity.into(),
            material_table_identity: material_table_identity.into(),
            resource_layout_identity: resource_layout_identity.into(),
        }
    }
}

pub const PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMaterialBindingTable {
    pub backend: PreparedMaterialBindingTableBackend,
    pub slots: Vec<PreparedMaterialBindingSlot>,
}

impl Default for PreparedMaterialBindingTable {
    fn default() -> Self {
        Self {
            backend: PreparedMaterialBindingTableBackend::FixedCapacityArray {
                capacity: PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT,
            },
            slots: Vec::new(),
        }
    }
}

impl PreparedMaterialBindingTable {
    pub fn fixed_capacity(
        slots: impl IntoIterator<Item = PreparedMaterialBindingSlot>,
    ) -> Result<Self, PreparedMaterialBindingTableError> {
        let slots = slots.into_iter().collect::<Vec<_>>();
        if slots.len() > PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT {
            return Err(PreparedMaterialBindingTableError::new(format!(
                "material binding table has {} slots, portable limit is {}",
                slots.len(),
                PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT
            )));
        }
        let mut slot_indices = std::collections::BTreeSet::new();
        for slot in &slots {
            if slot.slot_index as usize >= PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT {
                return Err(PreparedMaterialBindingTableError::new(format!(
                    "material binding slot {} exceeds portable limit {}",
                    slot.slot_index, PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT
                )));
            }
            if !slot_indices.insert(slot.slot_index) {
                return Err(PreparedMaterialBindingTableError::new(format!(
                    "duplicate material binding slot {}",
                    slot.slot_index
                )));
            }
        }
        Ok(Self {
            backend: PreparedMaterialBindingTableBackend::FixedCapacityArray {
                capacity: PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT,
            },
            slots,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreparedMaterialBindingTableBackend {
    FixedCapacityArray { capacity: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMaterialBindingSlot {
    pub slot_index: u32,
    pub material_instance_id: String,
    pub formed_material_artifact_id: String,
    pub shader_artifact_id: String,
    pub material_cache_key: String,
    pub shader_cache_key: String,
    pub prior_valid: bool,
}

impl PreparedMaterialBindingSlot {
    pub fn new(
        slot_index: u32,
        material_instance_id: impl Into<String>,
        formed_material_artifact_id: impl Into<String>,
        shader_artifact_id: impl Into<String>,
        material_cache_key: impl Into<String>,
        shader_cache_key: impl Into<String>,
    ) -> Self {
        Self {
            slot_index,
            material_instance_id: material_instance_id.into(),
            formed_material_artifact_id: formed_material_artifact_id.into(),
            shader_artifact_id: shader_artifact_id.into(),
            material_cache_key: material_cache_key.into(),
            shader_cache_key: shader_cache_key.into(),
            prior_valid: false,
        }
    }

    pub fn with_prior_valid(mut self, prior_valid: bool) -> Self {
        self.prior_valid = prior_valid;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMaterialBindingTableError {
    message: String,
}

impl PreparedMaterialBindingTableError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PreparedMaterialBindingTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for PreparedMaterialBindingTableError {}

const MATERIAL_PARAMETER_PAYLOAD_FORMAT_V1: &str = "runenwerk.material-parameters.v1";
pub const PREPARED_MATERIAL_PARAMETER_PAYLOAD_V1_MAX_PARAMETERS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedMaterialParameterProfile {
    PbrPreview,
    RenderMaterial,
}

impl Default for PreparedMaterialParameterProfile {
    fn default() -> Self {
        Self::PbrPreview
    }
}

impl PreparedMaterialParameterProfile {
    pub fn label(self) -> &'static str {
        match self {
            Self::PbrPreview => "pbr_preview",
            Self::RenderMaterial => "render_material",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "pbr_preview" => Some(Self::PbrPreview),
            "render_material" => Some(Self::RenderMaterial),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedMaterialOutputTarget {
    PbrPreview,
    FieldMaterialChannel,
    RenderMaterial,
}

impl Default for PreparedMaterialOutputTarget {
    fn default() -> Self {
        Self::PbrPreview
    }
}

impl PreparedMaterialOutputTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::PbrPreview => "pbr_preview",
            Self::FieldMaterialChannel => "field_material_channel",
            Self::RenderMaterial => "render_material",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "pbr_preview" => Some(Self::PbrPreview),
            "field_material_channel" => Some(Self::FieldMaterialChannel),
            "render_material" => Some(Self::RenderMaterial),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedMaterialParameterKind {
    Scalar,
    Vector2,
    Vector3,
    Vector4,
    Texture2D,
    Texture3D,
}

impl Default for PreparedMaterialParameterKind {
    fn default() -> Self {
        Self::Scalar
    }
}

impl PreparedMaterialParameterKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Vector2 => "vector2",
            Self::Vector3 => "vector3",
            Self::Vector4 => "vector4",
            Self::Texture2D => "texture2d",
            Self::Texture3D => "texture3d",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "scalar" => Some(Self::Scalar),
            "vector2" => Some(Self::Vector2),
            "vector3" => Some(Self::Vector3),
            "vector4" => Some(Self::Vector4),
            "texture2d" => Some(Self::Texture2D),
            "texture3d" => Some(Self::Texture3D),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreparedMaterialParameterInput {
    pub key: String,
    pub kind: PreparedMaterialParameterKind,
}

impl PreparedMaterialParameterInput {
    pub fn new(key: impl Into<String>, kind: PreparedMaterialParameterKind) -> Self {
        Self {
            key: key.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreparedMaterialParameterPayloadV1 {
    pub profile: PreparedMaterialParameterProfile,
    pub output_target: PreparedMaterialOutputTarget,
    pub parameters: Vec<PreparedMaterialParameterInput>,
}

impl PreparedMaterialParameterPayloadV1 {
    pub fn new(
        profile: PreparedMaterialParameterProfile,
        output_target: PreparedMaterialOutputTarget,
        parameters: impl IntoIterator<Item = PreparedMaterialParameterInput>,
    ) -> Self {
        let mut parameters = parameters.into_iter().collect::<Vec<_>>();
        parameters.sort_by(|left, right| {
            left.key
                .cmp(&right.key)
                .then(left.kind.label().cmp(right.kind.label()))
        });
        Self {
            profile,
            output_target,
            parameters,
        }
    }

    pub fn encode_v1(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        push_payload_field(&mut payload, "format", MATERIAL_PARAMETER_PAYLOAD_FORMAT_V1);
        push_payload_field(&mut payload, "version", "1");
        push_payload_field(&mut payload, "profile", self.profile.label());
        push_payload_field(&mut payload, "output_target", self.output_target.label());
        push_payload_field(
            &mut payload,
            "parameter_count",
            &self.parameters.len().to_string(),
        );
        for parameter in &self.parameters {
            push_payload_field(&mut payload, "parameter_key", &parameter.key);
            push_payload_field(&mut payload, "parameter_kind", parameter.kind.label());
        }
        payload
    }

    pub fn decode_v1(bytes: &[u8]) -> Result<Self, PreparedMaterialParameterPayloadDecodeError> {
        let mut cursor = PayloadFieldCursor::new(bytes);
        cursor.expect_field("format", MATERIAL_PARAMETER_PAYLOAD_FORMAT_V1)?;
        cursor.expect_field("version", "1")?;
        let profile_label = cursor.required_value("profile")?;
        let profile = PreparedMaterialParameterProfile::from_label(&profile_label)
            .ok_or_else(|| PreparedMaterialParameterPayloadDecodeError::new("unknown profile"))?;
        let output_target_label = cursor.required_value("output_target")?;
        let output_target = PreparedMaterialOutputTarget::from_label(&output_target_label)
            .ok_or_else(|| {
                PreparedMaterialParameterPayloadDecodeError::new("unknown output target")
            })?;
        let parameter_count = cursor
            .required_value("parameter_count")?
            .parse::<usize>()
            .map_err(|_| {
                PreparedMaterialParameterPayloadDecodeError::new("invalid parameter count")
            })?;
        if parameter_count > PREPARED_MATERIAL_PARAMETER_PAYLOAD_V1_MAX_PARAMETERS {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(
                "parameter count exceeds v1 limit",
            ));
        }
        let mut parameters = Vec::with_capacity(parameter_count);
        for _ in 0..parameter_count {
            let key = cursor.required_value("parameter_key")?;
            let kind_label = cursor.required_value("parameter_kind")?;
            let kind = PreparedMaterialParameterKind::from_label(&kind_label).ok_or_else(|| {
                PreparedMaterialParameterPayloadDecodeError::new("unknown parameter kind")
            })?;
            parameters.push(PreparedMaterialParameterInput::new(key, kind));
        }
        cursor.expect_end()?;
        Ok(Self::new(profile, output_target, parameters))
    }

    pub fn encoded_len(&self) -> usize {
        self.encode_v1().len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMaterialParameterPayloadDecodeError {
    message: String,
}

impl PreparedMaterialParameterPayloadDecodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PreparedMaterialParameterPayloadDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PreparedMaterialParameterPayloadDecodeError {}

fn push_payload_field(payload: &mut Vec<u8>, label: &str, value: &str) {
    payload.extend_from_slice(label.as_bytes());
    payload.push(b'=');
    payload.extend_from_slice(value.len().to_string().as_bytes());
    payload.push(b':');
    payload.extend_from_slice(value.as_bytes());
    payload.push(b'\n');
}

struct PayloadFieldCursor<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> PayloadFieldCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    fn required_value(
        &mut self,
        expected_label: &str,
    ) -> Result<String, PreparedMaterialParameterPayloadDecodeError> {
        let Some((label, value)) = self.next_field()? else {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(format!(
                "missing {expected_label}"
            )));
        };
        if label != expected_label {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(format!(
                "expected {expected_label}, got {label}"
            )));
        }
        Ok(value.clone())
    }

    fn expect_field(
        &mut self,
        expected_label: &str,
        expected_value: &str,
    ) -> Result<(), PreparedMaterialParameterPayloadDecodeError> {
        let value = self.required_value(expected_label)?;
        if value != expected_value {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(format!(
                "unsupported {expected_label}"
            )));
        }
        Ok(())
    }

    fn expect_end(&self) -> Result<(), PreparedMaterialParameterPayloadDecodeError> {
        if self.index == self.bytes.len() {
            Ok(())
        } else {
            Err(PreparedMaterialParameterPayloadDecodeError::new(
                "trailing payload fields",
            ))
        }
    }

    fn next_field(
        &mut self,
    ) -> Result<Option<(String, String)>, PreparedMaterialParameterPayloadDecodeError> {
        if self.index >= self.bytes.len() {
            return Ok(None);
        }
        let label_start = self.index;
        while self.index < self.bytes.len() && self.bytes[self.index] != b'=' {
            self.index += 1;
        }
        if self.index == self.bytes.len() || self.index == label_start {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(
                "invalid payload field label",
            ));
        }
        let label = std::str::from_utf8(&self.bytes[label_start..self.index])
            .map_err(|_| PreparedMaterialParameterPayloadDecodeError::new("non-utf8 label"))?
            .to_string();
        self.index += 1;

        let length_start = self.index;
        while self.index < self.bytes.len() && self.bytes[self.index] != b':' {
            if !self.bytes[self.index].is_ascii_digit() {
                return Err(PreparedMaterialParameterPayloadDecodeError::new(
                    "invalid payload field length",
                ));
            }
            self.index += 1;
        }
        if self.index == self.bytes.len() || self.index == length_start {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(
                "missing payload field length",
            ));
        }
        let length = std::str::from_utf8(&self.bytes[length_start..self.index])
            .map_err(|_| PreparedMaterialParameterPayloadDecodeError::new("non-utf8 length"))?
            .parse::<usize>()
            .map_err(|_| {
                PreparedMaterialParameterPayloadDecodeError::new("invalid payload field length")
            })?;
        self.index += 1;

        let value_end = self.index.checked_add(length).ok_or_else(|| {
            PreparedMaterialParameterPayloadDecodeError::new("payload field length overflow")
        })?;
        if value_end > self.bytes.len() {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(
                "payload field exceeds input",
            ));
        }
        let value = std::str::from_utf8(&self.bytes[self.index..value_end])
            .map_err(|_| PreparedMaterialParameterPayloadDecodeError::new("non-utf8 value"))?
            .to_string();
        self.index = value_end;
        if self.index >= self.bytes.len() || self.bytes[self.index] != b'\n' {
            return Err(PreparedMaterialParameterPayloadDecodeError::new(
                "payload field missing terminator",
            ));
        }
        self.index += 1;
        Ok(Some((label, value)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct PreparedMaterialInstanceInput {
    pub material_instance_id: String,
    pub specialization_key_fragment: String,
    pub parameter_payload: PreparedMaterialParameterPayloadV1,
    pub texture_bindings: Vec<PreparedMaterialTextureBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMaterialTextureBinding {
    pub node_id: u64,
    pub binding_key: String,
    pub resource_slot_index: u32,
    pub artifact_id: String,
    pub artifact_path: String,
    pub texture_kind: PreparedMaterialTextureKind,
    pub extent_width: u32,
    pub extent_height: u32,
    pub extent_depth: u32,
    pub cache_key: String,
    pub sampler_policy: String,
    pub texture_dimension: String,
    pub residency_identity: String,
    pub artifact_revision: String,
    pub descriptor_hash: String,
    pub pixel_format: String,
    pub supercompression: String,
    pub container_byte_length: Option<u64>,
}

impl PreparedMaterialTextureBinding {
    pub fn new(
        node_id: u64,
        binding_key: impl Into<String>,
        artifact_id: impl Into<String>,
        artifact_path: impl Into<String>,
        texture_kind: PreparedMaterialTextureKind,
        cache_key: impl Into<String>,
    ) -> Self {
        Self {
            node_id,
            binding_key: binding_key.into(),
            resource_slot_index: 0,
            artifact_id: artifact_id.into(),
            artifact_path: artifact_path.into(),
            texture_kind,
            extent_width: 1,
            extent_height: 1,
            extent_depth: match texture_kind {
                PreparedMaterialTextureKind::Texture2D => 1,
                PreparedMaterialTextureKind::Texture3D => 2,
            },
            cache_key: cache_key.into(),
            sampler_policy: "linear_repeat".to_string(),
            texture_dimension: String::new(),
            residency_identity: String::new(),
            artifact_revision: String::new(),
            descriptor_hash: String::new(),
            pixel_format: "rgba8_unorm".to_string(),
            supercompression: "none".to_string(),
            container_byte_length: None,
        }
    }

    pub fn with_resource_slot_index(mut self, resource_slot_index: u32) -> Self {
        self.resource_slot_index = resource_slot_index;
        self
    }

    pub fn with_texture_dimension(mut self, texture_dimension: impl Into<String>) -> Self {
        self.texture_dimension = texture_dimension.into();
        self
    }

    pub fn with_extent(mut self, width: u32, height: u32, depth: u32) -> Self {
        self.extent_width = width;
        self.extent_height = height;
        self.extent_depth = depth;
        self
    }

    pub fn with_residency_identity(mut self, residency_identity: impl Into<String>) -> Self {
        self.residency_identity = residency_identity.into();
        self
    }

    pub fn with_artifact_revision(mut self, artifact_revision: impl Into<String>) -> Self {
        self.artifact_revision = artifact_revision.into();
        self
    }

    pub fn with_descriptor_hash(mut self, descriptor_hash: impl Into<String>) -> Self {
        self.descriptor_hash = descriptor_hash.into();
        self
    }

    pub fn with_ktx2_contract(
        mut self,
        pixel_format: impl Into<String>,
        supercompression: impl Into<String>,
        container_byte_length: Option<u64>,
    ) -> Self {
        self.pixel_format = pixel_format.into();
        self.supercompression = supercompression.into();
        self.container_byte_length = container_byte_length;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreparedMaterialTextureKind {
    Texture2D,
    Texture3D,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDeformationFeatureContribution {
    pub streams: Vec<PreparedDeformationStream>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDeformationStream {
    pub stream_id: String,
    pub input_pose_ref: String,
    pub output_buffer_ref: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_parameter_payload_round_trips_canonical_v1() {
        let payload = PreparedMaterialParameterPayloadV1::new(
            PreparedMaterialParameterProfile::RenderMaterial,
            PreparedMaterialOutputTarget::RenderMaterial,
            [
                PreparedMaterialParameterInput::new(
                    "roughness",
                    PreparedMaterialParameterKind::Scalar,
                ),
                PreparedMaterialParameterInput::new(
                    "base_color",
                    PreparedMaterialParameterKind::Vector4,
                ),
            ],
        );

        let encoded = payload.encode_v1();
        let decoded =
            PreparedMaterialParameterPayloadV1::decode_v1(&encoded).expect("payload decodes");

        assert_eq!(decoded, payload);
        assert!(String::from_utf8_lossy(&encoded).contains("version=1:1"));
        assert!(!String::from_utf8_lossy(&encoded).contains("Vector4"));
    }

    #[test]
    fn material_parameter_payload_rejects_unknown_version() {
        let payload = PreparedMaterialParameterPayloadV1::new(
            PreparedMaterialParameterProfile::PbrPreview,
            PreparedMaterialOutputTarget::PbrPreview,
            std::iter::empty::<PreparedMaterialParameterInput>(),
        );
        let mut encoded = payload.encode_v1();
        let original = b"version=1:1\n";
        let replacement = b"version=1:2\n";
        let position = encoded
            .windows(original.len())
            .position(|window| window == original)
            .expect("version field should exist");
        encoded.splice(
            position..position + original.len(),
            replacement.iter().copied(),
        );

        let error = PreparedMaterialParameterPayloadV1::decode_v1(&encoded)
            .expect_err("unknown version should fail");

        assert!(error.to_string().contains("unsupported version"));
    }

    #[test]
    fn material_parameter_payload_rejects_oversized_parameter_count_before_allocation() {
        let mut encoded = Vec::new();
        push_payload_field(&mut encoded, "format", MATERIAL_PARAMETER_PAYLOAD_FORMAT_V1);
        push_payload_field(&mut encoded, "version", "1");
        push_payload_field(&mut encoded, "profile", "pbr_preview");
        push_payload_field(&mut encoded, "output_target", "pbr_preview");
        push_payload_field(&mut encoded, "parameter_count", "999999999");

        let error = PreparedMaterialParameterPayloadV1::decode_v1(&encoded)
            .expect_err("oversized parameter count should fail before allocation");

        assert!(error.to_string().contains("exceeds v1 limit"));
    }

    #[test]
    fn material_binding_table_rejects_slots_above_portable_limit() {
        let error =
            PreparedMaterialBindingTable::fixed_capacity([PreparedMaterialBindingSlot::new(
                PREPARED_MATERIAL_BINDING_TABLE_PORTABLE_SLOT_LIMIT as u32,
                "material.product.1",
                "artifact.material.1",
                "artifact.shader.2",
                "material-cache",
                "shader-cache",
            )])
            .expect_err("slot above portable limit should fail");

        assert!(error.to_string().contains("exceeds portable limit"));
    }

    #[test]
    fn material_feature_rejects_duplicate_texture_resource_slots() {
        let duplicate = PreparedMaterialTextureBinding::new(
            1,
            "texture_ref",
            "artifact.1",
            ".runenwerk/artifacts/texture.ktx2",
            PreparedMaterialTextureKind::Texture2D,
            "texture-cache",
        );
        let contribution = PreparedMaterialFeatureContribution {
            instances: vec![PreparedMaterialInstanceInput {
                material_instance_id: "material.product.1".to_string(),
                specialization_key_fragment: "material.first_slice".to_string(),
                parameter_payload: PreparedMaterialParameterPayloadV1::default(),
                texture_bindings: vec![duplicate.clone(), duplicate],
            }],
            binding_table: PreparedMaterialBindingTable::default(),
            scene_bundle: None,
            model_mesh_material_selections: Vec::new(),
        };

        let error = contribution
            .validate_portable_limits()
            .expect_err("duplicate resource slots should be rejected");

        assert!(
            error
                .to_string()
                .contains("duplicate material texture resource slot")
        );
    }

    #[test]
    fn model_mesh_surface_material_selection_uses_typed_identity() {
        let source = PreparedModelMeshMaterialSourceIdentity::new(42, 84)
            .expect("source-backed model/mesh identity should be valid")
            .with_source_revision_id(2)
            .with_source_revision("sha256:abc");
        let surface =
            PreparedModelMeshMaterialRegionIdentity::new(source, "source_material_slot:0")
                .expect("source-backed model/mesh identity should be valid");
        let selection = PreparedModelMeshMaterialSelection::new(surface.clone(), 2, 2, 1, false)
            .expect("source-backed model/mesh material selection should form");
        let contribution = PreparedMaterialFeatureContribution {
            instances: Vec::new(),
            binding_table: PreparedMaterialBindingTable::default(),
            scene_bundle: None,
            model_mesh_material_selections: vec![selection.clone()],
        };

        contribution
            .validate_portable_limits()
            .expect("typed model/mesh material selection should validate");
        assert_eq!(
            contribution.model_mesh_material_selections[0].surface,
            surface
        );
        assert_eq!(
            contribution.model_mesh_material_selections[0]
                .surface
                .source
                .asset_id,
            42
        );
        assert_eq!(
            contribution.model_mesh_material_selections[0]
                .surface
                .source
                .source_id,
            84
        );
        assert_eq!(
            contribution.model_mesh_material_selections[0].material_table_index,
            1
        );
        assert_eq!(
            contribution.model_mesh_material_selections[0].requested_material_slot_id,
            2
        );
    }

    #[test]
    fn model_mesh_surface_material_selection_rejects_transient_renderer_identity() {
        let source =
            PreparedModelMeshMaterialSourceIdentity::new(42, 84).expect("source should form");
        let region_error =
            PreparedModelMeshMaterialRegionIdentity::new(source, "renderable_index:7").expect_err(
                "raw renderer identity must not become prepared model/mesh region truth",
            );

        assert!(region_error.to_string().contains("transient"));
    }

    #[test]
    fn model_mesh_surface_material_selection_rejects_duplicate_regions() {
        let source = PreparedModelMeshMaterialSourceIdentity::new(42, 84)
            .expect("source-backed model/mesh identity should be valid");
        let surface =
            PreparedModelMeshMaterialRegionIdentity::new(source, "source_material_slot:0")
                .expect("source-backed model/mesh identity should be valid");
        let contribution = PreparedMaterialFeatureContribution {
            instances: Vec::new(),
            binding_table: PreparedMaterialBindingTable::default(),
            scene_bundle: None,
            model_mesh_material_selections: vec![
                PreparedModelMeshMaterialSelection::new(surface.clone(), 2, 2, 1, false)
                    .expect("first selection should validate"),
                PreparedModelMeshMaterialSelection::new(surface, 3, 3, 2, false)
                    .expect("second selection should validate"),
            ],
        };

        let error = contribution
            .validate_portable_limits()
            .expect_err("same source-backed region cannot be selected twice");

        assert!(
            error
                .to_string()
                .contains("duplicate model/mesh material region")
        );
    }
}
