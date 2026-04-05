//! File: domain/editor/editor_viewport/src/snap.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapSettings {
	pub translate_step: f32,
	pub rotate_degrees: f32,
	pub scale_step: f32,
	pub enabled: bool,
}

impl Default for SnapSettings {
	fn default() -> Self {
		Self {
			translate_step: 0.5,
			rotate_degrees: 15.0,
			scale_step: 0.1,
			enabled: false,
		}
	}
}