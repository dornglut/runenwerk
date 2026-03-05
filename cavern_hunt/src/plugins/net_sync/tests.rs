use super::{
    CavernDeltaEventV1, CavernGeometryEditsEventV1, CavernNetSyncMode, CavernNetSyncState,
    CavernPatchEventV2, CavernSnapshotEventV1, ClientReplicationStateV2,
    REPLICATION_DELTA_INTERVAL_TICKS, RUN_EVENT_DELTA, RUN_EVENT_GEOMETRY_EDITS,
    RUN_EVENT_KEYFRAME_V2, RUN_EVENT_PATCH_V2, RUN_EVENT_SNAPSHOT,
    ServerReplicationStateByConnection, apply_player_patch_ops_v2,
    apply_replication_tuning_overrides_from_reader, apply_replication_tuning_preset,
    client_apply_replication_events, client_apply_replication_events_v2,
    compute_load_shed_level_v2, current_net_sync_mode, server_capture_control_input,
    server_emit_replication, server_emit_replication_v2, should_emit_patch_channel,
};
use crate::domain::{
    AdaptiveSmoothingState, CavernAimState, CavernCameraState, CavernControlState,
    CavernMetaProfile, CavernPatchPriorityV2, CavernPlayerOwnershipState, CavernPlayerPatchOpV2,
    CavernPredictionState, CavernRunConfig, CavernRunState, CavernServerControlMap,
    ClientReplicationMap, CorrectionStats, GeometryEdit, GeometryEditKind, GeometryOp,
    GeometryPrimitiveShape3, InterpolationConfig, LocalPlayerRef, LootTableRegistry,
    NetworkEntityId, ReplicationBudgetConfig, ReplicationCadenceConfig, ReplicationLoadShedConfig,
    ReplicationRuntimeMetrics, ServerReplicationMap, SpawnDirector, capture_cavern_run_snapshot,
    restore_cavern_run_snapshot,
};
use crate::plugins::{combat, game, worldgen};
use engine::prelude::{
    FixedTimeConfig, NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus,
    SimulationProfile, SimulationProfileConfig, SimulationTick, World,
};
use engine_net::{
    ClientCommandEnvelope, ClientMessage, ConnectionId, InputFrame, MoveCommand, RunEvent,
    ServerMessage,
};

fn server_world() -> World {
    let mut world = World::new();
    world.insert_resource(CavernRunConfig::default());
    world.insert_resource(CavernRunState::default());
    world.insert_resource(crate::domain::CavernLayout::default());
    world.insert_resource(SpawnDirector::default());
    world.insert_resource(LootTableRegistry::default());
    world.insert_resource(CavernMetaProfile::default());
    world.insert_resource(LocalPlayerRef::default());
    world.insert_resource(CavernCameraState::default());
    world.insert_resource(CavernAimState::default());
    world.insert_resource(CavernControlState::default());
    world.insert_resource(CavernPredictionState::default());
    world.insert_resource(CavernServerControlMap::default());
    world.insert_resource(CavernPlayerOwnershipState::default());
    world.insert_resource(NetworkServerOutbox::default());
    world.insert_resource(NetworkSessionStatus {
        phase: engine_net::SessionPhase::Active,
        connection_id: Some(ConnectionId(7)),
        connected: true,
        ..Default::default()
    });
    world.insert_resource(CavernNetSyncState::default());
    world.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Server,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    world.insert_resource(SimulationTick(1));
    worldgen::initialize_run_world(&mut world, false).unwrap();
    world
}

#[path = "tests/budget_geometry.rs"]
mod budget_geometry;
#[path = "tests/keyframe_flow.rs"]
mod keyframe_flow;
#[path = "tests/legacy_snapshot_client.rs"]
mod legacy_snapshot_client;
#[path = "tests/player_patch.rs"]
mod player_patch;
#[path = "tests/replication_flow.rs"]
mod replication_flow;
#[path = "tests/tuning_input.rs"]
mod tuning_input;
