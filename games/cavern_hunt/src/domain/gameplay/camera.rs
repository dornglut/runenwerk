#[derive(Debug, Clone, PartialEq, ecs::Resource)]
pub struct CavernCameraState {
    pub target: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub fov_y_radians: f32,
}

impl Default for CavernCameraState {
    fn default() -> Self {
        Self {
            target: [0.0, 1.9, 0.0],
            yaw: std::f32::consts::PI,
            pitch: 1.14,
            distance: 34.0,
            pitch_min: 0.95,
            pitch_max: 1.34,
            distance_min: 18.0,
            distance_max: 48.0,
            fov_y_radians: 52.0_f32.to_radians(),
        }
    }
}
