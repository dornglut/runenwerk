mod capture;
mod delta;
mod restore;

use super::{SceneSimulationDeltaV2, SceneSimulationSnapshotV2};

pub(crate) use capture::capture_scene_simulation_snapshot;
pub(crate) use restore::restore_scene_simulation_snapshot;

pub(crate) fn build_scene_simulation_delta(
    base: &SceneSimulationSnapshotV2,
    current: &SceneSimulationSnapshotV2,
) -> SceneSimulationDeltaV2 {
    delta::build_scene_simulation_delta(base, current)
}

pub(crate) fn apply_scene_simulation_delta(
    base: &SceneSimulationSnapshotV2,
    delta: &SceneSimulationDeltaV2,
) -> SceneSimulationSnapshotV2 {
    delta::apply_scene_simulation_delta(base, delta)
}
