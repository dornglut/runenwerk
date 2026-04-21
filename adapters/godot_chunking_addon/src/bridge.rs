use godot::builtin::Vector3;

/// Function: vector3_to_meters
pub fn vector3_to_meters(value: Vector3) -> [f32; 3] {
    [value.x, value.y, value.z]
}
