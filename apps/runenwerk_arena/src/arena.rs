use engine::plugins::world::adapters::{PartitionConfigResource, SdfChunkStoreResource};
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfMetricCapabilityCatalogResource,
    WorldRuntimeSdfProductCatalogResource, WorldSdfRuntimePayloadPackage,
    enqueue_ratified_world_sdf_payload_package,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::prelude::{App, Plugin, Res, ResMut, Resource, Startup};
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_sdf::{
    ChunkGeneration, ChunkRevision, FieldProductConsumerClass, FieldProductDescriptor,
    FieldProductId, FieldProductKind, FieldProductLineage, FieldProductScope,
    SDF_METRIC_BRICK_EDGE_SAMPLES, SDF_PAGE_EDGE_BRICKS, SdfChunkPayload, WorldSdfMetricEncoding,
    WorldSdfMetricPayloadRef, WorldSdfOpenTopExactFormation, WorldSdfOpenTopRectInteriorSource,
    WorldSdfPayloadRef, form_world_sdf_open_top_exact_source,
};

#[cfg(test)]
use world_sdf::SdfPageCoord3;

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

#[cfg(test)]
const ARENA_PAGE_COORD: SdfPageCoord3 = SdfPageCoord3 { x: 0, y: 0, z: 0 };
/// The game selects the authored value; the same normalized value drives all
/// its existing analytic and sampled producer outputs.
pub fn arena_geometry_source() -> WorldSdfOpenTopRectInteriorSource {
    WorldSdfOpenTopRectInteriorSource::try_new([0.5, 3.5], 0.5, [0.5, 3.5])
        .expect("maintained Arena interior bounds must be valid")
}

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
    sdf_store: Res<SdfChunkStoreResource>,
    mut products: ResMut<WorldRuntimeSdfProductCatalogResource>,
    mut metric_capabilities: ResMut<WorldRuntimeSdfMetricCapabilityCatalogResource>,
) {
    *partition = PartitionConfigResource(arena_partition_config());

    let formation = arena_exact_formation(
        arena_geometry_source(),
        ARENA_FIELD_PRODUCT_ID,
        1,
        ChunkRevision::default(),
        ChunkGeneration::default(),
    );
    let enqueued = enqueue_ratified_world_sdf_payload_package(
        &mut completed,
        &mut chunks,
        &sdf_store,
        &mut products,
        &mut metric_capabilities,
        WorldSdfRuntimePayloadPackage::from_exact_formation(formation),
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
    arena_signed_distance_from_source(arena_geometry_source(), position)
}

/// Producer-local exact analytic oracle for the specified immutable source.
/// Not a World-admitted collision query or a RunenSDF public numerical API.
pub fn arena_signed_distance_from_source(
    source: WorldSdfOpenTopRectInteriorSource,
    position: [f64; 3],
) -> f64 {
    source.signed_distance_meters(position)
}

/// Domain formation binds normalized full-precision producer source and
/// derived metric bytes in one immutable artifact before World admission.
pub fn arena_exact_formation(
    source: WorldSdfOpenTopRectInteriorSource,
    product_id: FieldProductId,
    source_revision: u64,
    chunk_revision: ChunkRevision,
    chunk_generation: ChunkGeneration,
) -> WorldSdfOpenTopExactFormation {
    form_world_sdf_open_top_exact_source(
        source,
        arena_partition_config(),
        arena_chunk_id(),
        chunk_revision,
        chunk_generation,
        1,
        product_id,
        FieldProductLineage::new(source_revision, "runenwerk_arena.runtime_sdf"),
    )
    .expect("Arena exact source must have representable formed metric samples")
}

pub fn build_arena_chunk_payload() -> SdfChunkPayload {
    build_arena_chunk_payload_from_source(arena_geometry_source())
}

/// Maintained Arena payload production uses the same domain-owned formation
/// as its exact-source admission, not an independent source attachment.
pub fn build_arena_chunk_payload_from_source(
    source: WorldSdfOpenTopRectInteriorSource,
) -> SdfChunkPayload {
    arena_exact_formation(
        source,
        ARENA_FIELD_PRODUCT_ID,
        1,
        ChunkRevision::default(),
        ChunkGeneration::default(),
    )
    .into_payload()
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
        let payload = build_arena_chunk_payload();
        let page = payload.page_table.get(&ARENA_PAGE_COORD).unwrap();
        for cell in [[0, 2, 2], [3, 2, 2], [2, 2, 0], [2, 2, 3], [2, 0, 2]] {
            assert_ne!(page.bricks.get(&cell).unwrap().metadata.occupancy_mask, 0);
        }
        assert_eq!(
            page.bricks.get(&[2, 2, 2]).unwrap().metadata.occupancy_mask,
            0
        );
    }

    #[test]
    fn arena_signed_distance_is_euclidean_at_wall_and_floor_junctions() {
        let wall_wall = arena_signed_distance_meters([0.25, 1.0, 0.25]);
        let wall_wall_floor = arena_signed_distance_meters([0.25, 0.25, 0.25]);
        assert!((wall_wall + (2.0_f64).sqrt() * 0.25).abs() <= 1.0e-12);
        assert!((wall_wall_floor + (3.0_f64).sqrt() * 0.25).abs() <= 1.0e-12);
    }
}
