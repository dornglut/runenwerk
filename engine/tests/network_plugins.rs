use engine::net::prelude::*;
use engine::plugins::net::{
    ActiveClientReplicatedStateProduct, AuthorityReplicationPolicy, ClientPredictionPolicy,
    ClientReplicationPolicy, NetPluginConfig, NetworkClientInbox, NetworkClientOutbox,
    NetworkDiagnostics, NetworkOutboundQueue, NetworkServerInbox, NetworkServerOutbox,
    NetworkSessionStatus, OutboundServerMessage, PredictionDiagnostics, ReplicationDiagnostics,
    RunenNetSessionCore, RunenNetSessionProjection, authority_replication_submissions,
    cancel_authority_replication_submission, client_inbox_is_empty,
    client_outbox_len, client_prediction_connection_lost,
    client_prediction_participant_membership_ended, client_prediction_pending_bytes,
    client_prediction_pending_count, client_prediction_session_closed, client_prediction_state,
    client_replication_acknowledgement, client_replication_lineage, client_replication_state,
    enqueue_client_inbox, enqueue_client_outbox, enqueue_server_inbox, enqueue_server_inbox_from,
    enqueue_server_outbox_broadcast, record_authority_replication_delivery_acceptance,
    record_reconnect_attempt,
    require_client_replication_connection_replacement, server_inbox_is_empty, server_outbox_len,
    sync_runennet_session_projection,
};
use engine::plugins::{ScenePlugin, SimulationPlugin, default_plugins};
use engine::prelude::*;
use runen_net::{DeliveryAcceptance, identity::{ConnectionHandle, ParticipantId, SessionId}};
use runen_net::input::{
    AuthorityInputAggregateLimits, AuthorityInputLimits, PredictionInvalidationReason,
    PredictionLimits, PredictionState as RunenNetPredictionState,
};
use runen_net::protocol::{
    CompatibilityOffer, NegotiatedContract, NegotiationManager, NegotiationManagerLimits,
    NegotiationRequirements, OfferLimits, ProtocolContract, ProtocolId, ProtocolRevision,
};
use runen_net::replication::{
    AuthorityAggregateLimits, ClientAggregateLimits, ClientRecoveryReason, ClientReplicationState,
    ReplicationLineageKey, ReplicationRetentionLimits,
};
use runen_net::session::{RecoveryDuration, RetentionPolicy, Session, SessionLimits};
use serde::{Deserialize, Serialize};
use std::io;
use std::num::{NonZeroU64, NonZeroUsize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct MoveCommand {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AbilityCommand {
    slot: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum ClientCommandEnvelope {
    Move(MoveCommand),
    Ability(AbilityCommand),
}

impl Default for ClientCommandEnvelope {
    fn default() -> Self {
        Self::Move(MoveCommand { x: 0.0, y: 0.0 })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Component, runen_ecs::Resource)]
struct PlayerCommandBuffer {
    commands: Vec<ClientCommandEnvelope>,
}

impl PlayerCommandBuffer {
    fn push(&mut self, command: ClientCommandEnvelope) {
        self.commands.push(command);
    }

    fn drain(&mut self) -> Vec<ClientCommandEnvelope> {
        std::mem::take(&mut self.commands)
    }

    fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, runen_ecs::Resource)]
struct AppliedInputLog {
    inputs: Vec<ClientCommandEnvelope>,
    ticks: Vec<engine_sim::SimulationTick>,
}

#[derive(Debug, Clone, Copy, Default, runen_ecs::Resource)]
struct RejectSnapshotRealization(bool);

#[derive(Debug, Clone, Copy, Default, runen_ecs::Resource)]
struct RejectReplayAndNextSnapshot(bool);

#[derive(Debug, Clone, Copy, Default, runen_ecs::Resource)]
struct RejectNextInputApplication(bool);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestSnapshot {
    context: TestSnapshotContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestSnapshotContext {
    world_scene_label: String,
}

impl Default for TestSnapshot {
    fn default() -> Self {
        Self {
            context: TestSnapshotContext {
                world_scene_label: "gameplay_stub".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct TestDelta {
    changed: bool,
}

struct TestReplicationDriver;

impl ReplicationDriver for TestReplicationDriver {
    type Snapshot = TestSnapshot;
    type Delta = TestDelta;
    type Input = ClientCommandEnvelope;
    type Error = io::Error;

    fn capture_snapshot(_world: &World) -> Result<Option<Self::Snapshot>, Self::Error> {
        Ok(Some(TestSnapshot::default()))
    }

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta {
        TestDelta {
            changed: previous != current,
        }
    }

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot {
        if delta.changed {
            Self::Snapshot::default()
        } else {
            base.clone()
        }
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}

impl SnapshotApplyDriver for TestReplicationDriver {
    fn apply_snapshot(
        world: &mut World,
        _tick: engine_sim::SimulationTick,
        _snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error> {
        if world
            .resource::<RejectSnapshotRealization>()
            .map(|reject| reject.0)
            .unwrap_or(false)
        {
            return Err(io::Error::other("test snapshot realization rejected"));
        }
        Ok(true)
    }
}

impl InputDriver for TestReplicationDriver {
    fn receive_remote_input(
        _world: &mut World,
        _connection: ConnectionHandle,
        _tick: engine_sim::SimulationTick,
        _input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn take_local_input(world: &mut World) -> Result<Vec<Self::Input>, Self::Error> {
        Ok(world
            .resource_mut::<PlayerCommandBuffer>()
            .map(|commands| commands.drain())
            .unwrap_or_default())
    }

    fn apply_input(
        world: &mut World,
        tick: engine_sim::SimulationTick,
        input: &[Self::Input],
    ) -> Result<(), Self::Error> {
        let reject_initial = world
            .resource_mut::<RejectNextInputApplication>()
            .map(|reject| {
                let reject_now = reject.0;
                reject.0 = false;
                reject_now
            })
            .unwrap_or(false);
        if reject_initial {
            return Err(io::Error::other(
                "test local prediction application rejected",
            ));
        }

        let reject_replay = world
            .resource_mut::<RejectReplayAndNextSnapshot>()
            .map(|reject| {
                let reject_now = reject.0;
                reject.0 = false;
                reject_now
            })
            .unwrap_or(false);
        if reject_replay {
            if let Ok(reject_snapshot) = world.resource_mut::<RejectSnapshotRealization>() {
                reject_snapshot.0 = true;
            } else {
                world.insert_resource(RejectSnapshotRealization(true));
            }
            return Err(io::Error::other(
                "test replay application rejected and next snapshot restoration armed",
            ));
        }

        if world.resource::<AppliedInputLog>().is_err() {
            world.insert_resource(AppliedInputLog::default());
        }
        let log = world
            .resource_mut::<AppliedInputLog>()
            .expect("applied-input log should exist after initialization");
        log.inputs.extend_from_slice(input);
        log.ticks.push(tick);
        Ok(())
    }
}

fn test_client_replication_policy() -> ClientReplicationPolicy {
    test_client_replication_policy_with_state_limit(64 * 1024)
}

fn test_client_replication_policy_with_state_limit(
    max_state_image_bytes: usize,
) -> ClientReplicationPolicy {
    let max_state_image_bytes =
        NonZeroUsize::new(max_state_image_bytes).expect("test state-image limit must be non-zero");
    let retention = ReplicationRetentionLimits::new(
        max_state_image_bytes,
        NonZeroUsize::new(256).expect("test retained-image limit must be non-zero"),
        NonZeroUsize::new(16 * 1024 * 1024).expect("test retained-byte limit must be non-zero"),
        max_state_image_bytes,
        NonZeroUsize::new(256).expect("test emission-evidence limit must be non-zero"),
    )
    .expect("test client retention limits must be valid");
    let aggregate = ClientAggregateLimits::new(
        NonZeroUsize::new(1).expect("test lineage limit must be non-zero"),
        NonZeroUsize::new(256).expect("test aggregate retained-image limit must be non-zero"),
        NonZeroUsize::new(16 * 1024 * 1024)
            .expect("test aggregate retained-byte limit must be non-zero"),
    );
    ClientReplicationPolicy::new(
        ReplicationLineageKey::new(SessionId::new(1), ParticipantId::new(1)),
        aggregate,
        retention,
    )
}

fn test_authority_replication_policy() -> AuthorityReplicationPolicy {
    let state_image = NonZeroUsize::new(64 * 1024).expect("test state-image limit must be non-zero");
    let retention = ReplicationRetentionLimits::new(
        state_image,
        NonZeroUsize::new(64).expect("test retained-image limit must be non-zero"),
        NonZeroUsize::new(4 * 1024 * 1024).expect("test retained-byte limit must be non-zero"),
        state_image,
        NonZeroUsize::new(64).expect("test emission-evidence limit must be non-zero"),
    )
    .expect("test authority retention limits must be valid");
    let aggregate = AuthorityAggregateLimits::new(
        NonZeroUsize::new(16).expect("test lineage limit must be non-zero"),
        NonZeroUsize::new(4 * 1024 * 1024).expect("test state-byte limit must be non-zero"),
        NonZeroUsize::new(1_024).expect("test retained-image aggregate must be non-zero"),
        NonZeroUsize::new(16 * 1024 * 1024)
            .expect("test retained-byte aggregate must be non-zero"),
        NonZeroUsize::new(1_024).expect("test emission-evidence aggregate must be non-zero"),
    );
    AuthorityReplicationPolicy::new(aggregate, retention)
}

fn test_client_prediction_policy() -> ClientPredictionPolicy {
    ClientPredictionPolicy::new(PredictionLimits::new(
        NonZeroUsize::new(256).expect("test pending-input limit must be non-zero"),
        NonZeroUsize::new(16 * 1024 * 1024).expect("test pending-byte limit must be non-zero"),
        8,
    ))
}

fn test_authority_input_policy() -> AuthorityInputPolicy {
    let participant_limits = AuthorityInputLimits::new(
        NonZeroUsize::new(64 * 1024).expect("test batch limit must be non-zero"),
        NonZeroUsize::new(64).expect("test participant key limit must be non-zero"),
        NonZeroUsize::new(512 * 1024).expect("test participant byte limit must be non-zero"),
        8,
    )
    .expect("test participant input limits must be valid");
    let aggregate_limits = AuthorityInputAggregateLimits::new(
        NonZeroUsize::new(1_024).expect("test aggregate key limit must be non-zero"),
        NonZeroUsize::new(8 * 1024 * 1024).expect("test aggregate byte limit must be non-zero"),
    );
    AuthorityInputPolicy::new(participant_limits, aggregate_limits)
}

struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(SimulationPlugin);
        app.add_plugin(
            NetPlugin::<TestReplicationDriver>::new(NetRole::Client).with_config(
                NetPluginConfig::default()
                    .with_client_replication_policy(test_client_replication_policy())
                    .with_client_prediction_policy(test_client_prediction_policy()),
            ),
        );
    }
}

struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(SimulationPlugin);
        app.add_plugin(NetPlugin::<TestReplicationDriver>::new(NetRole::Server));
    }
}

struct NetworkHostPlugin;

impl Plugin for NetworkHostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(SimulationPlugin);
        app.add_plugin(
            NetPlugin::<TestReplicationDriver>::new(NetRole::Host).with_config(
                NetPluginConfig::default()
                    .with_client_replication_policy(test_client_replication_policy())
                    .with_client_prediction_policy(test_client_prediction_policy()),
            ),
        );
    }
}

fn test_protocol_contract() -> ProtocolContract {
    ProtocolContract::new(ProtocolId::new(1), ProtocolRevision::new(1))
}

fn test_compatibility_offer() -> CompatibilityOffer {
    CompatibilityOffer::new(vec![test_protocol_contract()], vec![], vec![], None)
}

fn test_runennet_session_core_without_authority_input() -> RunenNetSessionCore {
    let negotiation =
        NegotiationManager::new(OfferLimits::default(), NegotiationManagerLimits::default())
            .expect("test negotiation limits must be valid");
    let capacity = NonZeroUsize::new(16).expect("test session capacity must be non-zero");
    let limits = SessionLimits::new(capacity, capacity).expect("test session limits must be valid");
    let session = Session::new(SessionId::new(1), limits);
    RunenNetSessionCore::new(negotiation, session)
}

fn test_runennet_session_core() -> RunenNetSessionCore {
    test_runennet_session_core_without_authority_input()
        .with_authority_input_policy(test_authority_input_policy())
        .with_authority_replication_policy(test_authority_replication_policy())
}

fn establish_runennet_negotiation(core: &mut RunenNetSessionCore, connection: ConnectionHandle) {
    core.negotiation_mut()
        .start(
            connection,
            test_compatibility_offer(),
            test_compatibility_offer(),
        )
        .expect("compatible test negotiation must start");
    core.negotiation_mut()
        .propose(
            connection,
            NegotiatedContract::new(test_protocol_contract()),
            &NegotiationRequirements::default(),
        )
        .expect("compatible test contract must be proposed");
    core.negotiation_mut()
        .validate_authority(connection)
        .expect("authority validation must succeed");
    core.negotiation_mut()
        .validate_peer(connection)
        .expect("peer validation must establish compatibility");
}

fn establish_runennet_connection(
    core: &mut RunenNetSessionCore,
    projection: &mut RunenNetSessionProjection,
    participant: ParticipantId,
    connection: ConnectionHandle,
) {
    establish_runennet_negotiation(core, connection);
    core.admit_established(projection, participant, connection)
        .expect("established RunenNet connection must be admitted");
}

fn install_runennet_connections(app: &mut App, bindings: &[(ConnectionHandle, ParticipantId)]) {
    let mut core = test_runennet_session_core();
    let mut projection = RunenNetSessionProjection::default();
    for (connection, participant) in bindings.iter().copied() {
        establish_runennet_connection(&mut core, &mut projection, participant, connection);
    }
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());
}

include!("network_plugins/basic_flow.rs");

include!("network_plugins/runtime_and_replication.rs");

include!("network_plugins/delta_and_reconnect.rs");

include!("network_plugins/client_replication_cutover.rs");

include!("network_plugins/replicated_view_r0.rs");
