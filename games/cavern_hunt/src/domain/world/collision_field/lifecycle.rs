use super::{CavernCollisionField, CavernGeometryGraph, GeometryBounds3};

impl CavernCollisionField {
    pub fn from_graph(graph: &CavernGeometryGraph) -> Self {
        Self {
            world_bounds: graph.bounds,
            revision_seen: graph.revision,
            ..Default::default()
        }
    }

    pub fn sync_revision(&mut self, graph: &CavernGeometryGraph) {
        self.world_bounds = graph.bounds;
        self.revision_seen = graph.revision;
    }

    pub fn invalidate_bounds(&mut self, bounds: GeometryBounds3) {
        let chunk_bounds = self.chunk_bounds_for_aabb(bounds.expanded(1.0));
        for key in chunk_bounds {
            self.dirty_chunks.keys.insert(key);
            if let Some(chunk) = self.chunks.get_mut(&key) {
                chunk.dirty = true;
            }
        }
    }

    pub fn mark_dirty_from_graph(&mut self, graph: &CavernGeometryGraph) {
        self.sync_revision(graph);
        for key in self.chunks.keys().copied().collect::<Vec<_>>() {
            self.dirty_chunks.keys.insert(key);
            if let Some(chunk) = self.chunks.get_mut(&key) {
                chunk.dirty = true;
            }
        }
    }

    pub fn active_chunk_count(&self) -> usize {
        self.chunks.len()
    }
}
