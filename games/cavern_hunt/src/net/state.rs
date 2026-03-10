use std::collections::BTreeMap;

use crate::{CavernRunSnapshotV1, ReplicationCursor};

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CavernNetRuntimeState {
    pub active_connection_id: Option<u64>,
    pub initial_snapshot_sent: bool,
    pub last_cursor: u64,
    pub last_sent_snapshot: Option<CavernRunSnapshotV1>,
    pub last_sent_geometry_edit_count: usize,
    pub last_received_cursor: u64,
    pub last_received_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ServerReplicationByConnectionState {
    pub cursors_by_connection: BTreeMap<u64, ReplicationCursor>,
    pub latest_cursor: ReplicationCursor,
    pub last_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ClientReplicationState {
    pub last_cursor: ReplicationCursor,
    pub has_keyframe: bool,
    pub remote_targets_by_player_id: BTreeMap<u32, RemotePlayerTarget>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct RemotePlayerTarget {
    pub pos: [f32; 2],
    pub velocity: [f32; 2],
    pub yaw: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub(crate) struct NetSyncDiagnosticsLogState {
    pub last_logged_tick: u64,
}

pub(crate) fn angle_delta(current: f32, target: f32) -> f32 {
    let mut delta = target - current;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}
