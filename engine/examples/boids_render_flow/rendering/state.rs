use engine::plugins::render::{GpuStorage, GpuUniform};

pub(crate) const DEFAULT_BOID_COUNT: u32 = 384;
pub(crate) const DEFAULT_SIMULATION_FPS: f32 = 60.0;
pub(crate) const DEFAULT_GRID_CELLS_X: u32 = 10;
pub(crate) const DEFAULT_GRID_CELLS_Y: u32 = 10;

const MODE_SEED: u32 = 0;
const MODE_CLEAR_COUNTS: u32 = 1;
const MODE_COUNT_CELLS: u32 = 2;
const MODE_SCAN_COUNTS: u32 = 3;
const MODE_RESET_CURSORS: u32 = 4;
const MODE_SCATTER_INDICES: u32 = 5;
const MODE_SIMULATE_GRID: u32 = 6;
const MODE_PUBLISH: u32 = 7;

#[derive(Debug, Clone, Copy, GpuStorage)]
pub(crate) struct BoidAgent {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub visual_heading: [f32; 2],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct BoidsComputeParams {
    pub meta: [u32; 4],
    pub grid: [u32; 4],
    pub sim_a: [f32; 4],
    pub sim_b: [f32; 4],
    pub sim_c: [f32; 4],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct BoidsDrawParams {
    pub surface: [f32; 4],
    pub sprite: [f32; 4],
}

#[derive(Debug, Clone, ecs::Resource)]
pub(crate) struct BoidsRenderState {
    tick: u32,
    boid_count: u32,
    initialized: bool,
    simulation_fps: f32,
    fixed_delta_seconds: f32,
    last_frame_delta_seconds: f32,
    grid_cells: [u32; 2],
}

impl Default for BoidsRenderState {
    fn default() -> Self {
        let fixed_delta_seconds = 1.0 / DEFAULT_SIMULATION_FPS;
        Self {
            tick: 0,
            boid_count: DEFAULT_BOID_COUNT,
            initialized: false,
            simulation_fps: DEFAULT_SIMULATION_FPS,
            fixed_delta_seconds,
            last_frame_delta_seconds: fixed_delta_seconds,
            grid_cells: [DEFAULT_GRID_CELLS_X, DEFAULT_GRID_CELLS_Y],
        }
    }
}

impl BoidsRenderState {
    pub(crate) fn advance_by_frame_delta(&mut self, delta_seconds: f32) {
        self.last_frame_delta_seconds = delta_seconds.clamp(0.0, 1.0 / 15.0).max(1.0 / 240.0);
        if !self.initialized {
            self.initialized = true;
            return;
        }

        self.tick = self.tick.saturating_add(1);
    }

    pub(crate) fn compute_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_SIMULATE_GRID)
    }

    pub(crate) fn seed_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_SEED)
    }

    pub(crate) fn clear_counts_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_CLEAR_COUNTS)
    }

    pub(crate) fn count_cells_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_COUNT_CELLS)
    }

    pub(crate) fn scan_counts_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_SCAN_COUNTS)
    }

    pub(crate) fn reset_cursors_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_RESET_CURSORS)
    }

    pub(crate) fn scatter_indices_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_SCATTER_INDICES)
    }

    pub(crate) fn publish_params(&self) -> BoidsComputeParams {
        self.params_for_mode(MODE_PUBLISH)
    }

    fn params_for_mode(&self, mode: u32) -> BoidsComputeParams {
        let cell_count = self.grid_cell_count();
        BoidsComputeParams {
            meta: [self.tick, mode, self.boid_count, cell_count],
            grid: [self.grid_cells[0], self.grid_cells[1], cell_count, 0],
            sim_a: [
                self.fixed_delta_seconds,
                0.46, // max_speed
                1.10, // max_force
                0.10, // neighbor_radius
            ],
            sim_b: [
                0.035, // separation_radius
                1.05,  // alignment_weight
                0.72,  // cohesion_weight
                1.35,  // separation_weight
            ],
            sim_c: [
                0.16, // center_weight
                0.12, // jitter_strength
                self.simulation_fps,
                0.0,
            ],
        }
    }

    pub(crate) fn draw_params(&self, surface_size: (u32, u32)) -> BoidsDrawParams {
        let surface_width = surface_size.0.max(1) as f32;
        let surface_height = surface_size.1.max(1) as f32;
        BoidsDrawParams {
            surface: [
                surface_width,
                surface_height,
                1.0 / surface_width,
                1.0 / surface_height,
            ],
            sprite: [
                10.5, // body radius in screen pixels
                0.72, // half-width multiplier
                1.35, // half-height multiplier
                0.0,
            ],
        }
    }

    pub(crate) fn dispatch_workgroups(&self) -> [u32; 3] {
        [self.boid_count.div_ceil(64), 1, 1]
    }

    pub(crate) fn dispatch_grid_workgroups(&self) -> [u32; 3] {
        [self.grid_cell_count().div_ceil(64), 1, 1]
    }

    pub(crate) fn dispatch_scan_workgroups(&self) -> [u32; 3] {
        [1, 1, 1]
    }

    pub(crate) fn fixed_delta_seconds(&self) -> f32 {
        self.fixed_delta_seconds
    }

    #[cfg(test)]
    pub(crate) fn last_frame_delta_seconds(&self) -> f32 {
        self.last_frame_delta_seconds
    }

    pub(crate) fn submitted_step_count(&self) -> u32 {
        self.tick
    }

    pub(crate) fn grid_cell_count(&self) -> u32 {
        self.grid_cells[0]
            .checked_mul(self.grid_cells[1])
            .expect("boids grid dimensions must be validated")
    }
}

#[cfg(test)]
mod tests {
    use super::BoidsRenderState;

    #[test]
    fn first_update_keeps_seed_tick() {
        let mut state = BoidsRenderState::default();
        state.advance_by_frame_delta(1.0 / 120.0);
        let params = state.compute_params();
        assert_eq!(params.meta[0], 0);
        assert_eq!(params.meta[1], super::MODE_SIMULATE_GRID);
        assert_eq!(params.meta[2], super::DEFAULT_BOID_COUNT);
    }

    #[test]
    fn second_update_steps_and_publish_params_select_render_buffer_copy() {
        let mut state = BoidsRenderState::default();
        state.advance_by_frame_delta(1.0 / 120.0);
        state.advance_by_frame_delta(1.0 / 120.0);
        let compute = state.compute_params();
        assert_eq!(compute.meta[0], 1);
        assert_eq!(compute.meta[1], super::MODE_SIMULATE_GRID);
        assert_eq!(compute.meta[2], super::DEFAULT_BOID_COUNT);
        assert_eq!(
            compute.meta[3],
            super::DEFAULT_GRID_CELLS_X * super::DEFAULT_GRID_CELLS_Y
        );
        assert_eq!(compute.sim_a[0], state.fixed_delta_seconds());
        let publish = state.publish_params();
        assert_eq!(publish.meta[1], super::MODE_PUBLISH);
    }

    #[test]
    fn frame_delta_is_evidence_only_for_fixed_step_simulation() {
        let mut state = BoidsRenderState::default();
        state.advance_by_frame_delta(1.0 / 30.0);

        assert_eq!(
            state.compute_params().sim_a[0],
            1.0 / super::DEFAULT_SIMULATION_FPS
        );
        assert!(state.last_frame_delta_seconds() > state.fixed_delta_seconds());
    }

    #[test]
    fn draw_params_are_surface_aware() {
        let state = BoidsRenderState::default();
        let params = state.draw_params((1600, 900));

        assert_eq!(params.surface[0], 1600.0);
        assert_eq!(params.surface[1], 900.0);
        assert!(params.sprite[0] > 0.0);
    }

    #[test]
    fn dispatch_workgroups_cover_full_boid_count() {
        let state = BoidsRenderState::default();
        let dispatch = state.dispatch_workgroups();
        assert!(dispatch[0] > 0);
        assert_eq!(dispatch[1], 1);
        assert_eq!(dispatch[2], 1);
    }

    #[test]
    fn grid_dispatch_covers_all_cells() {
        let state = BoidsRenderState::default();
        assert_eq!(
            state.dispatch_grid_workgroups()[0],
            state.grid_cell_count().div_ceil(64)
        );
        assert_eq!(state.dispatch_scan_workgroups(), [1, 1, 1]);
    }
}
