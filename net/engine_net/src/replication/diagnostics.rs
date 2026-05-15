use crate::protocol::{DeltaSnapshot, DeltaSnapshotPayload, Snapshot, SnapshotPayload};
use crate::replication::{NetEntityMapEvent, ReplicationProfilePreset};
use crate::transport::{DeliveryGuarantee, TransportLane, lane_for_profile, semantics_for_lane};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDebugDump {
    pub tick: u64,
    pub cursor: u64,
    pub entity_count: usize,
    pub spawn_count: usize,
    pub despawn_count: usize,
    pub upsert_count: usize,
    pub remove_count: usize,
    pub payload_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaDebugDump {
    pub tick: u64,
    pub base_cursor: u64,
    pub cursor: u64,
    pub entity_count: usize,
    pub spawn_count: usize,
    pub despawn_count: usize,
    pub upsert_count: usize,
    pub remove_count: usize,
    pub payload_bytes: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneRouteTrace {
    pub profile: ReplicationProfilePreset,
    pub lane: TransportLane,
    pub guarantee: DeliveryGuarantee,
    pub ordered: bool,
}

impl LaneRouteTrace {
    pub fn from_profile(profile: ReplicationProfilePreset) -> Self {
        let lane = lane_for_profile(profile);
        let semantics = semantics_for_lane(lane);
        Self {
            profile,
            lane,
            guarantee: semantics.guarantee,
            ordered: semantics.ordered,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMapTrace {
    pub event: NetEntityMapEvent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotAckRejection {
    StaleCursor {
        last_acknowledged: crate::replication::SnapshotCursor,
    },
    FutureCursor {
        latest_cursor: crate::replication::SnapshotCursor,
    },
    UnsentCursor,
    PrunedCursor,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotAckOutcome {
    Accepted {
        cursor: crate::replication::SnapshotCursor,
    },
    Rejected {
        cursor: crate::replication::SnapshotCursor,
        reason: SnapshotAckRejection,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReplicationStats {
    pub full_snapshots_built: u64,
    pub delta_snapshots_built: u64,
    pub full_snapshot_bytes: u64,
    pub delta_snapshot_bytes: u64,
    pub resync_requests: u64,
    pub stale_snapshots_dropped: u64,
    pub snapshot_acks_accepted: u64,
    pub snapshot_acks_rejected: u64,
}

impl ReplicationStats {
    pub fn record_full_snapshot(&mut self, bytes: usize) {
        self.full_snapshots_built = self.full_snapshots_built.saturating_add(1);
        self.full_snapshot_bytes = self.full_snapshot_bytes.saturating_add(bytes as u64);
    }

    pub fn record_delta_snapshot(&mut self, bytes: usize) {
        self.delta_snapshots_built = self.delta_snapshots_built.saturating_add(1);
        self.delta_snapshot_bytes = self.delta_snapshot_bytes.saturating_add(bytes as u64);
    }

    pub fn record_resync_request(&mut self) {
        self.resync_requests = self.resync_requests.saturating_add(1);
    }

    pub fn record_stale_snapshot_drop(&mut self) {
        self.stale_snapshots_dropped = self.stale_snapshots_dropped.saturating_add(1);
    }

    pub fn record_snapshot_ack_accepted(&mut self) {
        self.snapshot_acks_accepted = self.snapshot_acks_accepted.saturating_add(1);
    }

    pub fn record_snapshot_ack_rejected(&mut self) {
        self.snapshot_acks_rejected = self.snapshot_acks_rejected.saturating_add(1);
    }
}

pub fn snapshot_debug_dump(snapshot: &Snapshot, payload: &SnapshotPayload) -> SnapshotDebugDump {
    SnapshotDebugDump {
        tick: snapshot.tick.0,
        cursor: snapshot.cursor.0,
        entity_count: snapshot.entity_ids.len(),
        spawn_count: payload.spawns.len(),
        despawn_count: payload.despawns.len(),
        upsert_count: payload.upserts.len(),
        remove_count: payload.removes.len(),
        payload_bytes: snapshot.payload.len(),
    }
}

pub fn delta_debug_dump(delta: &DeltaSnapshot, payload: &DeltaSnapshotPayload) -> DeltaDebugDump {
    DeltaDebugDump {
        tick: delta.tick.0,
        base_cursor: delta.base.0,
        cursor: delta.cursor.0,
        entity_count: delta.entity_ids.len(),
        spawn_count: payload.spawns.len(),
        despawn_count: payload.despawns.len(),
        upsert_count: payload.upserts.len(),
        remove_count: payload.removes.len(),
        payload_bytes: delta.payload.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::{LaneRouteTrace, ReplicationStats, snapshot_debug_dump};
    use crate::protocol::{Snapshot, SnapshotPayload};
    use crate::replication::{ReplicationProfilePreset, SnapshotCursor};
    use engine_sim::{NetEntityId, SimulationTick};

    #[test]
    fn snapshot_dump_reports_counts() {
        let payload = SnapshotPayload::default();
        let snapshot = Snapshot {
            tick: SimulationTick(10),
            cursor: SnapshotCursor(4),
            last_applied: SnapshotCursor(3),
            entity_ids: vec![NetEntityId(7)],
            payload: vec![1, 2, 3],
        };
        let dump = snapshot_debug_dump(&snapshot, &payload);
        assert_eq!(dump.tick, 10);
        assert_eq!(dump.cursor, 4);
        assert_eq!(dump.entity_count, 1);
        assert_eq!(dump.payload_bytes, 3);
    }

    #[test]
    fn lane_trace_maps_profile_to_expected_lane() {
        let trace = LaneRouteTrace::from_profile(ReplicationProfilePreset::InputCommand);
        assert_eq!(trace.lane, crate::transport::TransportLane::InputStream);
    }

    #[test]
    fn stats_accumulate_counts() {
        let mut stats = ReplicationStats::default();
        stats.record_full_snapshot(12);
        stats.record_delta_snapshot(8);
        stats.record_resync_request();
        stats.record_stale_snapshot_drop();
        stats.record_snapshot_ack_accepted();
        stats.record_snapshot_ack_rejected();
        assert_eq!(stats.full_snapshots_built, 1);
        assert_eq!(stats.delta_snapshots_built, 1);
        assert_eq!(stats.full_snapshot_bytes, 12);
        assert_eq!(stats.delta_snapshot_bytes, 8);
        assert_eq!(stats.resync_requests, 1);
        assert_eq!(stats.stale_snapshots_dropped, 1);
        assert_eq!(stats.snapshot_acks_accepted, 1);
        assert_eq!(stats.snapshot_acks_rejected, 1);
    }
}
