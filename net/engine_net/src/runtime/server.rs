use crate::protocol::{DeltaSnapshot, InputFrame, Snapshot, SnapshotPayload};
use crate::replication::interest::{InterestContext, InterestPolicy, allows_replication};
use crate::replication::{
    LaneRouteTrace, ReplicationProfilePreset, ReplicationStats, SnapshotAckOutcome,
    SnapshotAckRejection, SnapshotCursor, SnapshotTimeline,
};
use crate::transport::{ConnectionId, TransportLane, lane_for_profile};
use engine_sim::{NetEntityId, SimulationTick};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type InputValidationHook =
    Box<dyn Fn(ConnectionId, &InputFrame) -> Result<(), String> + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct QueuedInput {
    pub connection_id: ConnectionId,
    pub frame: InputFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerSnapshotMessage {
    Full(Snapshot),
    Delta(DeltaSnapshot),
}

impl ServerSnapshotMessage {
    pub fn cursor(&self) -> SnapshotCursor {
        match self {
            Self::Full(snapshot) => snapshot.cursor,
            Self::Delta(delta) => delta.cursor,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ConnectionReplicationState {
    last_acknowledged: Option<SnapshotCursor>,
    force_full_snapshot: bool,
    sent_cursors: BTreeSet<SnapshotCursor>,
}

#[derive(Default)]
pub struct AuthoritativeServerRuntime {
    timeline: SnapshotTimeline,
    input_by_tick: BTreeMap<SimulationTick, Vec<QueuedInput>>,
    validation_hook: Option<InputValidationHook>,
    connection_state: BTreeMap<ConnectionId, ConnectionReplicationState>,
    fallback_full_resync: VecDeque<(ConnectionId, String)>,
    lane_trace: Vec<LaneRouteTrace>,
    stats: ReplicationStats,
}

impl AuthoritativeServerRuntime {
    pub fn set_validation_hook(&mut self, hook: InputValidationHook) {
        self.validation_hook = Some(hook);
    }

    pub fn ingest_input(
        &mut self,
        connection_id: ConnectionId,
        frame: InputFrame,
    ) -> Result<(), String> {
        if let Some(validate) = &self.validation_hook {
            validate(connection_id, &frame)?;
        }
        self.input_by_tick
            .entry(frame.tick)
            .or_default()
            .push(QueuedInput {
                connection_id,
                frame,
            });
        Ok(())
    }

    pub fn drain_inputs_for_tick(&mut self, tick: SimulationTick) -> Vec<QueuedInput> {
        self.input_by_tick.remove(&tick).unwrap_or_default()
    }

    pub fn build_full_snapshot(
        &mut self,
        tick: SimulationTick,
        payload: SnapshotPayload,
    ) -> Result<Snapshot, postcard::Error> {
        let snapshot = self.timeline.build_full_snapshot(tick, payload)?;
        self.stats.record_full_snapshot(snapshot.payload.len());
        Ok(snapshot)
    }

    pub fn build_delta_snapshot(
        &mut self,
        tick: SimulationTick,
        base_cursor: SnapshotCursor,
        payload: &SnapshotPayload,
    ) -> Result<Option<DeltaSnapshot>, postcard::Error> {
        let delta = self
            .timeline
            .build_delta_snapshot(tick, base_cursor, payload)?;
        if let Some(delta) = &delta {
            self.stats.record_delta_snapshot(delta.payload.len());
        }
        Ok(delta)
    }

    pub fn mark_acknowledged(
        &mut self,
        connection_id: ConnectionId,
        cursor: SnapshotCursor,
    ) -> SnapshotAckOutcome {
        let outcome = self.validate_snapshot_ack(connection_id, cursor);
        match outcome {
            SnapshotAckOutcome::Accepted { .. } => {
                let state = self.connection_state.entry(connection_id).or_default();
                state.last_acknowledged = Some(cursor);
                state.force_full_snapshot = false;
                self.stats.record_snapshot_ack_accepted();
            }
            SnapshotAckOutcome::Rejected { .. } => {
                self.stats.record_snapshot_ack_rejected();
            }
        }
        outcome
    }

    fn validate_snapshot_ack(
        &self,
        connection_id: ConnectionId,
        cursor: SnapshotCursor,
    ) -> SnapshotAckOutcome {
        let Some(state) = self.connection_state.get(&connection_id) else {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::UnsentCursor,
            };
        };
        if let Some(last_acknowledged) = state.last_acknowledged
            && cursor <= last_acknowledged
        {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::StaleCursor { last_acknowledged },
            };
        }
        if let Some(latest_cursor) = self.latest_cursor()
            && cursor > latest_cursor
        {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::FutureCursor { latest_cursor },
            };
        }
        if !state.sent_cursors.contains(&cursor) {
            return Err(GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::InternalInvariant,
                "prepared reservation disappeared before publication",
            ));
        }
    }
