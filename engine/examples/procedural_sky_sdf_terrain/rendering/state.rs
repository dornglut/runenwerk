use engine::plugins::render::GpuUniform;

pub(crate) const DEFAULT_ORBIT_SPEED: f32 = 0.18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerrainViewMode {
    Lit,
    Height,
    Normals,
    Steps,
}

impl TerrainViewMode {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Lit => Self::Height,
            Self::Height => Self::Normals,
            Self::Normals => Self::Steps,
            Self::Steps => Self::Lit,
        }
    }

    pub(crate) fn as_u32(self) -> u32 {
        match self {
            Self::Lit => 0,
            Self::Height => 1,
            Self::Normals => 2,
            Self::Steps => 3,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Lit => "lit",
            Self::Height => "height",
            Self::Normals => "normals",
            Self::Steps => "steps",
        }
    }
}

#[derive(Debug, Clone, Copy, GpuUniform)]
pub(crate) struct TerrainComposeParams {
    pub time_data: [f32; 4],
    pub surface: [f32; 4],
    pub camera: [f32; 4],
    pub terrain: [f32; 4],
    pub sky: [f32; 4],
    pub view_data: [u32; 4],
    pub colors_a: [f32; 4],
    pub colors_b: [f32; 4],
    pub colors_c: [f32; 4],
    pub colors_d: [f32; 4],
}

#[derive(Debug, Clone, ecs::Resource)]
pub(crate) struct ProceduralSkyTerrainState {
    time_seconds: f32,
    orbit_yaw_radians: f32,
    orbit_pitch_radians: f32,
    orbit_distance: f32,
    orbit_speed: f32,
    fov_radians: f32,
    view_mode: TerrainViewMode,
}

impl Default for ProceduralSkyTerrainState {
    fn default() -> Self {
        Self {
            time_seconds: 0.0,
            orbit_yaw_radians: 0.26,
            orbit_pitch_radians: 0.24,
            orbit_distance: 16.0,
            orbit_speed: DEFAULT_ORBIT_SPEED,
            fov_radians: 52.0_f32.to_radians(),
            view_mode: TerrainViewMode::Lit,
        }
    }
}

impl ProceduralSkyTerrainState {
    pub(crate) fn cycle_view_mode(&mut self) {
        self.view_mode = self.view_mode.next();
    }

    pub(crate) fn view_mode_label(&self) -> &'static str {
        self.view_mode.label()
    }

    pub(crate) fn advance_by_frame_delta(&mut self, delta_seconds: f32) {
        let safe_delta = delta_seconds.clamp(0.0, 1.0 / 15.0).max(1.0 / 240.0);
        self.time_seconds += safe_delta;
        self.orbit_yaw_radians = (self.orbit_yaw_radians + safe_delta * self.orbit_speed)
            .rem_euclid(std::f32::consts::TAU);
    }

    pub(crate) fn compose_params(&self, surface: (u32, u32)) -> TerrainComposeParams {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let pulse = 0.5 + 0.5 * (self.time_seconds * 0.21).sin();

        TerrainComposeParams {
            time_data: [
                self.time_seconds,
                pulse,
                (self.time_seconds * 0.13).sin(),
                (self.time_seconds * 0.07).cos(),
            ],
            surface: [width, height, 1.0 / width, 1.0 / height],
            camera: [
                self.orbit_yaw_radians,
                self.orbit_pitch_radians,
                self.orbit_distance,
                self.fov_radians,
            ],
            terrain: [
                -1.8,  // base height
                6.4,   // height scale
                0.085, // base frequency
                2.6,   // detail gain
            ],
            sky: [
                0.75, // turbidity
                0.52, // sun elevation weight
                1.35, // cloud scale
                0.18, // cloud drift speed
            ],
            view_data: [self.view_mode.as_u32(), 0, 0, 0],
            colors_a: [0.06, 0.17, 0.38, 1.0], // sky top
            colors_b: [0.95, 0.60, 0.34, 1.0], // sky horizon
            colors_c: [0.25, 0.21, 0.16, 1.0], // terrain lowland
            colors_d: [0.24, 0.50, 0.24, 1.0], // terrain highland
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProceduralSkyTerrainState;

    #[test]
    fn advance_updates_time_and_orbit() {
        let mut state = ProceduralSkyTerrainState::default();
        let before = state.compose_params((1280, 720));
        state.advance_by_frame_delta(1.0 / 60.0);
        let after = state.compose_params((1280, 720));

        assert!(after.time_data[0] > before.time_data[0]);
        assert!(after.camera[0] > before.camera[0]);
    }

    #[test]
    fn compose_params_include_surface_metrics() {
        let state = ProceduralSkyTerrainState::default();
        let params = state.compose_params((1920, 1080));
        assert_eq!(params.surface[0], 1920.0);
        assert_eq!(params.surface[1], 1080.0);
        assert!((params.surface[2] - (1.0 / 1920.0)).abs() < 1.0e-6);
        assert!((params.surface[3] - (1.0 / 1080.0)).abs() < 1.0e-6);
    }

    #[test]
    fn cycle_view_mode_updates_uniform_mode() {
        let mut state = ProceduralSkyTerrainState::default();
        assert_eq!(state.compose_params((1, 1)).view_data[0], 0);
        state.cycle_view_mode();
        assert_eq!(state.compose_params((1, 1)).view_data[0], 1);
        state.cycle_view_mode();
        assert_eq!(state.compose_params((1, 1)).view_data[0], 2);
        state.cycle_view_mode();
        assert_eq!(state.compose_params((1, 1)).view_data[0], 3);
        state.cycle_view_mode();
        assert_eq!(state.compose_params((1, 1)).view_data[0], 0);
    }
}
