use engine::plugins::world::chunks::dirty::{ChunkDirtyReason, WorldDirtyChunkMapResource};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::plugins::world::ids::{ChunkCoord3, ChunkId, PlanetId};
use engine::plugins::world::plugin::{WorldAuthorityState, WorldPlugin};
use engine::plugins::world::sdf::storage::WorldSdfChunkStoreResource;
use engine::prelude::App;

#[test]
fn dirty_chunk_without_runtime_record_is_bootstrapped_and_built() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let chunk_id = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 2, y: -1, z: 4 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<WorldDirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, ChunkDirtyReason::Geometry);
    }

    let app = app
        .run_for_ticks(1)
        .expect("world plugin systems should run for one fixed tick");

    let chunk_runtime = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime map should be available");
    assert!(
        chunk_runtime.by_chunk_id.contains_key(&chunk_id),
        "dirty chunk should create a runtime record before build dispatch"
    );

    let sdf_store = app
        .world()
        .resource::<WorldSdfChunkStoreResource>()
        .expect("sdf store should be available");
    assert!(
        sdf_store.chunks.contains_key(&chunk_id),
        "dirty chunk should produce an integrated chunk payload in the same tick"
    );

    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("world authority state should be available");
    assert!(
        authority.world_revision.0 > 0,
        "world revision should advance when build outputs integrate"
    );
}
