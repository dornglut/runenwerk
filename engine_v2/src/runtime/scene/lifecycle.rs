use super::{SceneId, SceneLayer};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SceneLifecyclePhase {
    Enter,
    Exit,
    Pause,
    Resume,
}

#[derive(Debug, Copy, Clone)]
pub struct SceneLifecycleEvent {
    pub scene: SceneId,
    pub layer: SceneLayer,
    pub phase: SceneLifecyclePhase,
}
