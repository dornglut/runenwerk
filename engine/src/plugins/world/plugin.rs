use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, FixedUpdate, IntoSystemSetKey, RenderPrepare, SystemConfigExt};
use scheduler::SystemSetKey;

use super::build::integration::integrate_completed_build_outputs_system;
use super::build::jobs::dispatch_world_build_jobs_system;
use super::chunks::lifecycle::advance_chunk_lifecycle_system;
use super::chunks::render_cache_bridge::{
    WorldRenderCacheInvalidationQueueResource, flush_world_render_cache_invalidations_system,
};
use super::debug::metrics::WorldDebugMetricsResource;
use super::prepare::contributions::prepare_world_feature_contributions_system;
use super::{
    build::{graph::WorldBuildGraphResource, queue::WorldBuildQueueResource},
    caves::{
        lighting_scope::WorldCaveLightingScopeResource, portals::WorldCavePortalGraphResource,
        sectors::WorldCaveSectorResource,
    },
    chunks::{
        dirty::WorldDirtyChunkMapResource, lifecycle::WorldChunkRuntimeMapResource,
        partition::WorldPartitionConfig,
    },
    edits::log::WorldOperationLog,
    frames::planet_frame::{CameraRelativeFrameResource, PlanetFrameResource},
    queries::{collision::WorldCollisionQueryServiceResource, nav::WorldNavSummaryResource},
    sdf::storage::WorldSdfChunkStoreResource,
    streaming::{
        interest::WorldStreamingInterestResource, replication::WorldReplicationStateResource,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub enum WorldRuntimeMode {
    ServerAuthoritative,
    ClientReplica,
}

impl Default for WorldRuntimeMode {
    fn default() -> Self {
        Self::ClientReplica
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct WorldRuntimeConfig {
    pub mode: WorldRuntimeMode,
    pub chunk_edge_meters: f32,
    pub region_chunk_dims: [u32; 3],
    pub fixed_point_scale: i32,
}

impl Default for WorldRuntimeConfig {
    fn default() -> Self {
        Self {
            mode: WorldRuntimeMode::ClientReplica,
            chunk_edge_meters: 32.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldAuthorityState {
    pub world_revision: super::ids::revisions::WorldRevision,
}

#[derive(Debug, Copy, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldRuntimeState {
    pub integrated_build_outputs: u64,
    pub dropped_stale_build_outputs: u64,
}

pub struct WorldPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum WorldRuntimeSet {
    Lifecycle,
    BuildDispatch,
    BuildIntegrate,
    RenderCacheSync,
}

impl IntoSystemSetKey for WorldRuntimeSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::Lifecycle => SystemSetKey::of::<WorldRuntimeSet>("WorldRuntimeSet::Lifecycle"),
            Self::BuildDispatch => {
                SystemSetKey::of::<WorldRuntimeSet>("WorldRuntimeSet::BuildDispatch")
            }
            Self::BuildIntegrate => {
                SystemSetKey::of::<WorldRuntimeSet>("WorldRuntimeSet::BuildIntegrate")
            }
            Self::RenderCacheSync => {
                SystemSetKey::of::<WorldRuntimeSet>("WorldRuntimeSet::RenderCacheSync")
            }
        }
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldRuntimeConfig>();
        app.init_resource::<WorldRuntimeState>();
        app.init_resource::<WorldAuthorityState>();
        app.init_resource::<WorldPartitionConfig>();
        app.init_resource::<PlanetFrameResource>();
        app.init_resource::<CameraRelativeFrameResource>();
        app.init_resource::<WorldChunkRuntimeMapResource>();
        app.init_resource::<WorldDirtyChunkMapResource>();
        app.init_resource::<WorldRenderCacheInvalidationQueueResource>();
        app.init_resource::<WorldSdfChunkStoreResource>();
        app.init_resource::<WorldOperationLog>();
        app.init_resource::<WorldBuildGraphResource>();
        app.init_resource::<WorldBuildQueueResource>();
        app.init_resource::<super::build::jobs::WorldBuildJobRuntimeResource>();
        app.init_resource::<super::build::integration::WorldCompletedBuildQueueResource>();
        app.init_resource::<WorldCollisionQueryServiceResource>();
        app.init_resource::<WorldNavSummaryResource>();
        app.init_resource::<WorldStreamingInterestResource>();
        app.init_resource::<WorldReplicationStateResource>();
        app.init_resource::<WorldCaveSectorResource>();
        app.init_resource::<WorldCavePortalGraphResource>();
        app.init_resource::<WorldCaveLightingScopeResource>();
        app.init_resource::<WorldDebugMetricsResource>();

        app.add_systems(
            FixedUpdate,
            advance_chunk_lifecycle_system
                .in_set(WorldRuntimeSet::Lifecycle)
                .in_set(CoreSet::Simulation)
                .before(CoreSet::Replication),
        );
        app.add_systems(
            FixedUpdate,
            dispatch_world_build_jobs_system
                .in_set(WorldRuntimeSet::BuildDispatch)
                .in_set(CoreSet::Simulation)
                .after(WorldRuntimeSet::Lifecycle)
                .before(CoreSet::Replication),
        );
        app.add_systems(
            FixedUpdate,
            integrate_completed_build_outputs_system
                .in_set(WorldRuntimeSet::BuildIntegrate)
                .in_set(CoreSet::Simulation)
                .after(WorldRuntimeSet::BuildDispatch)
                .before(CoreSet::Replication),
        );
        app.add_systems(
            FixedUpdate,
            flush_world_render_cache_invalidations_system
                .in_set(WorldRuntimeSet::RenderCacheSync)
                .in_set(CoreSet::Simulation)
                .after(WorldRuntimeSet::BuildIntegrate)
                .before(CoreSet::Replication),
        );
        app.add_systems(RenderPrepare, prepare_world_feature_contributions_system);
    }
}
