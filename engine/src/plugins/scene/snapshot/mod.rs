mod capture;
#[cfg(test)]
mod delta;
mod restore;

#[cfg(test)]
use super::{SceneSimulationDeltaV2, SceneSimulationSnapshotV2};

pub(crate) use capture::capture_scene_simulation_snapshot;
pub(crate) use restore::restore_scene_simulation_snapshot;

#[cfg(test)]
pub(crate) fn build_scene_simulation_delta(
    base: &SceneSimulationSnapshotV2,
    current: &SceneSimulationSnapshotV2,
) -> SceneSimulationDeltaV2 {
    delta::build_scene_simulation_delta(base, current)
}

#[cfg(test)]
pub(crate) fn apply_scene_simulation_delta(
    base: &SceneSimulationSnapshotV2,
    delta: &SceneSimulationDeltaV2,
) -> SceneSimulationSnapshotV2 {
    delta::apply_scene_simulation_delta(base, delta)
}
