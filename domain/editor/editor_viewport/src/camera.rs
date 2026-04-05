//! File: domain/editor/editor_viewport/src/camera.rs

use ui_math::{UiPoint, UiVector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportProjection {
	Perspective,
	Orthographic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorCameraState {
	pub projection: ViewportProjection,
	pub orbit_target: [f32; 3],
	pub distance: f32,
	pub yaw_radians: f32,
	pub pitch_radians: f32,
	pub viewport_origin: UiPoint,
	pub viewport_size: ui_math::UiSize,
	pub pan_delta: UiVector,
}

impl Default for EditorCameraState {
	fn default() -> Self {
		Self {
			projection: ViewportProjection::Perspective,
			orbit_target: [0.0, 0.0, 0.0],
			distance: 5.0,
			yaw_radians: 0.0,
			pitch_radians: 0.0,
			viewport_origin: UiPoint::ZERO,
			viewport_size: ui_math::UiSize::ZERO,
			pan_delta: UiVector::ZERO,
		}
	}
}