// Owner: Game of Life SDF Example - Render State and Params
use engine::plugins::render::{GpuStorage, GpuUniform};

pub(crate) const DEFAULT_GRID_WIDTH: u32 = 160;
pub(crate) const DEFAULT_GRID_HEIGHT: u32 = 90;

#[derive(Debug, Clone, Copy, GpuStorage)]
pub(crate) struct GameOfLifeCell {
    alive: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct GameOfLifeComputeParams {
    tick: u32,
    seed: u32,
    grid_size: [u32; 2],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct GameOfLifeComposeParams {
    grid_size: [u32; 2],
    surface_size: [f32; 2],
    alive_mix: f32,
}

#[derive(Debug, Clone, ecs::Component)]
pub(crate) struct GameOfLifeRenderState {
    tick: u32,
    seed: u32,
    grid_size: [u32; 2],
    alive_mix: f32,
}

impl Default for GameOfLifeRenderState {
    fn default() -> Self {
        Self {
            tick: 0,
            seed: 0xC0FF_EE11,
            grid_size: [DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT],
            alive_mix: 1.0,
        }
    }
}

impl GameOfLifeRenderState {
    pub(crate) fn compute_params(&self) -> GameOfLifeComputeParams {
        GameOfLifeComputeParams {
            tick: self.tick,
            seed: self.seed,
            grid_size: self.grid_size,
        }
    }

    pub(crate) fn compose_params(&self, surface: (u32, u32)) -> GameOfLifeComposeParams {
        GameOfLifeComposeParams {
            grid_size: self.grid_size,
            surface_size: [surface.0 as f32, surface.1 as f32],
            alive_mix: self.alive_mix,
        }
    }
}
