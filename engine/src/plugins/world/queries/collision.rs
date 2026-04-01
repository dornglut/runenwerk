use super::super::chunks::partition::WorldPartitionConfig;
use super::super::ids::{ChunkId, PlanetId};
use super::super::sdf::storage::WorldSdfChunkStoreResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct WorldCollisionSample {
    pub chunk_id: ChunkId,
    pub distance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct WorldCollisionHit {
    pub chunk_id: ChunkId,
    pub hit_position: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct SphereSweepQuery {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Resource)]
pub struct WorldCollisionQueryServiceResource;

impl WorldCollisionQueryServiceResource {
    pub fn chunk_has_authoritative_payload(
        &self,
        partition: &WorldPartitionConfig,
        store: &WorldSdfChunkStoreResource,
        planet_id: PlanetId,
        world_position: [f32; 3],
    ) -> bool {
        let chunk_id = partition.chunk_id_from_position(planet_id, world_position);
        store.chunks.contains_key(&chunk_id)
    }

    pub fn sample_signed_distance(
        &self,
        partition: &WorldPartitionConfig,
        store: &WorldSdfChunkStoreResource,
        planet_id: PlanetId,
        world_position: [f32; 3],
    ) -> Option<WorldCollisionSample> {
        let chunk_id = partition.chunk_id_from_position(planet_id, world_position);
        let payload = store.chunks.get(&chunk_id)?;
        let summary_distance = if payload.page_table.is_empty() {
            1.0
        } else {
            -1.0
        };
        Some(WorldCollisionSample {
            chunk_id,
            distance: summary_distance,
        })
    }

    pub fn sweep_sphere(
        &self,
        partition: &WorldPartitionConfig,
        store: &WorldSdfChunkStoreResource,
        planet_id: PlanetId,
        query: SphereSweepQuery,
    ) -> Option<WorldCollisionHit> {
        let end_chunk = partition.chunk_id_from_position(planet_id, query.end);
        if !store.chunks.contains_key(&end_chunk) {
            return None;
        }
        let payload = store.chunks.get(&end_chunk)?;
        let has_geometry_pages = payload
            .page_table
            .values()
            .any(|page| !page.bricks.is_empty());
        if !has_geometry_pages {
            return None;
        }
        Some(WorldCollisionHit {
            chunk_id: end_chunk,
            hit_position: query.end,
            normal: [0.0, 1.0, 0.0],
            distance: query.radius.max(0.0),
        })
    }
}
