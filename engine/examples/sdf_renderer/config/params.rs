// Owner: SDF Renderer Example - Core Params Config
use crate::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SdfParamsConfig {
    pub(crate) world_scene_label: String,
    pub(crate) overlay_scene_label: String,
    pub(crate) world_bounds: [f32; 4],
    pub(crate) camera: SdfCameraConfig,
    pub(crate) controls: SdfControlsConfig,
    pub(crate) debug_view_mode: u32,
    pub(crate) world_paused: bool,
    pub(crate) render_mesh_overlay: bool,
    pub(crate) display: SdfDisplayConfig,
}

impl Default for SdfParamsConfig {
    fn default() -> Self {
        Self {
            world_scene_label: "gameplay_stub".to_string(),
            overlay_scene_label: "console_ui".to_string(),
            world_bounds: [-18.0, -18.0, 18.0, 18.0],
            camera: SdfCameraConfig::default(),
            controls: SdfControlsConfig::default(),
            debug_view_mode: 0,
            world_paused: false,
            render_mesh_overlay: false,
            display: SdfDisplayConfig::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SdfDisplayFitMode {
    Stretch,
    Contain,
    Cover,
    FixedHeight,
    FixedWidth,
}

impl SdfDisplayFitMode {
    pub(crate) const fn as_shader_mode(self) -> u32 {
        match self {
            Self::Stretch => 0,
            Self::Contain => 1,
            Self::Cover => 2,
            Self::FixedHeight => 3,
            Self::FixedWidth => 4,
        }
    }
}

impl Default for SdfDisplayFitMode {
    fn default() -> Self {
        Self::Stretch
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SdfDisplayConfig {
    pub(crate) fit_mode: SdfDisplayFitMode,
    pub(crate) target_aspect: f32,
    pub(crate) render_scale: f32,
    pub(crate) bar_color: [f32; 4],
}

impl Default for SdfDisplayConfig {
    fn default() -> Self {
        Self {
            fit_mode: SdfDisplayFitMode::Contain,
            target_aspect: 16.0 / 9.0,
            render_scale: 1.0,
            bar_color: [0.02, 0.02, 0.03, 1.0],
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SdfCameraConfig {
    pub(crate) target: [f32; 3],
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) distance: f32,
    pub(crate) pitch_min: f32,
    pub(crate) pitch_max: f32,
    pub(crate) distance_min: f32,
    pub(crate) distance_max: f32,
    pub(crate) fov_y_radians: f32,
}

impl Default for SdfCameraConfig {
    fn default() -> Self {
        Self {
            target: [0.0, 0.8, 0.0],
            yaw: 0.4,
            pitch: 0.25,
            distance: 9.5,
            pitch_min: -1.2,
            pitch_max: 1.2,
            distance_min: 2.0,
            distance_max: 30.0,
            fov_y_radians: 58.0f32.to_radians(),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SdfControlsConfig {
    pub(crate) base_move_speed: f32,
    pub(crate) speed_up_multiplier: f32,
    pub(crate) speed_down_multiplier: f32,
    pub(crate) mouse_rotate_sensitivity: f32,
    pub(crate) scroll_zoom_sensitivity: f32,
    pub(crate) camera_target_y_min: f32,
    pub(crate) camera_target_y_max: f32,
}

impl Default for SdfControlsConfig {
    fn default() -> Self {
        Self {
            base_move_speed: 7.5,
            speed_up_multiplier: 2.0,
            speed_down_multiplier: 0.35,
            mouse_rotate_sensitivity: 0.0045,
            scroll_zoom_sensitivity: 0.55,
            camera_target_y_min: -4.0,
            camera_target_y_max: 8.0,
        }
    }
}
