use super::super::ids::ChunkId;
use crate::plugins::render::features::world::runtime_cache::WorldRuntimeCacheResource;
use crate::runtime::WorldMut;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldRenderCacheInvalidationQueueResource {
    pub pending_chunks: BTreeSet<ChunkId>,
}

impl WorldRenderCacheInvalidationQueueResource {
    pub fn enqueue(&mut self, chunk_id: ChunkId) {
        self.pending_chunks.insert(chunk_id);
    }

    pub fn enqueue_many<I>(&mut self, chunk_ids: I)
    where
        I: IntoIterator<Item = ChunkId>,
    {
        self.pending_chunks.extend(chunk_ids);
    }
}

pub fn flush_world_render_cache_invalidations_system(mut world: WorldMut) {
    let pending_chunks = match world.resource::<WorldRenderCacheInvalidationQueueResource>() {
        Ok(queue) => {
            if queue.pending_chunks.is_empty() {
                return;
            }
            queue.pending_chunks.iter().copied().collect::<Vec<_>>()
        }
        Err(_) => return,
    };

    let Ok(runtime_cache) = world.resource_mut::<WorldRuntimeCacheResource>() else {
        // Keep pending invalidations queued until the render cache resource is available.
        return;
    };
    for chunk_id in pending_chunks {
        runtime_cache.mark_stale(chunk_id);
    }

    if let Ok(queue) = world.resource_mut::<WorldRenderCacheInvalidationQueueResource>() {
        queue.pending_chunks.clear();
    }
}
