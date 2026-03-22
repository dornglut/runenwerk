use engine::plugins::render::GpuUniform;

pub(crate) const DEFAULT_ORBIT_SPEED: f32 = 0.38;

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct SdfComposeParams {
    pub time_data: [f32; 4],
    pub surface: [f32; 4],
    pub camera: [f32; 4],
    pub fog: [f32; 4],
    pub color_a: [f32; 4],
    pub color_b: [f32; 4],
}

#[derive(Debug, Clone, ecs::Resource)]
pub(crate) struct Sdf3dRenderState {
    time_seconds: f32,
    orbit_yaw_radians: f32,
    orbit_pitch_radians: f32,
    orbit_distance: f32,
    orbit_speed: f32,
    fov_radians: f32,
}

impl Default for Sdf3dRenderState {
    fn default() -> Self {
        Self {
            time_seconds: 0.0,
            orbit_yaw_radians: 0.15,
            orbit_pitch_radians: 0.32,
            orbit_distance: 5.2,
            orbit_speed: DEFAULT_ORBIT_SPEED,
            fov_radians: 57.0_f32.to_radians(),
        }
    }
}

impl Sdf3dRenderState {
    pub(crate) fn advance_by_frame_delta(&mut self, delta_seconds: f32) {
        let safe_delta = delta_seconds.clamp(0.0, 1.0 / 15.0).max(1.0 / 240.0);
        self.time_seconds += safe_delta;
        self.orbit_yaw_radians = (self.orbit_yaw_radians + safe_delta * self.orbit_speed)
            .rem_euclid(std::f32::consts::TAU);
    }

    pub(crate) fn compose_params(&self, surface: (u32, u32)) -> SdfComposeParams {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let pulse = 0.5 + 0.5 * (self.time_seconds * 0.82).sin();

        SdfComposeParams {
            time_data: [
                self.time_seconds,
                pulse,
                (self.time_seconds * 0.35).sin(),
                (self.time_seconds * 0.27).cos(),
            ],
            surface: [width, height, 1.0 / width, 1.0 / height],
            camera: [
                self.orbit_yaw_radians,
                self.orbit_pitch_radians,
                self.orbit_distance,
                self.fov_radians,
            ],
            fog: [0.045, 2.4, 22.0, 0.0],
            color_a: [0.10, 0.52, 0.94, 1.0],
            color_b: [0.92, 0.34, 0.48, 1.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sdf3dRenderState;

    #[test]
    fn advance_updates_time_and_orbit() {
        let mut state = Sdf3dRenderState::default();
        let before = state.compose_params((1280, 720));
        state.advance_by_frame_delta(1.0 / 60.0);
        let after = state.compose_params((1280, 720));

        assert!(after.time_data[0] > before.time_data[0]);
        assert!(after.camera[0] > before.camera[0]);
    }

    #[test]
    fn compose_params_include_surface_metrics() {
        let state = Sdf3dRenderState::default();
        let params = state.compose_params((1920, 1080));
        assert_eq!(params.surface[0], 1920.0);
        assert_eq!(params.surface[1], 1080.0);
        assert!((params.surface[2] - (1.0 / 1920.0)).abs() < 1.0e-6);
        assert!((params.surface[3] - (1.0 / 1080.0)).abs() < 1.0e-6);
    }
}
