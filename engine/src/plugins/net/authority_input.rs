use anyhow::{Context, anyhow};
use engine_net::protocol::InputFrame;
use engine_net::replication::InputDriver;
use engine_sim::SimulationTick;
use runen_ecs::World;
use runen_net::identity::{
    ConnectionHandle, ParticipantId, SessionId, SimulationTick as RunenNetSimulationTick,
};
use runen_net::input::{
    AuthorityInputError, AuthorityInputOutcome, AuthorityInputSession, InputWindow,
};
use runen_net::session::Session;
use std::collections::BTreeMap;

use super::{AuthorityInputPolicy, ReplicationDiagnostics, RunenNetSessionCore};

#[derive(Debug, Clone)]
struct AuthorityInputExecutionBatch {
    participant: ParticipantId,
    payload: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct AuthorityInputIntegration {
    policy: AuthorityInputPolicy,
    semantic: AuthorityInputSession<Vec<u8>>,
    execution: BTreeMap<SimulationTick, Vec<AuthorityInputExecutionBatch>>,
}

impl AuthorityInputIntegration {
    pub(crate) fn new(session: SessionId, policy: AuthorityInputPolicy) -> Self {
        Self {
            policy,
            semantic: AuthorityInputSession::new(session, policy.aggregate_limits()),
            execution: BTreeMap::new(),
        }
    }

    pub(crate) const fn policy(&self) -> AuthorityInputPolicy {
        self.policy
    }

    pub(crate) fn reconcile(
        &mut self,
        session: &Session,
    ) -> Result<Vec<ParticipantId>, AuthorityInputError> {
        let removed = self.semantic.reconcile_memberships(session)?;
        if !removed.is_empty() {
            for batches in self.execution.values_mut() {
                batches.retain(|batch| !removed.contains(&batch.participant));
            }
            self.execution.retain(|_, batches| !batches.is_empty());
        }
        Ok(removed)
    }

    pub(crate) fn clear(&mut self) {
        self.semantic =
            AuthorityInputSession::new(self.semantic.session_id(), self.policy.aggregate_limits());
        self.execution.clear();
    }

    pub(crate) fn submit(
        &mut self,
        session: &Session,
        participant: ParticipantId,
        connection: ConnectionHandle,
        current_tick: SimulationTick,
        target_tick: SimulationTick,
        payload: &[u8],
    ) -> anyhow::Result<AuthorityInputOutcome> {
        let minimum_tick = current_tick
            .0
            .checked_add(1)
            .context("engine simulation tick exhausted while deriving authority-input window")?;
        let maximum_tick = minimum_tick
            .checked_add(self.policy.max_future_tick_distance())
            .context("authority-input future horizon overflowed simulation tick")?;
        let window = InputWindow::new(
            RunenNetSimulationTick::new(minimum_tick),
            RunenNetSimulationTick::new(maximum_tick),
        )
        .map_err(|error| anyhow!("invalid RunenNet authority-input window: {error:?}"))?;

        self.execution.retain(|tick, _| tick.0 >= minimum_tick);

        if self.semantic.participant_window(participant).is_some() {
            self.semantic
                .advance_window(participant, window)
                .map_err(|error| {
                    anyhow!("failed advancing RunenNet authority-input window: {error:?}")
                })?;
        } else {
            self.semantic
                .add_participant(
                    session,
                    participant,
                    window,
                    self.policy.participant_limits(),
                )
                .map_err(|error| {
                    anyhow!("failed configuring RunenNet authority-input participant: {error:?}")
                })?;
        }

        let owned_payload = payload.to_vec();
        let outcome = self
            .semantic
            .submit(
                session,
                participant,
                connection,
                RunenNetSimulationTick::new(target_tick.0),
                &owned_payload,
                payload.len(),
            )
            .map_err(|error| anyhow!("RunenNet authority-input admission failed: {error:?}"))?;

        if matches!(outcome, AuthorityInputOutcome::InputAccepted) {
            self.execution
                .entry(target_tick)
                .or_default()
                .push(AuthorityInputExecutionBatch {
                    participant,
                    payload: owned_payload,
                });
        }

        Ok(outcome)
    }

    pub(crate) fn drain_tick(&mut self, tick: SimulationTick) -> Vec<Vec<u8>> {
        let stale_ticks = self
            .execution
            .keys()
            .copied()
            .take_while(|staged_tick| *staged_tick < tick)
            .collect::<Vec<_>>();
        for stale_tick in stale_ticks {
            self.execution.remove(&stale_tick);
        }

        self.execution
            .remove(&tick)
            .unwrap_or_default()
            .into_iter()
            .map(|batch| batch.payload)
            .collect()
    }
}

impl RunenNetSessionCore {
    pub fn with_authority_input_policy(mut self, policy: AuthorityInputPolicy) -> Self {
        self.authority_input = Some(AuthorityInputIntegration::new(self.session().id(), policy));
        self
    }

    pub fn authority_input_policy(&self) -> Option<AuthorityInputPolicy> {
        self.authority_input
            .as_ref()
            .map(AuthorityInputIntegration::policy)
    }

    pub(crate) fn reconcile_authority_input_memberships(
        &mut self,
    ) -> Result<Vec<ParticipantId>, AuthorityInputError> {
        let Some(authority_input) = self.authority_input.as_mut() else {
            return Ok(Vec::new());
        };
        authority_input.reconcile(&self.session)
    }

    pub(crate) fn clear_authority_input(&mut self) {
        if let Some(authority_input) = self.authority_input.as_mut() {
            authority_input.clear();
        }
    }

    pub(crate) fn submit_authority_input(
        &mut self,
        connection: ConnectionHandle,
        current_tick: SimulationTick,
        target_tick: SimulationTick,
        payload: &[u8],
    ) -> anyhow::Result<AuthorityInputOutcome> {
        let participant = match self.session.participant_for_connection(connection) {
            Some(participant) => participant,
            None => return Ok(AuthorityInputOutcome::UnauthorizedInput),
        };
        let authority_input = self.authority_input.as_mut().context(
            "remote authority input requires RunenNetSessionCore::with_authority_input_policy with explicit finite policy",
        )?;
        authority_input.submit(
            &self.session,
            participant,
            connection,
            current_tick,
            target_tick,
            payload,
        )
    }

    pub(crate) fn drain_authority_input_payloads(&mut self, tick: SimulationTick) -> Vec<Vec<u8>> {
        self.authority_input
            .as_mut()
            .map(|authority_input| authority_input.drain_tick(tick))
            .unwrap_or_default()
    }
}

pub(crate) fn process_authority_input_frame<TDriver>(
    world: &mut World,
    connection: ConnectionHandle,
    frame: &InputFrame,
) -> anyhow::Result<()>
where
    TDriver: InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq,
{
    let policy = world
        .resource::<RunenNetSessionCore>()
        .context("remote authority input requires RunenNetSessionCore")?
        .authority_input_policy()
        .context(
            "remote authority input requires RunenNetSessionCore::with_authority_input_policy with explicit finite policy",
        )?;

    if frame.payload.len() <= policy.participant_limits().max_batch_bytes() {
        TDriver::decode_input(&frame.payload)
            .map_err(anyhow::Error::new)
            .context("decode remote input")?;
    }

    let current_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    let mut core = world
        .remove_resource::<RunenNetSessionCore>()
        .context("RunenNetSessionCore disappeared during authority-input admission")?;
    let outcome_result =
        core.submit_authority_input(connection, current_tick, frame.tick, &frame.payload);
    world.insert_resource(core);
    let outcome = outcome_result?;

    match outcome {
        AuthorityInputOutcome::InputAccepted => {}
        AuthorityInputOutcome::DuplicateInput => {
            if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.duplicate_inputs = diagnostics.duplicate_inputs.saturating_add(1);
            }
        }
        AuthorityInputOutcome::ConflictingInput => {
            if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.conflicting_inputs = diagnostics.conflicting_inputs.saturating_add(1);
            }
        }
        AuthorityInputOutcome::StaleInput => {
            if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.lagged = diagnostics.lagged.saturating_add(1);
            }
        }
        AuthorityInputOutcome::FutureInputOutsideWindow => {
            if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.future_inputs = diagnostics.future_inputs.saturating_add(1);
            }
        }
        AuthorityInputOutcome::InputResourceRejected => {
            if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.input_resource_rejections =
                    diagnostics.input_resource_rejections.saturating_add(1);
            }
        }
        AuthorityInputOutcome::UnauthorizedInput => {
            if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.unauthorized_inputs = diagnostics.unauthorized_inputs.saturating_add(1);
            }
        }
    }

    Ok(())
}

pub(crate) fn drain_authority_input_for_tick<TDriver>(
    world: &mut World,
    tick: SimulationTick,
) -> anyhow::Result<Vec<TDriver::Input>>
where
    TDriver: InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq,
{
    let Some(mut core) = world.remove_resource::<RunenNetSessionCore>() else {
        return Ok(Vec::new());
    };
    let payloads = core.drain_authority_input_payloads(tick);
    world.insert_resource(core);

    let mut decoded = Vec::new();
    for payload in payloads {
        let mut batch = TDriver::decode_input(&payload)
            .map_err(anyhow::Error::new)
            .context("decode accepted authority input for execution")?;
        decoded.append(&mut batch);
    }
    Ok(decoded)
}
