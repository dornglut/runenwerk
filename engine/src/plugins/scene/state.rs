use runen_ecs::Component;

#[derive(Debug, Clone, PartialEq, Component, runen_ecs::Resource)]
pub struct SceneRuntimeState {
    pub world_scene_label: String,
    pub overlay_scene_label: String,
    pub overlay_visible: bool,
    pub world_paused: bool,
    pub enemy_kills: u32,
}

impl Default for SceneRuntimeState {
    fn default() -> Self {
        Self {
            world_scene_label: "gameplay_stub".to_string(),
            overlay_scene_label: "console_ui".to_string(),
            overlay_visible: false,
            world_paused: false,
            enemy_kills: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Component, runen_ecs::Resource)]
pub struct SceneOverlayViewportState {
    pub screen_size: (f32, f32),
    pub scale: f32,
}

impl Default for SceneOverlayViewportState {
    fn default() -> Self {
        Self {
            screen_size: (1280.0, 720.0),
            scale: 1.0,
        }
    }
}
