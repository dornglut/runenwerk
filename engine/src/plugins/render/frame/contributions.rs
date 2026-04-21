use crate::plugins::render::api::ids::RenderFeatureId;
use crate::plugins::render::features::{
    CAVE_INTERIOR_RENDER_FEATURE_ID, DEFORMATION_RENDER_FEATURE_ID, DETAIL_RENDER_FEATURE_ID,
    FeatureContributionStatus, FeatureFallbackPolicy, MATERIAL_RENDER_FEATURE_ID,
    PROCEDURAL_WORLD_RENDER_FEATURE_ID, PreparedUiFrameContribution,
    SCENE_ROUTE_RENDER_FEATURE_ID, UI_RENDER_FEATURE_ID, WIND_FIELDS_RENDER_FEATURE_ID,
    WORLD_DRAW_RENDER_FEATURE_ID,
};
use spatial::ChunkId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct PreparedFrameContributions {
    pub by_feature: BTreeMap<RenderFeatureId, PreparedFeatureContribution>,
}

impl PreparedFrameContributions {
    pub fn feature(&self, id: &RenderFeatureId) -> Option<&PreparedFeatureContribution> {
        self.by_feature.get(id)
    }

    pub fn insert(&mut self, id: RenderFeatureId, contribution: PreparedFeatureContribution) {
        self.by_feature.insert(id, contribution);
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

#[derive(Debug, Clone, Default)]
pub struct PreparedMaterialFeatureContribution {
    pub instances: Vec<PreparedMaterialInstanceInput>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedMaterialInstanceInput {
    pub material_instance_id: String,
    pub specialization_key_fragment: String,
    pub parameter_blob: Vec<u8>,
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