use engine::plugins::render::{GpuStorage, GpuUniform};

pub(crate) const DEFAULT_BOID_COUNT: u32 = 384;
pub(crate) const DEFAULT_SIMULATION_FPS: f32 = 60.0;

#[derive(Debug, Clone, Copy, GpuStorage)]
pub(crate) struct BoidAgent {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct BoidsComputeParams {
    pub meta: [u32; 4],
    pub sim_a: [f32; 4],
    pub sim_b: [f32; 4],
    pub sim_c: [f32; 4],
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct BoidsComposeParams {
    pub meta: [u32; 4],
    pub surface: [f32; 4],
    pub draw: [f32; 4],
    pub background: [f32; 4],
    pub boid_color: [f32; 4],
}

#[derive(Debug, Clone, ecs::Resource)]
pub(crate) struct BoidsRenderState {
    tick: u32,
    boid_count: u32,
    current_is_a: bool,
    initialized: bool,
    simulation_fps: f32,
    last_delta_seconds: f32,
}

impl Default for BoidsRenderState {
    fn default() -> Self {
        Self {
            tick: 0,
            boid_count: DEFAULT_BOID_COUNT,
            current_is_a: true,
            initialized: false,
            simulation_fps: DEFAULT_SIMULATION_FPS,
            last_delta_seconds: 1.0 / DEFAULT_SIMULATION_FPS,
        }
    }
}

impl BoidsRenderState {
    pub(crate) fn advance_by_frame_delta(&mut self, delta_seconds: f32) {
        self.last_delta_seconds = delta_seconds.clamp(0.0, 1.0 / 15.0).max(1.0 / 240.0);
        if !self.initialized {
            self.initialized = true;
            return;
        }

        self.tick = self.tick.saturating_add(1);
        self.current_is_a = !self.current_is_a;
    }

    pub(crate) fn compute_params(&self) -> BoidsComputeParams {
        let read_from_a = if self.tick == 0 {
            1
        } else {
            u32::from(!self.current_is_a)
        };
        let step = u32::from(self.tick > 0);

        BoidsComputeParams {
            meta: [self.tick, step, read_from_a, self.boid_count],
            sim_a: [
                self.last_delta_seconds,
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

    pub(crate) fn compose_params(&self, surface: (u32, u32)) -> BoidsComposeParams {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        BoidsComposeParams {
            meta: [self.boid_count, u32::from(self.current_is_a), 0, 0],
            surface: [width, height, 1.0 / width, 1.0 / height],
            draw: [
                0.018, // body radius in normalized screen space
                0.040, // glow radius
                0.90,  // tail strength
                0.0,
            ],
            background: [0.020, 0.028, 0.040, 1.0],
            boid_color: [0.30, 0.95, 0.82, 1.0],
        }
    }

    pub(crate) fn dispatch_workgroups(&self) -> [u32; 3] {
        [self.boid_count.div_ceil(64), 1, 1]
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
        assert_eq!(params.meta[1], 0);
        assert_eq!(params.meta[2], 1);
    }

    #[test]
    fn second_update_steps_and_flips_current_buffer() {
        let mut state = BoidsRenderState::default();
        state.advance_by_frame_delta(1.0 / 120.0);
        state.advance_by_frame_delta(1.0 / 120.0);
        let compute = state.compute_params();
        let compose = state.compose_params((1920, 1080));
        assert_eq!(compute.meta[0], 1);
        assert_eq!(compute.meta[1], 1);
        assert_eq!(compute.meta[2], 1);
        assert_eq!(compose.meta[1], 0);
    }

    #[test]
    fn dispatch_workgroups_cover_full_boid_count() {
        let state = BoidsRenderState::default();
        let dispatch = state.dispatch_workgroups();
        assert!(dispatch[0] > 0);
        assert_eq!(dispatch[1], 1);
        assert_eq!(dispatch[2], 1);
    }
}
