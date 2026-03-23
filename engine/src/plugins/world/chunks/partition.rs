use super::super::ids::{ChunkCoord3, ChunkId, PlanetId, RegionCoord3, RegionId};

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct WorldPartitionConfig {
    pub chunk_edge_meters: f32,
    pub region_chunk_dims: [u32; 3],
}

impl Default for WorldPartitionConfig {
    fn default() -> Self {
        Self {
            chunk_edge_meters: 32.0,
            region_chunk_dims: [8, 8, 8],
        }
    }
}

impl WorldPartitionConfig {
    pub fn chunk_coord_from_planet_local_position(&self, position_meters: [f32; 3]) -> ChunkCoord3 {
        let edge = self.chunk_edge_meters.max(1.0);
        ChunkCoord3 {
            x: (position_meters[0] / edge).floor() as i32,
            y: (position_meters[1] / edge).floor() as i32,
            z: (position_meters[2] / edge).floor() as i32,
        }
    }

    pub fn region_coord_from_chunk_coord(&self, chunk: ChunkCoord3) -> RegionCoord3 {
        let dims = [
            self.region_chunk_dims[0].max(1) as i32,
            self.region_chunk_dims[1].max(1) as i32,
            self.region_chunk_dims[2].max(1) as i32,
        ];
        RegionCoord3 {
            x: div_floor(chunk.x, dims[0]),
            y: div_floor(chunk.y, dims[1]),
            z: div_floor(chunk.z, dims[2]),
        }
    }

    pub fn chunk_id_from_position(
        &self,
        planet_id: PlanetId,
        position_meters: [f32; 3],
    ) -> ChunkId {
        ChunkId::new(
            planet_id,
            self.chunk_coord_from_planet_local_position(position_meters),
        )
    }

    pub fn region_id_from_chunk_id(&self, chunk_id: ChunkId) -> RegionId {
        RegionId::new(
            chunk_id.planet_id,
            self.region_coord_from_chunk_coord(chunk_id.coord),
        )
    }
}

fn div_floor(value: i32, divisor: i32) -> i32 {
    let mut out = value / divisor;
    let remainder = value % divisor;
    if remainder != 0 && ((remainder < 0) != (divisor < 0)) {
        out -= 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_coord_handles_negative_chunks() {
        let partition = WorldPartitionConfig::default();
        let region = partition.region_coord_from_chunk_coord(ChunkCoord3 {
            x: -1,
            y: -8,
            z: -9,
        });
        assert_eq!(region.x, -1);
        assert_eq!(region.y, -1);
        assert_eq!(region.z, -2);
    }
}
