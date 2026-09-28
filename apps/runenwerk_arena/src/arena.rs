use engine::plugins::world::adapters::PartitionConfigResource;
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfProductCatalogResource,
    WorldSdfRuntimePayloadPackage, enqueue_ratified_world_sdf_payload_package,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::prelude::{App, Plugin, ResMut, Startup};
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_sdf::{
    FieldProductConsumerClass, FieldProductDescriptor, FieldProductId, FieldProductKind,
    FieldProductLineage, FieldProductScope, RegionSdfSummary, SdfBrickMetadata, SdfBrickRecord,
    SdfBrickSamples, SdfChunkPayload, SdfPageCoord3, SdfPageRecord, WorldSdfPayloadRef,
};

pub const ARENA_WORLD_ID: WorldId = WorldId::new(0);
pub const ARENA_CHUNK_EDGE_METERS: f64 = 4.0;
pub const ARENA_PLAYER_SPAWN: [f32; 3] = [1.0, 0.5, 1.0];
pub const ARENA_FIELD_PRODUCT_ID: FieldProductId = FieldProductId(1);

const ARENA_PAGE_COORD: SdfPageCoord3 = SdfPageCoord3 { x: 0, y: 0, z: 0 };
const ARENA_CELL_COUNT_PER_AXIS: u8 = 8;

pub struct ArenaWorldPlugin;

impl Plugin for ArenaWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, install_arena_world);
    }
}

fn install_arena_world(
    mut partition: ResMut<PartitionConfigResource>,
    mut completed: ResMut<WorldCompletedBuildQueueResource>,
    mut chunks: ResMut<WorldChunkRuntimeMapResource>,
    mut products: ResMut<WorldRuntimeSdfProductCatalogResource>,
) {
    *partition = PartitionConfigResource(arena_partition_config());

    let payload = build_arena_chunk_payload();
    let descriptor = arena_field_product_descriptor(&payload);
    let enqueued = enqueue_ratified_world_sdf_payload_package(
        &mut completed,
        &mut chunks,
        &mut products,
        WorldSdfRuntimePayloadPackage::new(
            descriptor,
            vec![payload],
            RegionSdfSummary {
                min_distance: -1,
                max_distance: 1,
                occupied_chunk_count: 1,
                surface_chunk_count: 1,
            },
        ),
    )
    .expect("maintained arena field product should satisfy runtime SDF intake");
    debug_assert_eq!(enqueued, 1);
}

pub fn arena_partition_config() -> GridPartitionConfig {
    GridPartitionConfig::try_new(ARENA_CHUNK_EDGE_METERS, [8, 8, 8])
        .expect("maintained arena partition configuration is valid")
}

pub fn arena_chunk_id() -> ChunkId {
    ChunkId::new(ARENA_WORLD_ID, ChunkCoord3::default())
}

pub fn arena_field_product_descriptor(payload: &SdfChunkPayload) -> FieldProductDescriptor {
    let mut descriptor = FieldProductDescriptor::new(
        ARENA_FIELD_PRODUCT_ID,
        FieldProductKind::WorldSdfChunkPages,
        FieldProductScope::from_chunks([payload.chunk_id]),
        FieldProductLineage::new(1, "runenwerk_arena.runtime_sdf"),
    );
    descriptor.consumer_class = FieldProductConsumerClass::RuntimeRead;
    descriptor
        .payload_refs
        .push(WorldSdfPayloadRef::from(payload));
    descriptor
}

pub fn build_arena_chunk_payload() -> SdfChunkPayload {
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };

    for brick_z in 0..4_u8 {
        for brick_y in 0..4_u8 {
            for brick_x in 0..4_u8 {
                let occupancy_mask = arena_brick_occupancy_mask([brick_x, brick_y, brick_z]);
                let (min_distance, max_distance) = match occupancy_mask {
                    0 => (1, 1),
                    u8::MAX => (-1, -1),
                    _ => (-1, 1),
                };
                page.bricks.insert(
                    [brick_x, brick_y, brick_z],
                    SdfBrickRecord {
                        metadata: SdfBrickMetadata {
                            min_distance,
                            max_distance,
                            occupancy_mask,
                            material_channel_mask: u16::from(occupancy_mask != 0),
                            surface_band_present: occupancy_mask != 0 && occupancy_mask != u8::MAX,
                            ..SdfBrickMetadata::default()
                        },
                        samples: SdfBrickSamples::default(),
                    },
                );
            }
        }
    }

    let mut page_table = std::collections::BTreeMap::new();
    page_table.insert(ARENA_PAGE_COORD, page);

    SdfChunkPayload {
        chunk_id: arena_chunk_id(),
        chunk_revision: Default::default(),
        chunk_generation: Default::default(),
        page_table,
        hierarchy_revision: 0,
        checksum: 1,
    }
}

fn arena_brick_occupancy_mask(brick: [u8; 3]) -> u8 {
    let mut mask = 0_u8;
    for octant_z in 0..2_u8 {
        for octant_y in 0..2_u8 {
            for octant_x in 0..2_u8 {
                let cell = [
                    brick[0] * 2 + octant_x,
                    brick[1] * 2 + octant_y,
                    brick[2] * 2 + octant_z,
                ];
                let solid = cell[1] == 0
                    || cell[0] == 0
                    || cell[0] == ARENA_CELL_COUNT_PER_AXIS - 1
                    || cell[2] == 0
                    || cell[2] == ARENA_CELL_COUNT_PER_AXIS - 1;
                if solid {
                    let octant_index = octant_x | (octant_y << 1) | (octant_z << 2);
                    mask |= 1 << octant_index;
                }
            }
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_payload_populates_every_brick_in_its_page() {
        let payload = build_arena_chunk_payload();
        let page = payload
            .page_table
            .get(&ARENA_PAGE_COORD)
            .expect("arena page should exist");
        assert_eq!(page.bricks.len(), 4 * 4 * 4);
    }

    #[test]
    fn arena_occupancy_encodes_floor_and_boundary_walls() {
        assert_ne!(arena_brick_occupancy_mask([0, 2, 2]), 0);
        assert_ne!(arena_brick_occupancy_mask([3, 2, 2]), 0);
        assert_ne!(arena_brick_occupancy_mask([2, 2, 0]), 0);
        assert_ne!(arena_brick_occupancy_mask([2, 2, 3]), 0);
        assert_ne!(arena_brick_occupancy_mask([2, 0, 2]), 0);
        assert_eq!(arena_brick_occupancy_mask([2, 2, 2]), 0);
    }
}
