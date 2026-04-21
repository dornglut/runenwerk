use super::domain::{
    OverlaySceneRuntime, SceneChannels, SceneCommand, SceneRegistry, SceneSlot, WorldSceneRuntime,
};

#[derive(Default, ecs::Component, ecs::Resource)]
pub(crate) struct SceneResource {
    pub(crate) manager: Option<SceneManager>,
}

pub(crate) struct SceneManager {
    pub(crate) world: SceneSlot,
    pub(crate) world_runtime: WorldSceneRuntime,
    pub(crate) overlay_runtime: OverlaySceneRuntime,
    pub(crate) registry: SceneRegistry,
    pub(crate) overlay_back_stack: Vec<(SceneSlot, OverlaySceneRuntime)>,
    pub(crate) channels: SceneChannels,
    pub(crate) overlays: Vec<SceneSlot>,
    pub(crate) pending: Vec<SceneCommand>,
}

pub use super::replay::codec::{
    SceneEntitySnapshotV2, SceneReplayArchive, SceneReplayInputFrameV2, SceneSimulationSnapshotV2,
    SceneWorldContextSnapshotV2,
};
