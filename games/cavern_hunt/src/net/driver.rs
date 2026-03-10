use std::io;

use ecs::World;
use engine_net::replication::ReplicationDriver;

use crate::{
    CavernCommandEnvelope, CavernRunDeltaV1, CavernRunSnapshotV1, apply_cavern_run_delta,
    build_cavern_run_delta,
};

pub struct CavernReplicationDriver;

pub(super) fn into_driver_error(context: &'static str, error: anyhow::Error) -> io::Error {
    io::Error::other(format!("{context}: {error:#}"))
}

impl ReplicationDriver for CavernReplicationDriver {
    type Snapshot = CavernRunSnapshotV1;
    type Delta = CavernRunDeltaV1;
    type Input = CavernCommandEnvelope;
    type Error = io::Error;

    fn capture_snapshot(_world: &World) -> Result<Option<Self::Snapshot>, Self::Error> {
        // Transitional compatibility: Cavern Hunt currently emits chunked run events through its
        // gameplay runtime path to stay within transport datagram budgets.
        Ok(None)
    }

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta {
        build_cavern_run_delta(previous, current)
    }

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot {
        apply_cavern_run_delta(base, delta)
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}
