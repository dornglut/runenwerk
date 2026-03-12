// Owner: Game of Life SDF Example - Runtime State

#[derive(Debug, Clone, engine::prelude::Component)]
pub(crate) struct GameOfLifeSdfState {
    pub(crate) grid_size: [u32; 2],
    pub(crate) step_simulation: bool,
    pub(crate) cell_radius: f32,
    pub(crate) edge_softness: f32,
    pub(crate) grid_line_width: f32,
    pub(crate) glow_strength: f32,
    pub(crate) alive_color: [f32; 4],
    pub(crate) dead_color: [f32; 4],
    pub(crate) grid_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
}

impl GameOfLifeSdfState {
    pub(crate) fn compute_params(&self) -> crate::rendering::GameOfLifeComputeParams {
        crate::rendering::GameOfLifeComputeParams {
            grid_size: self.grid_size,
            step: self.step_simulation,
            _pad0: 0,
        }
    }

    pub(crate) fn compose_params(
        &self,
        surface: (u32, u32),
    ) -> crate::rendering::GameOfLifeComposeParams {
        crate::rendering::GameOfLifeComposeParams {
            output_size: [surface.0 as f32, surface.1 as f32],
            grid_size: [self.grid_size[0] as f32, self.grid_size[1] as f32],
            cell_radius: self.cell_radius.clamp(0.05, 0.49),
            edge_softness: self.edge_softness.clamp(0.001, 0.25),
            grid_line_width: self.grid_line_width.clamp(0.0, 0.2),
            glow_strength: self.glow_strength.clamp(0.0, 2.0),
            alive_color: self.alive_color,
            dead_color: self.dead_color,
            grid_color: self.grid_color,
            background_color: self.background_color,
        }
    }
}

impl Default for GameOfLifeSdfState {
    fn default() -> Self {
        Self {
            grid_size: [160, 90],
            step_simulation: true,
            cell_radius: 0.33,
            edge_softness: 0.06,
            grid_line_width: 0.032,
            glow_strength: 0.36,
            alive_color: [0.26, 0.95, 0.66, 1.0],
            dead_color: [0.10, 0.15, 0.13, 1.0],
            grid_color: [0.16, 0.24, 0.22, 0.38],
            background_color: [0.03, 0.045, 0.042, 1.0],
        }
    }
}
