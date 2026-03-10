use crate::protocol::{DeltaSnapshot, InputFrame, Snapshot, SnapshotPayload};
use crate::replication::interest::{InterestContext, InterestPolicy, allows_replication};
use crate::replication::{
    LaneRouteTrace, ReplicationProfilePreset, ReplicationStats, SnapshotCursor, SnapshotTimeline,
};
use crate::transport::{ConnectionId, TransportLane, lane_for_profile};
use engine_sim::{NetEntityId, SimulationTick};
use std::collections::{BTreeMap, VecDeque};

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

#[derive(Debug, Copy, Clone, Default)]
struct ConnectionReplicationState {
    last_acknowledged: Option<SnapshotCursor>,
    force_full_snapshot: bool,
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

    pub fn mark_acknowledged(&mut self, connection_id: ConnectionId, cursor: SnapshotCursor) {
        let state = self.connection_state.entry(connection_id).or_default();
        state.last_acknowledged = Some(cursor);
        state.force_full_snapshot = false;
    }

    pub fn mark_reconnect(&mut self, connection_id: ConnectionId) {
        let state = self.connection_state.entry(connection_id).or_default();
        state.force_full_snapshot = true;
    }

    pub fn build_snapshot_for_connection(
        &mut self,
        connection_id: ConnectionId,
        tick: SimulationTick,
        payload: &SnapshotPayload,
    ) -> Result<ServerSnapshotMessage, postcard::Error> {
        let (force_full_snapshot, last_acknowledged) = {
            let state = self.connection_state.entry(connection_id).or_default();
            (state.force_full_snapshot, state.last_acknowledged)
        };

        if !force_full_snapshot {
            if let Some(base_cursor) = last_acknowledged {
                if let Some(delta) = self.build_delta_snapshot(tick, base_cursor, payload)? {
                    return Ok(ServerSnapshotMessage::Delta(delta));
                }
                self.queue_full_resync(
                    connection_id,
                    format!(
                        "baseline cursor {} unavailable, forcing full snapshot",
                        base_cursor.0
                    ),
                );
            }
        }

        let snapshot = self.build_full_snapshot(tick, payload.clone())?;
        let state = self.connection_state.entry(connection_id).or_default();
        state.force_full_snapshot = false;
        Ok(ServerSnapshotMessage::Full(snapshot))
    }

    pub fn latest_cursor(&self) -> Option<SnapshotCursor> {
        self.timeline.latest_cursor()
    }

    pub fn queue_full_resync(&mut self, connection_id: ConnectionId, reason: impl Into<String>) {
        self.stats.record_resync_request();
        self.fallback_full_resync
            .push_back((connection_id, reason.into()));
    }

    pub fn take_full_resync_requests(&mut self) -> Vec<(ConnectionId, String)> {
        self.fallback_full_resync.drain(..).collect()
    }

    pub fn route_profile(&mut self, profile: ReplicationProfilePreset) -> TransportLane {
        let trace = LaneRouteTrace::from_profile(profile);
        self.lane_trace.push(trace);
        lane_for_profile(profile)
    }

    pub fn take_lane_trace(&mut self) -> Vec<LaneRouteTrace> {
        self.lane_trace.drain(..).collect()
    }

    pub fn stats(&self) -> ReplicationStats {
        self.stats
    }

    pub fn record_stale_snapshot_drop(&mut self) {
        self.stats.record_stale_snapshot_drop();
    }

    pub fn filter_payload_for_connection<F>(
        &self,
        payload: &SnapshotPayload,
        mut should_include: F,
    ) -> SnapshotPayload
    where
        F: FnMut(NetEntityId, Option<&str>) -> bool,
    {
        let spawns = payload
            .spawns
            .iter()
            .filter(|spawn| should_include(spawn.net_entity_id, None))
            .cloned()
            .collect();
        let despawns = payload
            .despawns
            .iter()
            .filter(|despawn| should_include(despawn.net_entity_id, None))
            .cloned()
            .collect();
        let upserts = payload
            .upserts
            .iter()
            .filter(|upsert| should_include(upsert.net_entity_id, Some(&upsert.component_name)))
            .cloned()
            .collect();
        let removes = payload
            .removes
            .iter()
            .filter(|remove| should_include(remove.net_entity_id, Some(&remove.component_name)))
            .cloned()
            .collect();
        SnapshotPayload {
            spawns,
            despawns,
            upserts,
            removes,
        }
    }
}

pub fn should_send_to_connection(
    policy: InterestPolicy,
    connection_id: ConnectionId,
    owner: Option<ConnectionId>,
    same_team: bool,
    within_distance: bool,
    in_spatial_aoi: bool,
) -> bool {
    allows_replication(
        policy,
        InterestContext {
            viewer: connection_id,
            owner,
            same_team,
            within_distance,
            in_spatial_aoi,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{AuthoritativeServerRuntime, ServerSnapshotMessage};
    use crate::protocol::{ComponentUpsert, SnapshotPayload};
    use crate::replication::{ReplicationProfilePreset, SnapshotCursor};
    use crate::transport::ConnectionId;
    use engine_sim::{NetEntityId, SimulationTick};

    #[test]
    fn snapshot_flow_prefers_delta_after_client_ack() {
        let mut runtime = AuthoritativeServerRuntime::default();
        let connection = ConnectionId(1);
        let first = SnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(11),
                component_name: "Health".to_string(),
                payload: vec![100],
            }],
            ..SnapshotPayload::default()
        };
        let full = runtime
            .build_snapshot_for_connection(connection, SimulationTick(1), &first)
            .expect("full snapshot build should succeed");
        let full_cursor = match full {
            ServerSnapshotMessage::Full(snapshot) => snapshot.cursor,
            ServerSnapshotMessage::Delta(_) => panic!("first snapshot should be full"),
        };
        runtime.mark_acknowledged(connection, full_cursor);

        let second = SnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(11),
                component_name: "Health".to_string(),
                payload: vec![90],
            }],
            ..SnapshotPayload::default()
        };
        let next = runtime
            .build_snapshot_for_connection(connection, SimulationTick(2), &second)
            .expect("second snapshot build should succeed");
        assert!(matches!(
            next,
            ServerSnapshotMessage::Delta(delta) if delta.base == full_cursor
        ));
    }

    #[test]
    fn missing_baseline_requests_full_resync() {
        let mut runtime = AuthoritativeServerRuntime::default();
        let connection = ConnectionId(7);
        runtime.mark_acknowledged(connection, SnapshotCursor(99));
        let payload = SnapshotPayload::default();
        let message = runtime
            .build_snapshot_for_connection(connection, SimulationTick(10), &payload)
            .expect("fallback full snapshot should build");
        assert!(matches!(message, ServerSnapshotMessage::Full(_)));
        let requests = runtime.take_full_resync_requests();
        assert_eq!(requests.len(), 1);
    }

    #[test]
    fn route_profile_records_trace() {
        let mut runtime = AuthoritativeServerRuntime::default();
        let lane = runtime.route_profile(ReplicationProfilePreset::InputCommand);
        assert_eq!(lane, crate::transport::TransportLane::InputStream);
        let trace = runtime.take_lane_trace();
        assert_eq!(trace.len(), 1);
    }

    #[test]
    fn filter_payload_removes_disallowed_entities() {
        let runtime = AuthoritativeServerRuntime::default();
        let payload = SnapshotPayload {
            upserts: vec![
                ComponentUpsert {
                    net_entity_id: NetEntityId(1),
                    component_name: "Public".to_string(),
                    payload: vec![1],
                },
                ComponentUpsert {
                    net_entity_id: NetEntityId(2),
                    component_name: "Private".to_string(),
                    payload: vec![2],
                },
            ],
            ..SnapshotPayload::default()
        };
        let filtered = runtime.filter_payload_for_connection(&payload, |entity, _| entity.0 == 1);
        assert_eq!(filtered.upserts.len(), 1);
        assert_eq!(filtered.upserts[0].net_entity_id, NetEntityId(1));
    }
}
