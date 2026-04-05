//! File: domain/ui/ui_core/src/paint.rs
//! Purpose: Renderer-independent paint command contracts.

use ui_math::{UiPoint, UiRect};

#[derive(Debug, Clone, PartialEq)]
pub enum PaintCommand {
	FillRect {
		rect: UiRect,
		radius: f32,
		color: [f32; 4],
	},
	StrokeRect {
		rect: UiRect,
		radius: f32,
		width: f32,
		color: [f32; 4],
	},
	Text {
		position: UiPoint,
		content: String,
		style_id: u64,
		clip: Option<UiRect>,
	},
	PushClip(UiRect),
	PopClip,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PaintList {
	pub commands: Vec<PaintCommand>,
}

impl PaintList {
	pub fn push(&mut self, command: PaintCommand) {
		self.commands.push(command);
	}
}