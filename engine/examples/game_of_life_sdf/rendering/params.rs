use engine::plugins::render::GpuUniform;

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct GameOfLifeComputeParams {
    pub(crate) grid_size: [u32; 2],
    pub(crate) step: bool,
    pub(crate) _pad0: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct GameOfLifeComposeParams {
    pub(crate) output_size: [f32; 2],
    pub(crate) grid_size: [f32; 2],
    pub(crate) cell_radius: f32,
    pub(crate) edge_softness: f32,
    pub(crate) grid_line_width: f32,
    pub(crate) glow_strength: f32,
    pub(crate) alive_color: [f32; 4],
    pub(crate) dead_color: [f32; 4],
    pub(crate) grid_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
}