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

#[derive(Debug, Clone)]
pub struct RenderFrameData {
    pub world_scene_label: String,
    pub overlay_scene_label: String,
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

impl RenderFrameData {
    pub fn from_world_frame(world_frame: &WorldRenderFrame) -> Self {
        Self {
            world_scene_label: world_frame.world_scene_label.clone(),
            overlay_scene_label: world_frame.overlay_scene_label.clone(),
            render_world: world_frame.render_world,
            world_bounds: world_frame.world_bounds,
            world_paused: world_frame.world_paused,
            chunk_size: world_frame.chunk_size,
            chunk_load_radius: world_frame.chunk_load_radius,
            infinite_world: world_frame.infinite_world,
            camera_yaw: world_frame.camera_yaw,
            camera_pitch: world_frame.camera_pitch,
            camera_distance: world_frame.camera_distance,
            camera_pitch_min: world_frame.camera_pitch_min,
            camera_pitch_max: world_frame.camera_pitch_max,
            camera_distance_min: world_frame.camera_distance_min,
            camera_distance_max: world_frame.camera_distance_max,
            camera_follow_dampening: world_frame.camera_follow_dampening,
            camera_target: world_frame.camera_target,
            camera_fov_y: world_frame.camera_fov_y,
            debug_view_mode: world_frame.debug_view_mode,
            elapsed_time_seconds: world_frame.elapsed_time_seconds,
            render_mesh_overlay: world_frame.render_mesh_overlay,
            agents: world_frame.agents.clone(),
            model_proxies: world_frame.model_proxies.clone(),
        }
    }
}

impl Default for RenderFrameData {
    fn default() -> Self {
        Self::from_world_frame(&WorldRenderFrame::default())
    }
}
