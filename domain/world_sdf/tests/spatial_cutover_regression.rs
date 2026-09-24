use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_ops::{ChunkGeneration, ChunkRevision};
use world_sdf::{
    CollisionQueryService, CollisionReadiness, CollisionSweepOutcome, SDF_PAGE_EDGE_BRICKS,
    SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples, SdfChunkPayload, SdfChunkStore,
    SdfPageCoord3, SdfPageRecord, SphereSweep,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(1.0, [8, 8, 8]).expect("test partition configuration is valid")
}

fn clear_payload(chunk_id: ChunkId) -> SdfChunkPayload {
    SdfChunkPayload {
        chunk_id,
        chunk_revision: ChunkRevision::default(),
        chunk_generation: ChunkGeneration::default(),
        page_table: Default::default(),
        hierarchy_revision: 0,
        checksum: 0,
    }
}

fn uniform_page_payload(chunk_id: ChunkId, occupancy_mask: u8) -> SdfChunkPayload {
    let mut payload = clear_payload(chunk_id);
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };
    for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                page.bricks.insert(
                    [x, y, z],
                    SdfBrickRecord {
                        metadata: SdfBrickMetadata {
                            occupancy_mask,
                            ..SdfBrickMetadata::default()
                        },
                        samples: SdfBrickSamples::default(),
                    },
                );
            }
        }
    }
    payload
        .page_table
        .insert(SdfPageCoord3 { x: 0, y: 0, z: 0 }, page);
    payload
}

#[test]
fn sweep_readiness_is_ready_when_all_required_chunks_are_loaded() {
    let service = CollisionQueryService;
    let partition = partition();
    let mut store = SdfChunkStore::default();
    let world_id = WorldId::new(0);
    for x in 0..=2 {
        let chunk = ChunkId::new(world_id, ChunkCoord3 { x, y: 0, z: 0 });
        store.chunks.insert(chunk, clear_payload(chunk));
    }

    let readiness = service
        .collision_readiness_for_sweep(
            &partition,
            &store,
            world_id,
            SphereSweep {
                start: [0.1, 0.1, 0.1],
                end: [2.1, 0.1, 0.1],
                radius: 0.0,
            },
        )
        .expect("sweep bounds are valid");

    assert_eq!(readiness, CollisionReadiness::Ready);
}

#[test]
fn sweep_can_stay_clear_inside_partial_occupancy_chunk() {
    let service = CollisionQueryService;
    let partition = partition();
    let mut store = SdfChunkStore::default();
    let world_id = WorldId::new(0);
    let chunk_id = ChunkId::new(world_id, ChunkCoord3::default());
    store
        .chunks
        .insert(chunk_id, uniform_page_payload(chunk_id, 0b1000_0000));

    let outcome = service
        .sweep_sphere_authoritative(
            &partition,
            &store,
            world_id,
            SphereSweep {
                start: [0.1, 0.1, 0.1],
                end: [0.9, 0.1, 0.1],
                radius: 0.0,
            },
        )
        .expect("sweep query is spatially valid");

    assert_eq!(outcome, CollisionSweepOutcome::Clear);
}
