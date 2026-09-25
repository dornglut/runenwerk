use super::{
    DeltaSnapshot as EngineDeltaSnapshot, ReplicationDriver, ServerMessage,
    Snapshot as EngineSnapshot, SnapshotCursor,
};
use anyhow::{Context, anyhow};
use engine_sim::SimulationTick;
use runen_ecs::World;
use runen_net::{
    DeliveryAcceptance,
    identity::{ConnectionHandle, ParticipantId, SimulationTick as RunenNetSimulationTick},
    replication::{
        AccountedState, AuthorityAckOutcome, AuthorityReplicationSession,
        AuthorityReplicationState, AuthoritySessionError, EmittedSnapshot, FullSnapshot,
        PendingSnapshotRef, PendingSnapshotSummary, ReplicationCursor, ReplicationLineageKey,
        SnapshotKind,
    },
    session::{ConnectionLossOutcome, Session},
};
use world_ops::SyncCursor;

use super::{
    AuthorityReplicationPolicy, NetStreamingStateResource, ReplicationDiagnostics,
    RunenNetSessionCore, RunenNetSessionProjection,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AuthorityReplicationSubmissionToken {
    connection: ConnectionHandle,
    lineage: ReplicationLineageKey,
    pending: PendingSnapshotSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityReplicationSubmission {
    connection: ConnectionHandle,
    token: AuthorityReplicationSubmissionToken,
    message: ServerMessage,
}

impl AuthorityReplicationSubmission {
    pub const fn connection(&self) -> ConnectionHandle {
        self.connection
    }

    pub const fn token(&self) -> AuthorityReplicationSubmissionToken {
        self.token
    }

    pub fn message(&self) -> &ServerMessage {
        &self.message
    }

    pub fn into_parts(
        self,
    ) -> (
        ConnectionHandle,
        AuthorityReplicationSubmissionToken,
        ServerMessage,
    ) {
        (self.connection, self.token, self.message)
    }
}

#[derive(Debug)]
pub(crate) struct AuthorityReplicationIntegration {
    policy: AuthorityReplicationPolicy,
    semantic: AuthorityReplicationSession<Vec<u8>, Vec<u8>>,
}

impl AuthorityReplicationIntegration {
    fn new(session: runen_net::identity::SessionId, policy: AuthorityReplicationPolicy) -> Self {
        Self {
            policy,
            semantic: AuthorityReplicationSession::new(session, policy.aggregate_limits()),
        }
    }

    const fn policy(&self) -> AuthorityReplicationPolicy {
        self.policy
    }

    fn ensure_lineage(&mut self, participant: ParticipantId) -> Result<(), AuthoritySessionError> {
        if self.semantic.lineage(participant).is_none() {
            self.semantic
                .add_lineage(participant, self.policy.retention_limits())?;
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.semantic = AuthorityReplicationSession::new(
            self.semantic.session_id(),
            self.policy.aggregate_limits(),
        );
    }

    fn remove_lineage(&mut self, participant: ParticipantId) {
        self.semantic.remove_lineage(participant);
    }

    fn cancel_pending_for_participant(
        &mut self,
        participant: ParticipantId,
    ) -> Result<(), AuthoritySessionError> {
        if self.semantic.lineage(participant).is_some() {
            self.semantic.cancel_pending(participant)?;
        }
        Ok(())
    }

    fn connection_replaced(
        &mut self,
        session: &Session,
        connection: ConnectionHandle,
        participant: ParticipantId,
    ) -> Result<(), AuthoritySessionError> {
        if self.semantic.lineage(participant).is_some() {
            self.semantic
                .connection_replaced(session, connection, participant)?;
        }
        Ok(())
    }

    fn pending_for(&self, participant: ParticipantId) -> bool {
        self.semantic
            .lineage(participant)
            .and_then(|lineage| lineage.pending_summary())
            .is_some()
    }

    fn prepare<TDriver>(
        &mut self,
        participant: ParticipantId,
        tick: SimulationTick,
        snapshot: TDriver::Snapshot,
    ) -> anyhow::Result<Option<PendingSnapshotSummary>>
    where
        TDriver: ReplicationDriver,
    {
        self.ensure_lineage(participant).map_err(|error| {
            anyhow!("configure RunenNet authority replication lineage: {error:?}")
        })?;

        let (state, greatest_emitted, pending, retained_base) = {
            let lineage = self.semantic.lineage(participant).context(
                "RunenNet authority replication lineage disappeared after configuration",
            )?;
            let state = lineage.replication_state();
            let retained_base = match state {
                AuthorityReplicationState::DeltaEligible(base) => {
                    Some((base, lineage.retained_state(base).cloned()))
                }
                AuthorityReplicationState::FullSnapshotRequired { .. } => None,
            };
            (
                state,
                lineage.greatest_emitted_cursor(),
                lineage.pending_summary(),
                retained_base,
            )
        };

        if pending.is_some() {
            return Ok(None);
        }

        let target_cursor = match greatest_emitted {
            Some(cursor) => ReplicationCursor::new(
                cursor
                    .get()
                    .checked_add(1)
                    .context("RunenNet authority replication cursor exhausted")?,
            ),
            None => ReplicationCursor::new(1),
        };
        let target_tick = RunenNetSimulationTick::new(tick.0);
        let encoded_snapshot = TDriver::encode_snapshot(&snapshot)
            .map_err(anyhow::Error::new)
            .context("encode authority target snapshot")?;
        let snapshot_bytes = encoded_snapshot.len();

        let summary = match state {
            AuthorityReplicationState::FullSnapshotRequired { .. } => self
                .semantic
                .prepare_full(
                    participant,
                    FullSnapshot::new(
                        target_cursor,
                        target_tick,
                        AccountedState::new(encoded_snapshot, snapshot_bytes),
                    ),
                    true,
                )
                .map_err(|error| anyhow!("prepare RunenNet authority full snapshot: {error:?}"))?,
            AuthorityReplicationState::DeltaEligible(base_cursor) => {
                let (retained_cursor, base_bytes) = retained_base.context(
                    "RunenNet delta-eligible authority lineage has no retained base entry",
                )?;
                debug_assert_eq!(base_cursor, retained_cursor);
                let base_bytes = base_bytes.context(
                    "RunenNet delta-eligible authority lineage has no retained base state",
                )?;
                let base_snapshot = TDriver::decode_snapshot(&base_bytes)
                    .map_err(anyhow::Error::new)
                    .context("decode RunenNet-retained authority delta base")?;
                let delta = TDriver::build_delta(&base_snapshot, &snapshot);
                let encoded_delta = TDriver::encode_delta(&delta)
                    .map_err(anyhow::Error::new)
                    .context("encode authority delta candidate")?;
                let delta_bytes = encoded_delta.len();
                self.semantic
                    .prepare_delta(
                        participant,
                        target_cursor,
                        target_tick,
                        AccountedState::new(encoded_snapshot, snapshot_bytes),
                        encoded_delta,
                        delta_bytes,
                    )
                    .map_err(|error| {
                        anyhow!("prepare RunenNet authority delta snapshot: {error:?}")
                    })?
            }
        };

        Ok(Some(summary))
    }

    fn submission(
        &self,
        session: &Session,
        connection: ConnectionHandle,
    ) -> anyhow::Result<Option<AuthorityReplicationSubmission>> {
        let Some(participant) = session.participant_for_connection(connection) else {
            return Ok(None);
        };
        let Some(lineage) = self.semantic.lineage(participant) else {
            return Ok(None);
        };
        let Some(pending) = lineage.pending_summary() else {
            return Ok(None);
        };
        let Some(candidate) = lineage.pending_snapshot() else {
            return Ok(None);
        };
        let last_applied = SnapshotCursor(
            lineage
                .latest_confirmed_cursor()
                .map(ReplicationCursor::get)
                .unwrap_or(0),
        );

        let message = match candidate {
            PendingSnapshotRef::Full { snapshot, .. } => ServerMessage::Snapshot(EngineSnapshot {
                tick: SimulationTick(snapshot.target_tick().get()),
                cursor: SnapshotCursor(snapshot.target_cursor().get()),
                last_applied,
                payload: snapshot.image().state().clone(),
            }),
            PendingSnapshotRef::Delta {
                base_cursor,
                target_cursor,
                target_tick,
                delta,
                ..
            } => ServerMessage::DeltaSnapshot(EngineDeltaSnapshot {
                tick: SimulationTick(target_tick.get()),
                base: SnapshotCursor(base_cursor.get()),
                cursor: SnapshotCursor(target_cursor.get()),
                payload: delta.clone(),
            }),
        };

        let token = AuthorityReplicationSubmissionToken {
            connection,
            lineage: lineage.key(),
            pending,
        };
        Ok(Some(AuthorityReplicationSubmission {
            connection,
            token,
            message,
        }))
    }

    fn validate_feedback(
        &self,
        session: &Session,
        token: AuthorityReplicationSubmissionToken,
    ) -> anyhow::Result<ParticipantId> {
        if token.lineage.session() != self.semantic.session_id()
            || token.lineage.session() != session.id()
        {
            anyhow::bail!("stale authority replication submission session");
        }
        let participant = token.lineage.participant();
        if !session.is_authorized(participant, token.connection) {
            anyhow::bail!("authority replication submission connection is no longer authorized");
        }
        let lineage = self
            .semantic
            .lineage(participant)
            .context("authority replication submission lineage no longer exists")?;
        if lineage.pending_summary() != Some(token.pending) {
            anyhow::bail!("stale authority replication submission token");
        }
        Ok(participant)
    }

    fn record_delivery_acceptance(
        &mut self,
        session: &Session,
        token: AuthorityReplicationSubmissionToken,
        acceptance: DeliveryAcceptance,
    ) -> anyhow::Result<Option<EmittedSnapshot>> {
        let participant = self.validate_feedback(session, token)?;
        self.semantic
            .record_delivery_acceptance(participant, acceptance)
            .map_err(|error| anyhow!("record RunenNet authority delivery acceptance: {error:?}"))
    }

    fn cancel_submission(
        &mut self,
        session: &Session,
        token: AuthorityReplicationSubmissionToken,
    ) -> anyhow::Result<bool> {
        let participant = self.validate_feedback(session, token)?;
        self.semantic
            .cancel_pending(participant)
            .map_err(|error| anyhow!("cancel RunenNet authority replication candidate: {error:?}"))
    }

    fn acknowledge(
        &mut self,
        session: &Session,
        connection: ConnectionHandle,
        cursor: SnapshotCursor,
    ) -> anyhow::Result<Option<AuthorityAckOutcome>> {
        let Some(participant) = session.participant_for_connection(connection) else {
            return Ok(None);
        };
        if self.semantic.lineage(participant).is_none() {
            anyhow::bail!("replication ACK participant has no RunenNet authority lineage");
        }
        self.semantic
            .acknowledge_authorized(
                session,
                connection,
                participant,
                ReplicationCursor::new(cursor.0),
            )
            .map(Some)
            .map_err(|error| anyhow!("RunenNet authority ACK failed: {error:?}"))
    }
}

impl RunenNetSessionCore {
    pub fn with_authority_replication_policy(mut self, policy: AuthorityReplicationPolicy) -> Self {
        self.authority_replication = Some(AuthorityReplicationIntegration::new(
            self.session().id(),
            policy,
        ));
        self
    }

    pub fn authority_replication_policy(&self) -> Option<AuthorityReplicationPolicy> {
        self.authority_replication
            .as_ref()
            .map(AuthorityReplicationIntegration::policy)
    }

    pub(crate) fn authority_replication_pending(&self, connection: ConnectionHandle) -> bool {
        let Some(participant) = self.session.participant_for_connection(connection) else {
            return false;
        };
        self.authority_replication
            .as_ref()
            .is_some_and(|integration| integration.pending_for(participant))
    }

    pub(crate) fn prepare_authority_replication<TDriver>(
        &mut self,
        connection: ConnectionHandle,
        tick: SimulationTick,
        snapshot: TDriver::Snapshot,
    ) -> anyhow::Result<Option<PendingSnapshotSummary>>
    where
        TDriver: ReplicationDriver,
    {
        let participant = self
            .session
            .participant_for_connection(connection)
            .context("authority snapshot connection is not authorized by RunenNet session")?;
        let integration = self.authority_replication.as_mut().context(
            "authority replication requires RunenNetSessionCore::with_authority_replication_policy with explicit finite policy",
        )?;
        integration.prepare::<TDriver>(participant, tick, snapshot)
    }

    pub(crate) fn authority_replication_submission(
        &self,
        connection: ConnectionHandle,
    ) -> anyhow::Result<Option<AuthorityReplicationSubmission>> {
        let Some(integration) = self.authority_replication.as_ref() else {
            return Ok(None);
        };
        integration.submission(&self.session, connection)
    }

    pub(crate) fn record_authority_replication_delivery(
        &mut self,
        token: AuthorityReplicationSubmissionToken,
        acceptance: DeliveryAcceptance,
    ) -> anyhow::Result<Option<EmittedSnapshot>> {
        let integration = self
            .authority_replication
            .as_mut()
            .context("authority replication delivery feedback requires explicit finite policy")?;
        integration.record_delivery_acceptance(&self.session, token, acceptance)
    }

    pub(crate) fn cancel_authority_replication_submission(
        &mut self,
        token: AuthorityReplicationSubmissionToken,
    ) -> anyhow::Result<bool> {
        let integration = self
            .authority_replication
            .as_mut()
            .context("authority replication cancellation requires explicit finite policy")?;
        integration.cancel_submission(&self.session, token)
    }

    pub(crate) fn acknowledge_authority_replication(
        &mut self,
        connection: ConnectionHandle,
        cursor: SnapshotCursor,
    ) -> anyhow::Result<Option<AuthorityAckOutcome>> {
        let integration = self
            .authority_replication
            .as_mut()
            .context("authority replication ACK requires explicit finite policy")?;
        integration.acknowledge(&self.session, connection, cursor)
    }

    pub(crate) fn authority_replication_connection_lost(
        &mut self,
        participant: ParticipantId,
        outcome: ConnectionLossOutcome,
    ) -> Result<(), AuthoritySessionError> {
        let Some(integration) = self.authority_replication.as_mut() else {
            return Ok(());
        };
        match outcome {
            ConnectionLossOutcome::Terminated => integration.remove_lineage(participant),
            ConnectionLossOutcome::Retained { .. } => {
                integration.cancel_pending_for_participant(participant)?;
            }
        }
        Ok(())
    }

    pub(crate) fn authority_replication_connection_replaced(
        &mut self,
        participant: ParticipantId,
        connection: ConnectionHandle,
    ) -> Result<(), AuthoritySessionError> {
        let Some(integration) = self.authority_replication.as_mut() else {
            return Ok(());
        };
        integration.connection_replaced(&self.session, connection, participant)
    }

    pub(crate) fn remove_authority_replication_lineage(&mut self, participant: ParticipantId) {
        if let Some(integration) = self.authority_replication.as_mut() {
            integration.remove_lineage(participant);
        }
    }

    pub(crate) fn clear_authority_replication(&mut self) {
        if let Some(integration) = self.authority_replication.as_mut() {
            integration.clear();
        }
    }
}

pub fn authority_replication_submissions(
    world: &World,
) -> anyhow::Result<Vec<AuthorityReplicationSubmission>> {
    let projection = world
        .resource::<RunenNetSessionProjection>()
        .context("authority replication submissions require RunenNetSessionProjection")?;
    if projection.active_connection_count() == 0 {
        return Ok(Vec::new());
    }
    let core = world
        .resource::<RunenNetSessionCore>()
        .context("authority replication submissions require RunenNetSessionCore")?;
    let mut connections = projection.active_connections().collect::<Vec<_>>();
    connections.sort_by_key(|connection| connection.get());
    let mut submissions = Vec::new();
    for connection in connections {
        if let Some(submission) = core.authority_replication_submission(connection)? {
            submissions.push(submission);
        }
    }
    Ok(submissions)
}

pub fn record_authority_replication_delivery_acceptance(
    world: &mut World,
    token: AuthorityReplicationSubmissionToken,
    acceptance: DeliveryAcceptance,
) -> anyhow::Result<Option<EmittedSnapshot>> {
    let mut core = world
        .remove_resource::<RunenNetSessionCore>()
        .context("authority replication delivery feedback requires RunenNetSessionCore")?;
    let result = core.record_authority_replication_delivery(token, acceptance);
    world.insert_resource(core);
    let emitted = result?;

    if let Some(emitted) = emitted {
        if let Ok(streaming) = world.resource_mut::<NetStreamingStateResource>() {
            streaming.mark_snapshot_sent(
                token.connection,
                SyncCursor(emitted.target_cursor.get()),
                matches!(emitted.kind, SnapshotKind::Full),
            );
        }
        if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.last_snapshot_cursor = emitted.target_cursor.get();
            diagnostics.emitted_snapshots = diagnostics.emitted_snapshots.saturating_add(1);
        }
        return Ok(Some(emitted));
    }
    Ok(None)
}

pub fn cancel_authority_replication_submission(
    world: &mut World,
    token: AuthorityReplicationSubmissionToken,
) -> anyhow::Result<bool> {
    let mut core = world
        .remove_resource::<RunenNetSessionCore>()
        .context("authority replication cancellation requires RunenNetSessionCore")?;
    let result = core.cancel_authority_replication_submission(token);
    world.insert_resource(core);
    result
}

pub(crate) fn authority_replication_pending_for_connection(
    world: &World,
    connection: ConnectionHandle,
) -> bool {
    world
        .resource::<RunenNetSessionCore>()
        .ok()
        .is_some_and(|core| core.authority_replication_pending(connection))
}

pub(crate) fn prepare_authority_replication_candidate<TDriver>(
    world: &mut World,
    connection: ConnectionHandle,
    tick: SimulationTick,
    snapshot: TDriver::Snapshot,
) -> anyhow::Result<Option<PendingSnapshotSummary>>
where
    TDriver: ReplicationDriver,
{
    let mut core = world
        .remove_resource::<RunenNetSessionCore>()
        .context("authority replication requires RunenNetSessionCore")?;
    let result = core.prepare_authority_replication::<TDriver>(connection, tick, snapshot);
    world.insert_resource(core);
    result
}

pub(crate) fn acknowledge_authority_replication(
    world: &mut World,
    connection: ConnectionHandle,
    cursor: SnapshotCursor,
) -> anyhow::Result<Option<AuthorityAckOutcome>> {
    let mut core = world
        .remove_resource::<RunenNetSessionCore>()
        .context("authority replication ACK requires RunenNetSessionCore")?;
    let result = core.acknowledge_authority_replication(connection, cursor);
    world.insert_resource(core);
    result
}
