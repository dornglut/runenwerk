use ecs::World;
use engine_net::SimulationTick;
use engine_net::replication::SnapshotApplyDriver;

use crate::{apply_cavern_run_delta, capture_cavern_run_snapshot, restore_cavern_run_snapshot};

use super::driver::{CavernReplicationDriver, into_driver_error};

impl SnapshotApplyDriver for CavernReplicationDriver {
    fn apply_snapshot(
        world: &mut World,
        _tick: SimulationTick,
        snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error> {
        restore_cavern_run_snapshot(world, &snapshot)
            .map_err(|error| into_driver_error("restore cavern snapshot", error))?;
        Ok(true)
    }

    fn apply_delta(
        world: &mut World,
        _tick: SimulationTick,
        delta: Self::Delta,
    ) -> Result<bool, Self::Error> {
        let base = capture_cavern_run_snapshot(world)
            .map_err(|error| into_driver_error("capture baseline cavern snapshot", error))?;
        let merged = apply_cavern_run_delta(&base, &delta);
        restore_cavern_run_snapshot(world, &merged)
            .map_err(|error| into_driver_error("restore cavern snapshot from delta", error))?;
        Ok(true)
    }
}
