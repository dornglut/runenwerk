pub const MAX_WORLD_RENDER_AGENTS: usize = 512;
pub const MAX_WORLD_RENDER_MODELS: usize = 1024;

#[derive(Debug, Clone)]
pub struct WorldRenderAgent {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub health_ratio: f32,
    pub team: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldRenderModelProxy {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct WorldRenderFrame {
    pub world_scene_label: String,
    pub overlay_scene_label: String,
    pub scene_render_graph_passes: Vec<crate::plugins::scene::manifest::FramePassDescriptor>,
    pub render_world: bool,
    pub world_bounds: [f32; 4],
    pub world_paused: bool,
    pub chunk_size: f32,
    pub chunk_load_radius: u32,
    pub infinite_world: bool,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub camera_pitch_min: f32,
    pub camera_pitch_max: f32,
    pub camera_distance_min: f32,
    pub camera_distance_max: f32,
    pub camera_follow_dampening: f32,
    pub camera_target: [f32; 3],
    pub camera_fov_y: f32,
    pub debug_view_mode: u32,
    pub elapsed_time_seconds: f32,
    pub render_mesh_overlay: bool,
    pub agents: Vec<WorldRenderAgent>,
    pub model_proxies: Vec<WorldRenderModelProxy>,
}

impl Default for WorldRenderFrame {
    fn default() -> Self {
        Self {
            world_scene_label: "gameplay_stub".to_string(),
            overlay_scene_label: "console_ui".to_string(),
            scene_render_graph_passes: Vec::new(),
            render_world: false,
            world_bounds: [-32.0, -18.0, 32.0, 18.0],
            world_paused: false,
            chunk_size: 24.0,
            chunk_load_radius: 2,
            infinite_world: true,
            camera_yaw: std::f32::consts::PI,
            camera_pitch: 0.45,
            camera_distance: 10.0,
            camera_pitch_min: -1.15,
            camera_pitch_max: 1.15,
            camera_distance_min: 3.0,
            camera_distance_max: 48.0,
            camera_follow_dampening: 0.12,
            camera_target: [0.0, 1.1, 0.0],
            camera_fov_y: 55.0f32.to_radians(),
            debug_view_mode: 0,
            elapsed_time_seconds: 0.0,
            render_mesh_overlay: true,
            agents: Vec::new(),
            model_proxies: Vec::new(),
        }
    }
}
