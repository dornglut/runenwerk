use super::super::chunks::lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource};
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::ids::{BuildGeneration, ChunkGeneration, ChunkId, ChunkRevision};
use super::super::sdf::storage::{RegionSdfSummary, SdfChunkPayload};
use super::graph::{WorldBuildGraphNode, WorldBuildGraphPhase, WorldBuildGraphResource};
use super::integration::{WorldCompletedBuildOutput, WorldCompletedBuildQueueResource};
use super::queue::{WorldBuildQueueClass, WorldBuildQueueItem, WorldBuildQueueResource};
use crate::runtime::ResMut;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
pub enum WorldBuildStaleness {
    Current,
    Superseded,
    InvalidBase,
}

#[derive(Debug, Clone, ecs::Resource)]
pub struct WorldBuildJob {
    pub chunk_id: ChunkId,
    pub target_chunk_revision: ChunkRevision,
    pub target_build_generation: BuildGeneration,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct WorldBuildJobRuntimeResource {
    pub max_jobs_per_update: usize,
    pub enqueued_jobs: u64,
    pub completed_jobs: u64,
    pub dropped_jobs: u64,
}

impl Default for WorldBuildJobRuntimeResource {
    fn default() -> Self {
        Self {
            max_jobs_per_update: 4,
            enqueued_jobs: 0,
            completed_jobs: 0,
            dropped_jobs: 0,
        }
    }
}

pub fn dispatch_world_build_jobs_system(
    mut chunks: ResMut<WorldChunkRuntimeMapResource>,
    mut queue: ResMut<WorldBuildQueueResource>,
    mut graph: ResMut<WorldBuildGraphResource>,
    mut runtime: ResMut<WorldBuildJobRuntimeResource>,
    mut completed: ResMut<WorldCompletedBuildQueueResource>,
    mut debug: ResMut<WorldDebugMetricsResource>,
) {
    queue_dirty_chunks(&chunks, &mut queue, &mut runtime);
    let (interactive_depth, background_depth) = queue.queue_depths();
    debug.interactive_queue_depth = interactive_depth;
    debug.background_queue_depth = background_depth;

    for _ in 0..runtime.max_jobs_per_update {
        let Some(item) = queue.pop_next() else {
            break;
        };

        let record = chunks.ensure_chunk(item.chunk_id);
        if !matches!(
            record.lifecycle,
            ChunkLifecycleState::Dirty | ChunkLifecycleState::Rebuilding
        ) {
            runtime.dropped_jobs = runtime.dropped_jobs.saturating_add(1);
            continue;
        }

        let target_chunk_revision = ChunkRevision(record.chunk_revision.0.saturating_add(1));
        let target_build_generation = BuildGeneration(record.build_generation.0.saturating_add(1));
        record.pending_build_generation = Some(target_build_generation);
        record.lifecycle = ChunkLifecycleState::Rebuilding;

        push_phase_nodes(
            &mut graph,
            item.chunk_id,
            target_chunk_revision,
            target_build_generation,
        );

        let payload = placeholder_chunk_payload(
            item.chunk_id,
            target_chunk_revision,
            target_build_generation,
        );
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id: item.chunk_id,
            target_chunk_revision,
            target_build_generation,
            staleness: WorldBuildStaleness::Current,
            chunk_payload: payload,
            region_summary: RegionSdfSummary {
                min_distance: -32,
                max_distance: 32,
                occupied_chunk_count: 1,
                surface_chunk_count: 1,
            },
        });
        runtime.completed_jobs = runtime.completed_jobs.saturating_add(1);
    }

    debug.enqueued_build_jobs = runtime.enqueued_jobs;
    debug.completed_build_jobs = runtime.completed_jobs;
}

fn queue_dirty_chunks(
    chunks: &WorldChunkRuntimeMapResource,
    queue: &mut WorldBuildQueueResource,
    runtime: &mut WorldBuildJobRuntimeResource,
) {
    for record in chunks.by_chunk_id.values() {
        if !matches!(record.lifecycle, ChunkLifecycleState::Dirty) {
            continue;
        }
        if record.pending_build_generation.is_some() || queue.contains_chunk(record.chunk_id) {
            continue;
        }
        let queue_class = if record.gameplay_locked {
            WorldBuildQueueClass::Interactive
        } else {
            WorldBuildQueueClass::Background
        };
        let priority_score =
            build_priority_score(record.chunk_id, queue_class, record.gameplay_locked);
        queue.enqueue(WorldBuildQueueItem {
            chunk_id: record.chunk_id,
            queue_class,
            priority_score,
            starvation_age: 0,
        });
        runtime.enqueued_jobs = runtime.enqueued_jobs.saturating_add(1);
    }
}

fn build_priority_score(
    chunk_id: ChunkId,
    class: WorldBuildQueueClass,
    gameplay_locked: bool,
) -> i64 {
    let base = match class {
        WorldBuildQueueClass::Interactive => 1_000_i64,
        WorldBuildQueueClass::Background => 100_i64,
    };
    let lock_bonus = if gameplay_locked { 500 } else { 0 };
    let coord_hash = (chunk_id.coord.x as i64).wrapping_mul(73856093)
        ^ (chunk_id.coord.y as i64).wrapping_mul(19349663)
        ^ (chunk_id.coord.z as i64).wrapping_mul(83492791);
    base.saturating_add(lock_bonus)
        .saturating_add(coord_hash.abs() % 97)
}

fn push_phase_nodes(
    graph: &mut WorldBuildGraphResource,
    chunk_id: ChunkId,
    target_chunk_revision: ChunkRevision,
    target_build_generation: BuildGeneration,
) {
    let phases = [
        WorldBuildGraphPhase::DirtyPlan,
        WorldBuildGraphPhase::OpWindowResolve,
        WorldBuildGraphPhase::SdfFieldBuild,
        WorldBuildGraphPhase::SummaryBuild,
        WorldBuildGraphPhase::DerivedRenderBuild,
        WorldBuildGraphPhase::Publish,
    ];
    for phase in phases {
        graph.nodes.push(WorldBuildGraphNode {
            chunk_id,
            phase,
            target_chunk_revision,
            input_generation_stamp: target_build_generation,
        });
    }
}

fn placeholder_chunk_payload(
    chunk_id: ChunkId,
    chunk_revision: ChunkRevision,
    build_generation: BuildGeneration,
) -> SdfChunkPayload {
    let chunk_generation = ChunkGeneration(build_generation.0);
    let checksum = chunk_id.coord.x as u64
        ^ ((chunk_id.coord.y as u64) << 16)
        ^ ((chunk_id.coord.z as u64) << 32)
        ^ chunk_revision.0
        ^ build_generation.0.rotate_left(7);
    SdfChunkPayload {
        chunk_id,
        chunk_revision,
        chunk_generation,
        page_table: Default::default(),
        hierarchy_revision: build_generation.0,
        checksum,
    }
}
