use engine::SimulationTick;
use engine::net::prelude::{
    Ack, ClientMessage, ConnectionId, NetPlugin, NetRole, ServerSessionState, SessionPhase,
    SnapshotCursor,
};
use engine::plugins::net::{NetworkServerInbox, update_connection_closed};
use engine::plugins::world::chunks::dirty::WorldDirtyChunkMapResource;
use engine::plugins::world::chunks::partition::WorldPartitionConfig;
use engine::plugins::world::edits::ingress::{WorldEditIngressMeta, submit_world_operation};
use engine::plugins::world::edits::invalidation::invalidate_dirty_chunks_from_op_log;
use engine::plugins::world::edits::log::WorldOperationLog;
use engine::plugins::world::edits::operation::{
    WorldBrushShape, WorldOperation, WorldOperationRecord, quantize_aabb, quantize_position,
};
use engine::plugins::world::edits::replay::{WorldReplayWindow, operations_for_replay_window};
use engine::plugins::world::ids::{
    ChunkCoord3, ChunkId, ChunkSyncCursor, PlanetId, WorldOpId, WorldRevision,
};
use engine::plugins::world::plugin::{WorldAuthorityState, WorldPlugin};
use engine::plugins::world::streaming::interest::WorldStreamingInterestResource;
use engine::plugins::world::streaming::replication::{
    RegionInvalidationDelta, WorldReplicationStateResource,
};
use engine::prelude::App;
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use serde::{Deserialize, Serialize};
use std::io;

fn build_test_log() -> WorldOperationLog {
    let mut log = WorldOperationLog::default();
    let operations = [
        WorldOperation::CsgSubtract {
            brush: WorldBrushShape::Sphere {
                center_q: quantize_position([2.0, 0.0, -1.0], 1024),
                radius_q: 1536,
            },
        },
        WorldOperation::Smooth {
            bounds_q: quantize_aabb([-6.0, -2.0, -6.0], [6.0, 2.0, 6.0], 1024),
            kernel_radius_q: 512,
            strength_q: 192,
        },
        WorldOperation::MaterialFieldEdit {
            bounds_q: quantize_aabb([-4.0, -1.0, -4.0], [4.0, 1.0, 4.0], 1024),
            channel_mask: 0b0011,
            payload: vec![1, 2, 3, 4],
        },
    ];

    for operation in operations {
        let bounds_q = match &operation {
            WorldOperation::CsgSubtract { .. } => {
                quantize_aabb([-3.0, -3.0, -3.0], [3.0, 3.0, 3.0], 1024)
            }
            WorldOperation::Smooth { bounds_q, .. } => *bounds_q,
            WorldOperation::MaterialFieldEdit { bounds_q, .. } => *bounds_q,
            _ => quantize_aabb([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0], 1024),
        };
        let _ = log.append(WorldOperationRecord {
            op_id: WorldOpId(0),
            base_world_revision: WorldRevision(1),
            planet_id: PlanetId(0),
            operation,
            affected_bounds_q: bounds_q,
            deterministic_seed: 1337,
            server_tick: SimulationTick(12),
            author_connection_id: Some(7),
        });
    }
    log
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ReplicationProbeSnapshot {
    world_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct ReplicationProbeDelta {
    changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct ReplicationProbeInput;

struct ReplicationProbeDriver;

impl ReplicationDriver for ReplicationProbeDriver {
    type Snapshot = ReplicationProbeSnapshot;
    type Delta = ReplicationProbeDelta;
    type Input = ReplicationProbeInput;
    type Error = io::Error;

    fn capture_snapshot(world: &ecs::World) -> Result<Option<Self::Snapshot>, Self::Error> {
        let world_revision = world
            .resource::<WorldAuthorityState>()
            .map(|state| state.world_revision.0)
            .unwrap_or(0);
        Ok(Some(ReplicationProbeSnapshot { world_revision }))
    }

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta {
        ReplicationProbeDelta {
            changed: previous != current,
        }
    }

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot {
        if delta.changed {
            Self::Snapshot {
                world_revision: base.world_revision.saturating_add(1),
            }
        } else {
            base.clone()
        }
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}

impl SnapshotApplyDriver for ReplicationProbeDriver {
    fn apply_snapshot(
        _world: &mut ecs::World,
        _tick: SimulationTick,
        _snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn apply_delta(
        _world: &mut ecs::World,
        _tick: SimulationTick,
        _delta: Self::Delta,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl InputDriver for ReplicationProbeDriver {
    fn receive_remote_input(
        _world: &mut ecs::World,
        _connection_id: ConnectionId,
        _tick: SimulationTick,
        _input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn take_local_input(_world: &mut ecs::World) -> Result<Vec<Self::Input>, Self::Error> {
        Ok(Vec::new())
    }

    fn apply_input(_world: &mut ecs::World, _input: &[Self::Input]) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn op_log_replay_and_invalidation_are_deterministic() {
    let log_a = build_test_log();
    let log_b = build_test_log();

    let replay_window = WorldReplayWindow {
        applied_op_exclusive: WorldOpId(0),
        target_op_inclusive: WorldOpId(3),
    };

    let replay_a = operations_for_replay_window(&log_a, replay_window);
    let replay_b = operations_for_replay_window(&log_b, replay_window);
    assert_eq!(replay_a, replay_b, "replay output must be deterministic");

    let partition = WorldPartitionConfig::default();
    let mut dirty_a = WorldDirtyChunkMapResource::default();
    let mut dirty_b = WorldDirtyChunkMapResource::default();
    invalidate_dirty_chunks_from_op_log(&mut dirty_a, &partition, &log_a, 1024);
    invalidate_dirty_chunks_from_op_log(&mut dirty_b, &partition, &log_b, 1024);
    assert_eq!(
        dirty_a.by_chunk, dirty_b.by_chunk,
        "dirty invalidation set must be deterministic for identical op logs"
    );
}

#[test]
fn world_replication_state_is_built_from_world_runtime() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let fixed_point_scale = app
        .world()
        .resource::<WorldPartitionConfig>()
        .expect("world partition config should exist")
        .quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        WorldOperation::Stamp {
            stamp_id: "tests.world.replication-runtime".to_string(),
            anchor_q: quantize_position([1.0, 1.0, 1.0], fixed_point_scale),
            payload: vec![9, 8, 7, 6],
        },
        quantize_aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: PlanetId(0),
            deterministic_seed: 99,
            server_tick: SimulationTick(5),
            author_connection_id: Some(7),
        },
    );
    assert!(op_id.is_some(), "world ingress should append operation");

    let app = app
        .run_for_ticks(1)
        .expect("world runtime should process one fixed tick");

    let replication = app
        .world()
        .resource::<WorldReplicationStateResource>()
        .expect("world replication state should exist");
    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("world authority state should exist");

    assert_eq!(
        replication.world_revision, authority.world_revision,
        "replication state revision should track authoritative world revision"
    );
    assert_eq!(
        replication.next_op_id.0, 2,
        "replication next op id should advance with ingress operation log"
    );
    assert_eq!(
        replication.pending_op_windows.len(),
        1,
        "replication state should publish op-window deltas from world runtime"
    );
    assert_eq!(
        replication.pending_op_windows[0].operations.len(),
        1,
        "replication op-window should include submitted operation"
    );
    assert!(
        !replication.pending_header_deltas.is_empty(),
        "replication state should publish chunk header deltas from runtime chunks"
    );
    assert!(
        !replication.pending_content_deltas.is_empty(),
        "replication state should publish chunk content deltas from authoritative store"
    );
    assert!(
        replication
            .pending_content_deltas
            .values()
            .all(|value| value
                .full_payload
                .as_ref()
                .is_some_and(|payload| !payload.is_empty())),
        "content deltas should carry serialized authoritative chunk payload snapshots"
    );
    let partition = app
        .world()
        .resource::<WorldPartitionConfig>()
        .expect("world partition config should exist");
    let expected_chunk = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    let expected_region = partition.region_id_from_chunk_id(expected_chunk);
    assert!(
        replication
            .pending_region_invalidations
            .iter()
            .any(|record| record.chunk_ids.contains(&expected_chunk)
                && record.region_ids.contains(&expected_region)),
        "replication state should surface region+chunk invalidation records from world-owned journal"
    );
    assert!(
        replication
            .pending_region_invalidations
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence),
        "region invalidation projection should preserve journal sequence ordering"
    );
}

#[test]
fn world_region_invalidation_projection_is_deterministic() {
    fn run_projection() -> Vec<RegionInvalidationDelta> {
        let mut app = App::headless();
        app.add_plugin(WorldPlugin);
        let fixed_point_scale = app
            .world()
            .resource::<WorldPartitionConfig>()
            .expect("world partition config should exist")
            .quantization_scale();
        let operations = [
            (
                quantize_aabb([0.0, 0.0, 0.0], [40.0, 1.0, 1.0], fixed_point_scale),
                31_u64,
            ),
            (
                quantize_aabb([-24.0, -2.0, -24.0], [-1.0, 2.0, -1.0], fixed_point_scale),
                32_u64,
            ),
        ];
        for (bounds_q, seed) in operations {
            let op_id = submit_world_operation(
                app.world_mut(),
                WorldOperation::Stamp {
                    stamp_id: "tests.world.region-journal".to_string(),
                    anchor_q: quantize_position([0.0, 0.0, 0.0], fixed_point_scale),
                    payload: vec![5, 4, 3, 2],
                },
                bounds_q,
                WorldEditIngressMeta {
                    planet_id: PlanetId(0),
                    deterministic_seed: seed,
                    server_tick: SimulationTick(seed),
                    author_connection_id: Some(seed),
                },
            );
            assert!(op_id.is_some(), "ingress op should append to journal");
        }

        let app = app
            .run_for_ticks(1)
            .expect("world tick should publish replication projection");
        app.world()
            .resource::<WorldReplicationStateResource>()
            .expect("replication resource should exist")
            .pending_region_invalidations
            .clone()
    }

    let projection_a = run_projection();
    let projection_b = run_projection();
    assert_eq!(
        projection_a, projection_b,
        "region invalidation projection must be deterministic across equivalent runs"
    );
}

#[test]
fn world_streaming_interest_tracks_connection_cursor_and_cleanup() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    app.add_plugin(NetPlugin::<ReplicationProbeDriver>::new(NetRole::Server));

    let connection_id = ConnectionId(55);
    {
        let mut session = app
            .world_mut()
            .resource_mut::<ServerSessionState>()
            .expect("net server session state should exist");
        session.phase = SessionPhase::Active;
        session.active_connection = Some(connection_id);
        session.active_connections.insert(connection_id);
    }

    let fixed_point_scale = app
        .world()
        .resource::<WorldPartitionConfig>()
        .expect("world partition config should exist")
        .quantization_scale();
    let _ = submit_world_operation(
        app.world_mut(),
        WorldOperation::Stamp {
            stamp_id: "tests.world.streaming-interest".to_string(),
            anchor_q: quantize_position([2.0, 0.0, -2.0], fixed_point_scale),
            payload: vec![4, 3, 2, 1],
        },
        quantize_aabb([-1.0, -1.0, -1.0], [3.0, 1.0, 3.0], fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: PlanetId(0),
            deterministic_seed: 17,
            server_tick: SimulationTick(9),
            author_connection_id: Some(connection_id.0),
        },
    );

    let mut app = app
        .run_for_ticks(1)
        .expect("fixed tick should produce one replication step");

    {
        let interest = app
            .world()
            .resource::<WorldStreamingInterestResource>()
            .expect("world streaming interest should exist");
        let per_connection = interest
            .per_connection
            .get(&connection_id)
            .expect("active connection should have streaming interest");
        assert!(
            !per_connection.relevant_chunks.is_empty(),
            "active connection should track authoritative runtime chunks"
        );
        assert_eq!(
            per_connection.last_sent_cursor,
            ChunkSyncCursor(1),
            "streaming cursor should advance after first replicated snapshot"
        );
        assert_eq!(
            per_connection.last_ack_cursor,
            ChunkSyncCursor(0),
            "ack cursor should remain at zero before client acknowledgment"
        );
        assert!(
            !per_connection.needs_full_resync,
            "full snapshot send should clear full-resync requirement"
        );
        assert!(
            per_connection.prepared_region_sequence > 0,
            "streaming interest should stage region journal sequence coverage before snapshot send"
        );
    }

    {
        let mut inbox = app
            .world_mut()
            .resource_mut::<NetworkServerInbox>()
            .expect("server inbox should exist");
        inbox.push_from(
            Some(connection_id),
            ClientMessage::Ack(Ack {
                cursor: SnapshotCursor(1),
                last_received_tick: SimulationTick(9),
            }),
        );
    }
    let next_tick = app
        .world()
        .resource::<SimulationTick>()
        .expect("simulation tick should exist")
        .0
        .saturating_add(1);
    app = app
        .run_for_ticks(next_tick)
        .expect("ack tick should update world streaming cursor state");

    let second_chunk = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 2, y: 0, z: 0 });
    {
        let interest = app
            .world()
            .resource::<WorldStreamingInterestResource>()
            .expect("world streaming interest should exist");
        let per_connection = interest
            .per_connection
            .get(&connection_id)
            .expect("active connection should have streaming interest");
        assert_eq!(
            per_connection.last_ack_cursor,
            ChunkSyncCursor(1),
            "server ack processing should advance per-connection ack cursor"
        );
    }

    let _ = submit_world_operation(
        app.world_mut(),
        WorldOperation::Stamp {
            stamp_id: "tests.world.streaming-interest.region-delta".to_string(),
            anchor_q: quantize_position([80.0, 0.0, 0.0], fixed_point_scale),
            payload: vec![8, 8, 8, 8],
        },
        quantize_aabb([80.0, 0.0, 0.0], [80.0, 0.0, 0.0], fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: PlanetId(0),
            deterministic_seed: 18,
            server_tick: SimulationTick(10),
            author_connection_id: Some(connection_id.0),
        },
    );
    let next_tick = app
        .world()
        .resource::<SimulationTick>()
        .expect("simulation tick should exist")
        .0
        .saturating_add(1);
    app = app
        .run_for_ticks(next_tick)
        .expect("region delta tick should update per-connection relevant chunks");

    {
        let interest = app
            .world()
            .resource::<WorldStreamingInterestResource>()
            .expect("world streaming interest should exist");
        let per_connection = interest
            .per_connection
            .get(&connection_id)
            .expect("active connection should have streaming interest");
        assert_eq!(
            per_connection.last_ack_cursor,
            ChunkSyncCursor(1),
            "server ack processing should advance per-connection ack cursor"
        );
        assert!(
            per_connection.relevant_chunks.contains(&second_chunk),
            "post-ack region invalidation policy should surface only newly invalidated chunks"
        );
    }

    update_connection_closed::<ReplicationProbeSnapshot>(
        app.world_mut(),
        Some(connection_id),
        None,
    );

    let interest = app
        .world()
        .resource::<WorldStreamingInterestResource>()
        .expect("world streaming interest should exist");
    assert!(
        !interest.per_connection.contains_key(&connection_id),
        "streaming interest must drop entries for closed server connections"
    );
}
