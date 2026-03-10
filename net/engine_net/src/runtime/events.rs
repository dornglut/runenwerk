use crate::protocol::{DeltaSnapshot, InputFrame, Snapshot};
use crate::replication::ReplicationProfilePreset;
use crate::transport::ConnectionId;
use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicationRuntimeEvent {
    InputAccepted {
        connection_id: ConnectionId,
        tick: SimulationTick,
    },
    SnapshotBuilt {
        tick: SimulationTick,
    },
    DeltaBuilt {
        tick: SimulationTick,
    },
    FullSnapshotSent {
        connection_id: ConnectionId,
        cursor: crate::replication::SnapshotCursor,
    },
    DeltaSnapshotSent {
        connection_id: ConnectionId,
        cursor: crate::replication::SnapshotCursor,
    },
    SnapshotApplied {
        tick: SimulationTick,
        cursor: crate::replication::SnapshotCursor,
    },
    ResyncRequired {
        reason: String,
    },
    StaleSnapshotDropped {
        tick: SimulationTick,
    },
    LaneRouted {
        profile: ReplicationProfilePreset,
        lane: crate::transport::TransportLane,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicationRuntimeCommand {
    IngestInput {
        connection_id: ConnectionId,
        frame: InputFrame,
    },
    BuildSnapshot {
        tick: SimulationTick,
        payload: crate::protocol::SnapshotPayload,
    },
    SendSnapshot {
        connection_id: ConnectionId,
        snapshot: Snapshot,
    },
    SendDelta {
        connection_id: ConnectionId,
        delta: DeltaSnapshot,
    },
}
