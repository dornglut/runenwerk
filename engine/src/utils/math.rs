use glam::Vec3;

/// Frame-rate independent lerp smoothing.
/// `current` moves towards `target` using `speed` (units/sec), scaled by `dt`.
pub fn smooth_damp(current: f32, target: f32, speed: f32, dt: f32) -> f32 {
	let alpha = 1.0 - (-speed * dt).exp();
	current + (target - current) * alpha
}

pub fn smooth_damp_vec3(current: Vec3, target: Vec3, speed: f32, dt: f32) -> Vec3 {
	let alpha = 1.0 - (-speed * dt).exp();
	current + (target - current) * alpha
}
