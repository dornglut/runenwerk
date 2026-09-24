use anyhow::{Context, anyhow};
use engine_net::protocol::{
    Ack, DeltaSnapshot as EngineDeltaSnapshot, Snapshot as EngineSnapshot, SnapshotCursor,
};
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_sim::SimulationTick;
use runen_ecs::World;
use runen_net::identity::SimulationTick as RunenNetSimulationTick;
use runen_net::replication::{
    AccountedState, ClientApplyError, ClientReplicationSet, ClientSnapshotOutcome,
    DeltaReconstructionError, DeltaSnapshot, FullSnapshot, ReplicationCursor,
};
use std::sync::Arc;

use super::{
    ClientReplicationPolicy, PredictionDiagnostics, ReplicationDiagnostics,
    replay_pending_prediction,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientReplicatedStateProduct {
    bytes: Arc<[u8]>,
}

impl ClientReplicatedStateProduct {
    fn from_vec(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn accounted_bytes(&self) -> usize {
        self.bytes.len()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, runen_ecs::Resource)]
pub struct ActiveClientReplicatedStateProduct {
    active: Option<ClientReplicatedStateProduct>,
}

impl ActiveClientReplicatedStateProduct {
    pub fn active(&self) -> Option<&ClientReplicatedStateProduct> {
        self.active.as_ref()
    }

    fn activate(&mut self, candidate: ClientReplicatedStateProduct) {
        let _previous = self.active.replace(candidate);
    }
}

#[derive(Debug, runen_ecs::Resource)]
pub(crate) struct ClientReplicationIntegration {
    policy: ClientReplicationPolicy,
    semantic: ClientReplicationSet<ClientReplicatedStateProduct>,
}

impl ClientReplicationIntegration {
    fn new(policy: ClientReplicationPolicy) -> Self {
        let mut semantic = ClientReplicationSet::new(policy.aggregate_limits());
        semantic
            .add_lineage(policy.lineage(), policy.retention_limits())
            .expect("explicit client replication policy must admit its single configured lineage");
        Self { policy, semantic }
    }

    fn acknowledgement(&self) -> Option<(SnapshotCursor, SimulationTick)> {
        let lineage = self.semantic.lineage(self.policy.lineage())?;
        let cursor = lineage.acknowledgement_cursor()?;
        let tick = lineage.current_tick()?;
        Some((SnapshotCursor(cursor.get()), SimulationTick(tick.get())))
    }
}

pub(crate) fn configure_client_replication(
    app: &mut crate::App,
    policy: Option<ClientReplicationPolicy>,
) {
    let Some(policy) = policy else {
        return;
    };
    app.init_resource::<ActiveClientReplicatedStateProduct>();
    app.insert_resource(ClientReplicationIntegration::new(policy));
}

pub fn client_replication_acknowledgement(
    world: &World,
) -> Option<(SnapshotCursor, SimulationTick)> {
    world
        .resource::<ClientReplicationIntegration>()
        .ok()
        .and_then(ClientReplicationIntegration::acknowledgement)
}

pub fn client_replication_lineage(
    world: &World,
) -> Option<runen_net::replication::ReplicationLineageKey> {
    world
        .resource::<ClientReplicationIntegration>()
        .ok()
        .map(|integration| integration.policy.lineage())
}

pub fn client_replication_state(
    world: &World,
) -> Option<runen_net::replication::ClientReplicationState> {
    let integration = world.resource::<ClientReplicationIntegration>().ok()?;
    integration
        .semantic
        .lineage(integration.policy.lineage())
        .map(|lineage| lineage.replication_state())
}

pub fn require_client_replication_connection_replacement(world: &mut World) -> anyhow::Result<()> {
    let integration = world
        .resource_mut::<ClientReplicationIntegration>()
        .context("client replication requires explicit ClientReplicationPolicy")?;
    let lineage = integration.policy.lineage();
    integration
        .semantic
        .require_connection_replacement_full(lineage)
        .map_err(|error| anyhow!("RunenNet client replacement recovery failed: {error:?}"))
}

#[derive(Debug)]
pub(crate) struct ClientReplicationProcessResult {
    pub(crate) outcome: ClientSnapshotOutcome,
    pub(crate) acknowledgement: Option<Ack>,
    pub(crate) corrected: bool,
}

fn realize_committed_product<TDriver>(world: &mut World) -> anyhow::Result<(Ack, bool)>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    let (cursor, tick) = client_replication_acknowledgement(world)
        .context("RunenNet committed client lineage has no acknowledgement cursor/tick")?;
    let product = world
        .resource::<ActiveClientReplicatedStateProduct>()
        .context("client replicated-state product owner is unavailable")?
        .active()
        .cloned()
        .context("RunenNet committed client lineage has no active host product")?;
    let snapshot = TDriver::decode_snapshot(product.bytes())
        .map_err(anyhow::Error::new)
        .context("decode committed client replicated-state product")?;
    let corrected = TDriver::apply_snapshot(world, tick, snapshot)
        .map_err(anyhow::Error::new)
        .context("realize committed client replicated-state product")?;

    if let Ok(tick_resource) = world.resource_mut::<SimulationTick>() {
        *tick_resource = tick;
    }
    replay_pending_prediction::<TDriver>(
        world,
        tick,
        "replay predicted input after client commit",
    )?;

    Ok((
        Ack {
            cursor,
            last_received_tick: tick,
        },
        corrected,
    ))
}

fn finalize_outcome<TDriver>(
    world: &mut World,
    outcome: ClientSnapshotOutcome,
) -> anyhow::Result<ClientReplicationProcessResult>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    if matches!(
        outcome,
        ClientSnapshotOutcome::Committed(_) | ClientSnapshotOutcome::DuplicateCurrent
    ) {
        let (acknowledgement, corrected) = realize_committed_product::<TDriver>(world)?;
        if matches!(outcome, ClientSnapshotOutcome::Committed(_))
            && let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>()
        {
            diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
        }
        if corrected && let Ok(diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
            diagnostics.corrected = diagnostics.corrected.saturating_add(1);
        }
        return Ok(ClientReplicationProcessResult {
            outcome,
            acknowledgement: Some(acknowledgement),
            corrected,
        });
    }

    Ok(ClientReplicationProcessResult {
        outcome,
        acknowledgement: None,
        corrected: false,
    })
}

pub(crate) fn process_client_full_snapshot<TDriver>(
    world: &mut World,
    snapshot: &EngineSnapshot,
) -> anyhow::Result<ClientReplicationProcessResult>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    TDriver::decode_snapshot(&snapshot.payload)
        .map_err(anyhow::Error::new)
        .context("validate full snapshot payload before RunenNet commit")?;

    let product = ClientReplicatedStateProduct::from_vec(snapshot.payload.clone());
    let accounted_bytes = product.accounted_bytes();
    let runennet_snapshot = FullSnapshot::new(
        ReplicationCursor::new(snapshot.cursor.0),
        RunenNetSimulationTick::new(snapshot.tick.0),
        AccountedState::new(product, accounted_bytes),
    );

    let mut integration = world
        .remove_resource::<ClientReplicationIntegration>()
        .context("snapshot replication requires explicit ClientReplicationPolicy")?;
    let lineage = integration.policy.lineage();
    let apply = integration
        .semantic
        .apply_full(lineage, runennet_snapshot, |product| {
            world
                .resource_mut::<ActiveClientReplicatedStateProduct>()
                .map(|active| active.activate(product.clone()))
                .map_err(|_| "client replicated-state product owner is unavailable")
        });
    world.insert_resource(integration);

    let outcome = apply.map_err(|error: ClientApplyError<&'static str>| {
        anyhow!("RunenNet full snapshot apply failed: {error:?}")
    })?;
    finalize_outcome::<TDriver>(world, outcome)
}

pub(crate) fn process_client_delta_snapshot<TDriver>(
    world: &mut World,
    snapshot: &EngineDeltaSnapshot,
) -> anyhow::Result<ClientReplicationProcessResult>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    let runennet_snapshot = DeltaSnapshot::new(
        ReplicationCursor::new(snapshot.base.0),
        ReplicationCursor::new(snapshot.cursor.0),
        RunenNetSimulationTick::new(snapshot.tick.0),
        snapshot.payload.clone(),
    );

    let mut integration = world
        .remove_resource::<ClientReplicationIntegration>()
        .context("delta replication requires explicit ClientReplicationPolicy")?;
    let lineage = integration.policy.lineage();
    let apply = integration.semantic.apply_delta(
        lineage,
        runennet_snapshot,
        |base_product, delta_bytes, _candidate_limit| {
            let base = TDriver::decode_snapshot(base_product.bytes())
                .map_err(|_| DeltaReconstructionError::ReconstructionFailed)?;
            let delta = TDriver::decode_delta(delta_bytes)
                .map_err(|_| DeltaReconstructionError::Malformed)?;
            let rebuilt = TDriver::apply_delta_to_snapshot(&base, &delta);
            let bytes = TDriver::encode_snapshot(&rebuilt)
                .map_err(|_| DeltaReconstructionError::ReconstructionFailed)?;
            let product = ClientReplicatedStateProduct::from_vec(bytes);
            let accounted_bytes = product.accounted_bytes();
            Ok(AccountedState::new(product, accounted_bytes))
        },
        |product| {
            world
                .resource_mut::<ActiveClientReplicatedStateProduct>()
                .map(|active| active.activate(product.clone()))
                .map_err(|_| "client replicated-state product owner is unavailable")
        },
    );
    world.insert_resource(integration);

    let outcome = apply.map_err(|error: ClientApplyError<&'static str>| {
        anyhow!("RunenNet delta snapshot apply failed: {error:?}")
    })?;
    finalize_outcome::<TDriver>(world, outcome)
}
