//! File: domain/ui/ui_core/src/invalidation.rs
//! Purpose: Fine-grained invalidation flags.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Invalidation {
	pub layout: bool,
	pub paint: bool,
	pub text: bool,
	pub children: bool,
}

impl Invalidation {
	pub const fn clean() -> Self {
		Self {
			layout: false,
			paint: false,
			text: false,
			children: false,
		}
	}

	pub const fn repaint() -> Self {
		Self {
			layout: false,
			paint: true,
			text: false,
			children: false,
		}
	}

	pub const fn relayout() -> Self {
		Self {
			layout: true,
			paint: true,
			text: false,
			children: true,
		}
	}
}