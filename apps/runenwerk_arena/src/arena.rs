use engine::plugins::world::adapters::PartitionConfigResource;
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfMetricCapabilityCatalogResource,
    WorldRuntimeSdfProductCatalogResource, WorldSdfRuntimePayloadPackage,
    enqueue_ratified_world_sdf_payload_package,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::prelude::{App, Plugin, ResMut, Resource, Startup};
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_sdf::{
    FieldProductConsumerClass, FieldProductDescriptor, FieldProductId, FieldProductKind,
    FieldProductLineage, FieldProductScope, RegionSdfSummary, SDF_METRIC_BRICK_EDGE_SAMPLES,
    SDF_METRIC_BRICK_SAMPLE_COUNT, SDF_PAGE_EDGE_BRICKS, SdfBrickMetadata, SdfBrickRecord,
    SdfBrickSamples, SdfChunkPayload, SdfPageCoord3, SdfPageRecord, WorldSdfMetricEncoding,
    WorldSdfMetricPayloadRef, WorldSdfPayloadRef,
};

pub const ARENA_WORLD_ID: WorldId = WorldId::new(0);
pub const ARENA_CHUNK_EDGE_METERS: f64 = 4.0;
pub const ARENA_PLAYER_SPAWN: [f32; 3] = [1.0, 0.5, 1.0];
pub const ARENA_FIELD_PRODUCT_ID: FieldProductId = FieldProductId(1);
pub const ARENA_METRIC_DISTANCE_UNITS_PER_METER: u32 = 1024;
pub const ARENA_METRIC_MAX_ABSOLUTE_ERROR_UNITS: u32 = 444;

/// Static level policy. The trigger does not alter WorldSDF collision truth.
#[derive(Debug, Copy, Clone, PartialEq, Resource)]
pub struct ArenaHazardConfig {
    pub center: [f32; 3],
    pub radius: f32,
    pub damage: u8,
    pub cooldown_ticks: u64,
}

impl Default for ArenaHazardConfig {
    fn default() -> Self {
        Self {
            center: [2.5, 0.5, 2.5],
            radius: 0.45,
            damage: 1,
            cooldown_ticks: 30,
        }
    }
}

const ARENA_PAGE_COORD: SdfPageCoord3 = SdfPageCoord3 { x: 0, y: 0, z: 0 };
const ARENA_CELL_COUNT_PER_AXIS: u8 = 8;
const ARENA_INTERIOR_MIN_METERS: f64 = 0.5;
const ARENA_INTERIOR_MAX_XZ_METERS: f64 = 3.5;

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
    mut metric_capabilities: ResMut<WorldRuntimeSdfMetricCapabilityCatalogResource>,
) {
    *partition = PartitionConfigResource(arena_partition_config());

    let payload = build_arena_chunk_payload();
    let descriptor = arena_field_product_descriptor(&payload);
    let metric_payload_ref = arena_metric_payload_ref(&payload);
    let region_summary = arena_region_summary(&payload);
    let enqueued = enqueue_ratified_world_sdf_payload_package(
        &mut completed,
        &mut chunks,
        &mut products,
        &mut metric_capabilities,
        WorldSdfRuntimePayloadPackage::new(descriptor, vec![payload], region_summary)
            .with_metric_capability(metric_payload_ref),
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

pub fn arena_metric_encoding() -> WorldSdfMetricEncoding {
    WorldSdfMetricEncoding::try_new(
        ARENA_METRIC_DISTANCE_UNITS_PER_METER,
        ARENA_METRIC_MAX_ABSOLUTE_ERROR_UNITS,
    )
    .expect("maintained arena metric encoding is valid")
}

pub fn arena_metric_payload_ref(payload: &SdfChunkPayload) -> WorldSdfMetricPayloadRef {
    WorldSdfMetricPayloadRef::try_new(WorldSdfPayloadRef::from(payload), arena_metric_encoding())
        .expect("maintained arena metric capability is valid")
}

pub fn arena_metric_error_components_meters() -> [f64; 3] {
    let brick_edge_meters = ARENA_CHUNK_EDGE_METERS / SDF_PAGE_EDGE_BRICKS as f64;
    let subcell_edge_meters = brick_edge_meters / (SDF_METRIC_BRICK_EDGE_SAMPLES - 1) as f64;
    [
        0.0,
        (3.0_f64).sqrt() * subcell_edge_meters / 2.0,
        0.5 / f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER),
    ]
}

pub fn arena_signed_distance_meters(position: [f64; 3]) -> f64 {
    let x_violation = interval_violation(
        position[0],
        ARENA_INTERIOR_MIN_METERS,
        ARENA_INTERIOR_MAX_XZ_METERS,
    );
    let y_violation = (ARENA_INTERIOR_MIN_METERS - position[1]).max(0.0);
    let z_violation = interval_violation(
        position[2],
        ARENA_INTERIOR_MIN_METERS,
        ARENA_INTERIOR_MAX_XZ_METERS,
    );
    let outside_distance =
        (x_violation * x_violation + y_violation * y_violation + z_violation * z_violation).sqrt();

    if outside_distance > 0.0 {
        return -outside_distance;
    }

    (position[0] - ARENA_INTERIOR_MIN_METERS)
        .min(ARENA_INTERIOR_MAX_XZ_METERS - position[0])
        .min(position[1] - ARENA_INTERIOR_MIN_METERS)
        .min(position[2] - ARENA_INTERIOR_MIN_METERS)
        .min(ARENA_INTERIOR_MAX_XZ_METERS - position[2])
}

pub fn build_arena_chunk_payload() -> SdfChunkPayload {
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };

    for brick_z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for brick_y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for brick_x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                let brick_coord = [brick_x, brick_y, brick_z];
                let distances = arena_brick_metric_samples(brick_coord);
                let min_distance = *distances
                    .iter()
                    .min()
                    .expect("metric arena brick has canonical samples");
                let max_distance = *distances
                    .iter()
                    .max()
                    .expect("metric arena brick has canonical samples");
                let occupancy_mask = arena_brick_occupancy_mask(brick_coord);
                page.bricks.insert(
                    brick_coord,
                    SdfBrickRecord {
                        metadata: SdfBrickMetadata {
                            min_distance,
                            max_distance,
                            occupancy_mask,
                            material_channel_mask: u16::from(occupancy_mask != 0),
                            surface_band_present: min_distance <= 0 && max_distance >= 0,
                            ..SdfBrickMetadata::default()
                        },
                        samples: SdfBrickSamples { distances },
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

fn arena_region_summary(payload: &SdfChunkPayload) -> RegionSdfSummary {
    let mut min_distance = i16::MAX;
    let mut max_distance = i16::MIN;
    let mut occupied = false;
    let mut surface = false;

    for brick in payload
        .page_table
        .values()
        .flat_map(|page| page.bricks.values())
    {
        min_distance = min_distance.min(brick.metadata.min_distance);
        max_distance = max_distance.max(brick.metadata.max_distance);
        occupied |= brick.metadata.occupancy_mask != 0;
        surface |= brick.metadata.surface_band_present;
    }

    RegionSdfSummary {
        min_distance,
        max_distance,
        occupied_chunk_count: u32::from(occupied),
        surface_chunk_count: u32::from(surface),
    }
}

fn arena_brick_metric_samples(brick: [u8; 3]) -> Vec<i16> {
    let mut distances = Vec::with_capacity(SDF_METRIC_BRICK_SAMPLE_COUNT);
    for sample_z in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
        for sample_y in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
            for sample_x in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                let position = arena_metric_sample_position(brick, [sample_x, sample_y, sample_z]);
                distances.push(encode_arena_distance(arena_signed_distance_meters(
                    position,
                )));
            }
        }
    }
    distances
}

fn arena_metric_sample_position(brick: [u8; 3], sample: [usize; 3]) -> [f64; 3] {
    let brick_edge_meters = ARENA_CHUNK_EDGE_METERS / SDF_PAGE_EDGE_BRICKS as f64;
    let sample_step_meters = brick_edge_meters / (SDF_METRIC_BRICK_EDGE_SAMPLES - 1) as f64;
    [
        f64::from(brick[0]) * brick_edge_meters + sample[0] as f64 * sample_step_meters,
        f64::from(brick[1]) * brick_edge_meters + sample[1] as f64 * sample_step_meters,
        f64::from(brick[2]) * brick_edge_meters + sample[2] as f64 * sample_step_meters,
    ]
}

fn encode_arena_distance(distance_meters: f64) -> i16 {
    let encoded = (distance_meters * f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER)).round();
    assert!(
        encoded >= f64::from(i16::MIN) && encoded <= f64::from(i16::MAX),
        "maintained arena metric sample must fit i16"
    );
    encoded as i16
}

fn arena_brick_occupancy_mask(brick: [u8; 3]) -> u8 {
    let mut mask = 0_u8;
    for octant_z in 0..2_u8 {
        for octant_y in 0..2_u8 {
            for octant_x in 0..2_u8 {
                let position = arena_occupancy_cell_center(brick, [octant_x, octant_y, octant_z]);
                if arena_signed_distance_meters(position) < 0.0 {
                    let octant_index = octant_x | (octant_y << 1) | (octant_z << 2);
                    mask |= 1 << octant_index;
                }
            }
        }
    }
    mask
}

fn arena_occupancy_cell_center(brick: [u8; 3], octant: [u8; 3]) -> [f64; 3] {
    let cell_edge_meters = ARENA_CHUNK_EDGE_METERS / f64::from(ARENA_CELL_COUNT_PER_AXIS);
    [
        f64::from(brick[0] * 2 + octant[0]) * cell_edge_meters + cell_edge_meters * 0.5,
        f64::from(brick[1] * 2 + octant[1]) * cell_edge_meters + cell_edge_meters * 0.5,
        f64::from(brick[2] * 2 + octant[2]) * cell_edge_meters + cell_edge_meters * 0.5,
    ]
}

fn interval_violation(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value < minimum {
        minimum - value
    } else if value > maximum {
        value - maximum
    } else {
        0.0
    }
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

    #[test]
    fn arena_signed_distance_is_euclidean_at_wall_and_floor_junctions() {
        let wall_wall = arena_signed_distance_meters([0.25, 1.0, 0.25]);
        let wall_wall_floor = arena_signed_distance_meters([0.25, 0.25, 0.25]);
        assert!((wall_wall + (2.0_f64).sqrt() * 0.25).abs() <= 1.0e-12);
        assert!((wall_wall_floor + (3.0_f64).sqrt() * 0.25).abs() <= 1.0e-12);
    }
}
