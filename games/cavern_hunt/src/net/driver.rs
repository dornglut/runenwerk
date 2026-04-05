use std::io;

use ecs::World;
use engine_net::replication::ReplicationDriver;

use crate::domain::snapshot::types_and_bundles::{
    CavernRunDeltaV1, CavernRunDeltaV3, CavernRunSnapshotV1, CavernRunSnapshotV3,
};
use crate::{
    CavernCommandEnvelope, apply_cavern_run_delta, build_cavern_run_delta,
    capture_cavern_run_snapshot, capture_world_checkpoint,
};

pub struct CavernReplicationDriver;

pub(super) fn into_driver_error(context: &'static str, error: anyhow::Error) -> io::Error {
    io::Error::other(format!("{context}: {error:#}"))
}

impl ReplicationDriver for CavernReplicationDriver {
    type Snapshot = CavernRunSnapshotV3;
    type Delta = CavernRunDeltaV3;
    type Input = CavernCommandEnvelope;
    type Error = io::Error;

    fn capture_snapshot(world: &World) -> Result<Option<Self::Snapshot>, Self::Error> {
        let snapshot = capture_cavern_run_snapshot(world)
            .map_err(|error| into_driver_error("capture cavern snapshot", error))?;
        Ok(Some(snapshot))
    }

    fn capture_snapshot_for_connection(
        world: &World,
        connection_id: engine_net::ConnectionId,
    ) -> Result<Option<Self::Snapshot>, Self::Error> {
        let mut snapshot = capture_cavern_run_snapshot(world)
            .map_err(|error| into_driver_error("capture cavern snapshot for connection", error))?;
        snapshot.world_checkpoint = capture_world_checkpoint(world, Some(connection_id));
        Ok(Some(snapshot))
    }

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta {
        build_cavern_run_delta(previous, current)
    }

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot {
        apply_cavern_run_delta(base, delta)
    }

    fn decode_snapshot(bytes: &[u8]) -> Result<Self::Snapshot, Self::Error> {
        match postcard::from_bytes::<CavernRunSnapshotV3>(bytes) {
            Ok(snapshot) => {
                if snapshot.wire_version != 3 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "unsupported cavern snapshot version: {} (expected V3)",
                            snapshot.wire_version
                        ),
                    ));
                }
                Ok(snapshot)
            }
            Err(v2_error) => {
                if postcard::from_bytes::<CavernRunSnapshotV1>(bytes).is_ok() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported cavern snapshot version: V1 (expected V3)",
                    ));
                }
                Err(Self::map_codec_error(v2_error))
            }
        }
    }

    fn decode_delta(bytes: &[u8]) -> Result<Self::Delta, Self::Error> {
        match postcard::from_bytes::<CavernRunDeltaV3>(bytes) {
            Ok(delta) => {
                if delta.wire_version != 3 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "unsupported cavern delta version: {} (expected V3)",
                            delta.wire_version
                        ),
                    ));
                }
                Ok(delta)
            }
            Err(v2_error) => {
                if postcard::from_bytes::<CavernRunDeltaV1>(bytes).is_ok() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported cavern delta version: V1 (expected V3)",
                    ));
                }
                Err(Self::map_codec_error(v2_error))
            }
        }
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}
