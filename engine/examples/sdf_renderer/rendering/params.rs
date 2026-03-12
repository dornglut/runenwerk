use engine::plugins::render::{GpuStorage, GpuUniform};

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct SdfWorldParams {
    pub(crate) screen_size: [f32; 2],
    pub(crate) _pad0: [f32; 2],
    pub(crate) world_min: [f32; 2],
    pub(crate) _pad1: [f32; 2],
    pub(crate) world_max: [f32; 2],
    pub(crate) _pad2: [f32; 2],
    pub(crate) agent_count: u32,
    pub(crate) model_count: u32,
    pub(crate) paused: bool,
    pub(crate) _pad3: u32,
    pub(crate) camera_target_time: [f32; 4],
    pub(crate) camera_orbit: [f32; 4],
    pub(crate) debug_view_mode: u32,
    pub(crate) display_fit_mode: u32,
    pub(crate) display_target_aspect: f32,
    pub(crate) _pad4: u32,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
pub(crate) struct SdfWorldAgent {
    pub(crate) pos: [f32; 2],
    pub(crate) radius: f32,
    pub(crate) health: f32,
    pub(crate) team: u32,
    pub(crate) _pad0: [u32; 3],
}

#[derive(Debug, Clone, Copy, GpuStorage)]
pub(crate) struct SdfWorldModel {
    pub(crate) pos: [f32; 2],
    pub(crate) radius: f32,
    pub(crate) _pad0: f32,
    pub(crate) color: [f32; 4],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct SdfComposeParams {
    pub(crate) output_size: [f32; 2],
    pub(crate) target_aspect: f32,
    pub(crate) fit_mode: u32,
    pub(crate) bar_color: [f32; 4],
}
