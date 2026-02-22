use glam::{vec3, Quat, Vec3};
use tracing::info;
use winit::keyboard::KeyCode;
use crate::engine::input::InputState;
use crate::engine::world::World;
use crate::utils::{smooth_damp, smooth_damp_vec3};

pub struct Camera {
	pub position: Vec3,
	pub velocity: Vec3,
	pub forward: Vec3,
	pub up: Vec3,
	pub right: Vec3,
	pub fov: f32,
	pub aspect_ratio: f32,
	pub near: f32,
	pub far: f32,
	pub yaw: f32,
	pub pitch: f32,
	pub target_yaw: f32,
	pub target_pitch: f32,
}

impl Camera {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn update(&mut self, input: &InputState, dt: f32) {
		let camera = self;

		// ---- Mouse look ----
		let sensitivity = 0.5;
		camera.target_yaw = input.mouse_delta.0 * sensitivity;   // rotate around Y
		camera.target_pitch -= input.mouse_delta.1 * sensitivity; // rotate around local X
		camera.target_pitch = camera.target_pitch.clamp(-89.9, 89.9);   // prevent gimbal lock

		let smoothing = 20.0;
		camera.yaw = smooth_damp(camera.yaw, camera.target_yaw, smoothing, dt);
		camera.pitch = smooth_damp(camera.pitch, camera.target_pitch, smoothing, dt);

		let rotation = Quat::from_euler(glam::EulerRot::YXZ, camera.yaw.to_radians(), camera.pitch.to_radians(), 0.0);
		camera.forward = rotation * Vec3::Z * -1.0; // -Z forward
		camera.up = rotation * Vec3::Y;
		camera.right = camera.forward.cross(camera.up).normalize();

		// ---- Keyboard movement ----
		let mut dir = Vec3::ZERO;
		if input.key(KeyCode::KeyW) { dir += camera.forward; }
		if input.key(KeyCode::KeyS) { dir -= camera.forward; }
		if input.key(KeyCode::KeyA) { dir -= camera.right; }
		if input.key(KeyCode::KeyD) { dir += camera.right; }
		if input.key(KeyCode::Space) { dir += camera.up; }
		if input.key(KeyCode::ShiftLeft) { dir -= camera.up; }

		let speed = 5.0;
		let target_velocity = if dir != Vec3::ZERO { dir.normalize() * speed } else { Vec3::ZERO };

		camera.velocity = smooth_damp_vec3(camera.velocity, target_velocity, 20.0, dt);
		camera.position += camera.velocity * dt;
	}
}

impl Default for Camera {
	fn default() -> Self {
		let forward = vec3(0.0, 0.0, -1.0);
		let up = vec3(0.0, 1.0, 0.0);
		let right = forward.cross(up).normalize();

		Self {
			position: vec3(0.0, 0.0, 3.0),
			velocity: Vec3::ZERO,
			forward,
			up,
			right,
			fov: 60.0,
			aspect_ratio: 16.0 / 9.0,
			near: 0.1,
			far: 1000.0,
			yaw: 0.0,
			pitch: 0.0,
			target_yaw: 0.0,
			target_pitch: 0.0,
		}
	}
}