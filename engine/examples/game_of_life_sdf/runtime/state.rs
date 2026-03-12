// Owner: Game of Life SDF Example - Runtime State

pub(crate) const ACTION_TOGGLE_PAUSE: &str = "gol.toggle_pause";
pub(crate) const ACTION_SINGLE_STEP: &str = "gol.single_step";
pub(crate) const ACTION_SPEED_UP: &str = "gol.speed_up";
pub(crate) const ACTION_SPEED_DOWN: &str = "gol.speed_down";

#[derive(Debug, Clone, engine::prelude::Component)]
pub(crate) struct GameOfLifeSdfState {
    pub(crate) grid_size: [u32; 2],
    pub(crate) steps_per_second: f32,
    pub(crate) accumulator_seconds: f32,
    pub(crate) step_simulation: bool,
    pub(crate) paused: bool,
    pub(crate) single_step_requested: bool,
    pub(crate) generation: u64,
    pub(crate) initial_alive_density: f32,
    pub(crate) cell_radius: f32,
    pub(crate) edge_softness: f32,
    pub(crate) grid_line_width: f32,
    pub(crate) glow_strength: f32,
    pub(crate) alive_color: [f32; 4],
    pub(crate) dead_color: [f32; 4],
    pub(crate) grid_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
    pub(crate) time_seconds: f32,
}

impl GameOfLifeSdfState {
    pub(crate) fn step_interval_seconds(&self) -> f32 {
        1.0 / self.steps_per_second.max(1.0)
    }
}

impl Default for GameOfLifeSdfState {
    fn default() -> Self {
        Self {
            grid_size: [160, 90],
            steps_per_second: 18.0,
            accumulator_seconds: 0.0,
            step_simulation: false,
            paused: false,
            single_step_requested: false,
            generation: 0,
            initial_alive_density: 0.24,
            cell_radius: 0.33,
            edge_softness: 0.06,
            grid_line_width: 0.032,
            glow_strength: 0.36,
            alive_color: [0.26, 0.95, 0.66, 1.0],
            dead_color: [0.10, 0.15, 0.13, 1.0],
            grid_color: [0.16, 0.24, 0.22, 0.38],
            background_color: [0.03, 0.045, 0.042, 1.0],
            time_seconds: 0.0,
        }
    }
}
