use crate::protocol::{DeltaSnapshot, InputFrame, Snapshot};
use crate::replication::ReplicationProfilePreset;
use engine_sim::SimulationTick;
use runen_net::identity::ConnectionHandle;

/// Retained local replication-runtime evidence.
///
/// ConnectionHandle is local runtime identity and deliberately has no wire/serde meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicationRuntimeEvent {
    InputAccepted {
        connection: ConnectionHandle,
        tick: SimulationTick,
    },
    SnapshotBuilt {
        tick: SimulationTick,
    },
    DeltaBuilt {
        tick: SimulationTick,
    },
    FullSnapshotSent {
        connection: ConnectionHandle,
        cursor: crate::replication::SnapshotCursor,
    },
    DeltaSnapshotSent {
        connection: ConnectionHandle,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplicationRuntimeCommand {
    IngestInput {
        connection: ConnectionHandle,
        frame: InputFrame,
    },
    BuildSnapshot {
        tick: SimulationTick,
        payload: crate::protocol::SnapshotPayload,
    },
    SendSnapshot {
        connection: ConnectionHandle,
        snapshot: Snapshot,
    },
    SendDelta {
        connection: ConnectionHandle,
        delta: DeltaSnapshot,
    },
}
