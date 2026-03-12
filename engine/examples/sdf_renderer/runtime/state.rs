// Owner: SDF Renderer Example - Runtime State
use crate::*;

pub(crate) const ACTION_UP: &str = "sdf.move_up";
pub(crate) const ACTION_DOWN: &str = "sdf.move_down";
pub(crate) const ACTION_DEBUG_NEXT: &str = "sdf.debug_next";
pub(crate) const ACTION_DEBUG_PREV: &str = "sdf.debug_prev";
pub(crate) const ACTION_SPEED_UP: &str = "sdf.speed_up";
pub(crate) const ACTION_SPEED_DOWN: &str = "sdf.speed_down";

pub(crate) const SDF_ASSETS_DIR_PRIMARY: &str = "engine/examples/sdf_renderer/assets";
pub(crate) const SDF_ASSETS_DIR_FALLBACK: &str = "examples/sdf_renderer/assets";

pub(crate) const PARAMS_CONFIG_FILE: &str = "sdf_params.ron";
pub(crate) const INPUT_BINDINGS_CONFIG_FILE: &str = "input_bindings.ron";
pub(crate) const SDF_COMPUTE_SHADER: &str =
    include_str!("../../../../assets/shaders/sdf_compute_3d_example.wgsl");
pub(crate) const SDF_COMPOSE_SHADER: &str =
    include_str!("../../../../assets/shaders/world_compose_fullscreen.wgsl");
pub(crate) const SDF_MAX_AGENTS: usize = 512;
pub(crate) const SDF_MAX_MODELS: usize = 1;

#[derive(Debug, Clone, engine::prelude::Component)]
pub(crate) struct SdfRuntimeConfigState {
    pub(crate) controls: SdfControlsConfig,
    pub(crate) params_config_path: PathBuf,
    pub(crate) params_config_modified: Option<SystemTime>,
}

impl Default for SdfRuntimeConfigState {
    fn default() -> Self {
        let params_config_path = find_config_path(PARAMS_CONFIG_FILE);
        Self {
            controls: SdfControlsConfig::default(),
            params_config_modified: file_modified(&params_config_path),
            params_config_path,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SdfWorldAgent {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) radius: f32,
    pub(crate) health_ratio: f32,
    pub(crate) team: u32,
}

#[derive(Debug, Clone, engine::prelude::Component)]
pub(crate) struct SdfWorldState {
    pub(crate) world_bounds: [f32; 4],
    pub(crate) world_paused: bool,
    pub(crate) camera_yaw: f32,
    pub(crate) camera_pitch: f32,
    pub(crate) camera_distance: f32,
    pub(crate) camera_pitch_min: f32,
    pub(crate) camera_pitch_max: f32,
    pub(crate) camera_distance_min: f32,
    pub(crate) camera_distance_max: f32,
    pub(crate) camera_target: [f32; 3],
    pub(crate) camera_fov_y: f32,
    pub(crate) debug_view_mode: u32,
    pub(crate) display_fit_mode: u32,
    pub(crate) display_target_aspect: f32,
    pub(crate) display_render_scale: f32,
    pub(crate) display_bar_color: [f32; 4],
    pub(crate) elapsed_time_seconds: f32,
    pub(crate) agents: Vec<SdfWorldAgent>,
}

impl SdfWorldState {
    pub(crate) fn compute_params(&self) -> crate::rendering::SdfWorldParams {
        self.compute_params_with_surface((1, 1))
    }

    pub(crate) fn compute_params_with_surface(
        &self,
        surface: (u32, u32),
    ) -> crate::rendering::SdfWorldParams {
        let render_scale = self.display_render_scale.clamp(0.25, 4.0);
        let scaled_width = ((surface.0.max(1) as f32) * render_scale).round() as u32;
        let scaled_height = ((surface.1.max(1) as f32) * render_scale).round() as u32;
        let screen_size = [scaled_width.max(1) as f32, scaled_height.max(1) as f32];

        crate::rendering::SdfWorldParams {
            screen_size,
            _pad0: [0.0; 2],
            world_min: [self.world_bounds[0], self.world_bounds[1]],
            _pad1: [0.0; 2],
            world_max: [self.world_bounds[2], self.world_bounds[3]],
            _pad2: [0.0; 2],
            agent_count: self.agents.len().min(SDF_MAX_AGENTS) as u32,
            model_count: 0,
            paused: self.world_paused,
            _pad3: 0,
            camera_target_time: [
                self.camera_target[0],
                self.camera_target[1],
                self.camera_target[2],
                self.elapsed_time_seconds.max(0.0),
            ],
            camera_orbit: [
                self.camera_yaw,
                self.camera_pitch,
                self.camera_distance.max(0.1),
                self.camera_fov_y.clamp(0.1, std::f32::consts::PI - 0.1),
            ],
            debug_view_mode: self.debug_view_mode,
            display_fit_mode: self.display_fit_mode,
            display_target_aspect: self.display_target_aspect,
            _pad4: 0,
        }
    }

    pub(crate) fn compose_params(&self, surface: (u32, u32)) -> crate::rendering::SdfComposeParams {
        crate::rendering::SdfComposeParams {
            output_size: [surface.0.max(1) as f32, surface.1.max(1) as f32],
            target_aspect: self.display_target_aspect,
            fit_mode: self.display_fit_mode,
            bar_color: self.display_bar_color,
        }
    }

    pub(crate) fn agent_params(&self) -> Vec<crate::rendering::SdfWorldAgent> {
        self.agents
            .iter()
            .take(SDF_MAX_AGENTS)
            .map(|agent| crate::rendering::SdfWorldAgent {
                pos: [agent.x, agent.y],
                radius: agent.radius.max(0.2),
                health: agent.health_ratio.clamp(0.0, 1.0),
                team: agent.team,
                _pad0: [0; 3],
            })
            .collect()
    }
}

impl Default for SdfWorldState {
    fn default() -> Self {
        Self {
            world_bounds: [-18.0, -18.0, 18.0, 18.0],
            world_paused: false,
            camera_yaw: std::f32::consts::PI,
            camera_pitch: 0.58,
            camera_distance: 14.0,
            camera_pitch_min: -1.10,
            camera_pitch_max: 1.10,
            camera_distance_min: 2.0,
            camera_distance_max: 80.0,
            camera_target: [0.0, 1.8, 0.0],
            camera_fov_y: 55.0f32.to_radians(),
            debug_view_mode: 0,
            display_fit_mode: 0,
            display_target_aspect: 0.0,
            display_render_scale: 1.0,
            display_bar_color: [0.02, 0.02, 0.03, 1.0],
            elapsed_time_seconds: 0.0,
            agents: Vec::new(),
        }
    }
}
