use super::super::ids::PlanetId;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component, ecs::Resource)]
pub struct PlanetFrame {
    pub planet_id: PlanetId,
    pub origin_meters: [f64; 3],
    pub basis: [[f32; 3]; 3],
}

impl Default for PlanetFrame {
    fn default() -> Self {
        Self {
            planet_id: PlanetId(0),
            origin_meters: [0.0, 0.0, 0.0],
            basis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component, ecs::Resource)]
pub struct CameraRelativeFrame {
    pub planet_id: PlanetId,
    pub camera_world_position_meters: [f64; 3],
    pub camera_local_position_meters: [f32; 3],
    pub local_origin_offset_meters: [f32; 3],
}

impl Default for CameraRelativeFrame {
    fn default() -> Self {
        Self {
            planet_id: PlanetId(0),
            camera_world_position_meters: [0.0, 0.0, 0.0],
            camera_local_position_meters: [0.0, 0.0, 0.0],
            local_origin_offset_meters: [0.0, 0.0, 0.0],
        }
    }
}

pub type PlanetFrameResource = PlanetFrame;
pub type CameraRelativeFrameResource = CameraRelativeFrame;

pub fn build_camera_relative_frame(
    planet_frame: PlanetFrame,
    camera_world_position_meters: [f64; 3],
) -> CameraRelativeFrame {
    let local = [
        (camera_world_position_meters[0] - planet_frame.origin_meters[0]) as f32,
        (camera_world_position_meters[1] - planet_frame.origin_meters[1]) as f32,
        (camera_world_position_meters[2] - planet_frame.origin_meters[2]) as f32,
    ];
    CameraRelativeFrame {
        planet_id: planet_frame.planet_id,
        camera_world_position_meters,
        camera_local_position_meters: [0.0, 0.0, 0.0],
        local_origin_offset_meters: local,
    }
}
