use super::super::ids::{BuildGeneration, ChunkGeneration, ChunkId, ChunkRevision};
use super::dirty::{ChunkDirtyReasonSet, WorldDirtyChunkMapResource};
use crate::runtime::{ResMut, WorldMut};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub enum ChunkLifecycleState {
    Unloaded,
    Loading,
    Ready,
    Dirty,
    Rebuilding,
    Resident,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct WorldChunkRuntimeRecord {
    pub chunk_id: ChunkId,
    pub lifecycle: ChunkLifecycleState,
    pub chunk_revision: ChunkRevision,
    pub chunk_generation: ChunkGeneration,
    pub build_generation: BuildGeneration,
    pub dirty_reasons: ChunkDirtyReasonSet,
    pub pending_build_generation: Option<BuildGeneration>,
    pub gameplay_locked: bool,
}

impl WorldChunkRuntimeRecord {
    pub fn new(chunk_id: ChunkId) -> Self {
        Self {
            chunk_id,
            lifecycle: ChunkLifecycleState::Unloaded,
            chunk_revision: ChunkRevision::default(),
            chunk_generation: ChunkGeneration::default(),
            build_generation: BuildGeneration::default(),
            dirty_reasons: ChunkDirtyReasonSet::default(),
            pending_build_generation: None,
            gameplay_locked: false,
        }
    }
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldChunkRuntimeMapResource {
    pub by_chunk_id: BTreeMap<ChunkId, WorldChunkRuntimeRecord>,
}

impl WorldChunkRuntimeMapResource {
    pub fn ensure_chunk(&mut self, chunk_id: ChunkId) -> &mut WorldChunkRuntimeRecord {
        self.by_chunk_id
            .entry(chunk_id)
            .or_insert_with(|| WorldChunkRuntimeRecord::new(chunk_id))
    }
}

pub fn advance_chunk_lifecycle_system(
    mut chunks: ResMut<WorldChunkRuntimeMapResource>,
    mut dirty: ResMut<WorldDirtyChunkMapResource>,
) {
    let ids = chunks.by_chunk_id.keys().copied().collect::<Vec<_>>();
    for chunk_id in ids {
        let Some(record) = chunks.by_chunk_id.get_mut(&chunk_id) else {
            continue;
        };
        if let Some(reasons) = dirty.take_reasons(&chunk_id) {
            record.dirty_reasons = reasons;
            record.lifecycle = ChunkLifecycleState::Dirty;
        }
        if matches!(record.lifecycle, ChunkLifecycleState::Loading) {
            record.lifecycle = ChunkLifecycleState::Ready;
        }
        if matches!(record.lifecycle, ChunkLifecycleState::Resident)
            && !record.gameplay_locked
            && record.dirty_reasons.is_empty()
        {
            record.lifecycle = ChunkLifecycleState::Ready;
        }
    }
}

pub fn ensure_chunk_runtime_record(world: &mut WorldMut, chunk_id: ChunkId) {
    if let Ok(mut chunks) = world.resource_mut::<WorldChunkRuntimeMapResource>() {
        chunks.ensure_chunk(chunk_id);
    }
}
