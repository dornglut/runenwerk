use godot::builtin::Vector3;
use spatial::WorldLocalPosition;

/// File: adapters/godot_chunking_addon/src/bridge.rs
/// Function: vector3_to_world_local_position
pub fn vector3_to_world_local_position(value: Vector3) -> WorldLocalPosition {
	WorldLocalPosition::new([value.x, value.y, value.z])
}

/// File: adapters/godot_chunking_addon/src/bridge.rs
/// Function: vector3_to_meters
pub fn vector3_to_meters(value: Vector3) -> [f32; 3] {
	[value.x, value.y, value.z]
}