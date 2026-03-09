mod capture;
mod delta;
mod restore;

use super::{SceneSimulationDeltaV1, SceneSimulationSnapshotV1};

pub(crate) use capture::capture_scene_simulation_snapshot;
pub(crate) use restore::restore_scene_simulation_snapshot;

pub(crate) fn build_scene_simulation_delta(
    base: &SceneSimulationSnapshotV1,
    current: &SceneSimulationSnapshotV1,
) -> SceneSimulationDeltaV1 {
    delta::build_scene_simulation_delta(base, current)
}

pub(crate) fn apply_scene_simulation_delta(
    base: &SceneSimulationSnapshotV1,
    delta: &SceneSimulationDeltaV1,
) -> SceneSimulationSnapshotV1 {
    delta::apply_scene_simulation_delta(base, delta)
}
