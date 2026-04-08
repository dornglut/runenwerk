use ui_render_data::{UiFrame, UiPrimitive};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorUiRenderSubmission {
	pub frame: UiFrame,
}

impl EditorUiRenderSubmission {
	pub fn from_ui_frame(
		frame: &UiFrame,
	) -> Self {
		Self {
			frame: frame.clone(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.frame.is_empty()
	}

	pub fn primitive_count(&self) -> usize {
		self.frame
			.surfaces
			.iter()
			.map(|surface| {
				surface
					.layers
					.iter()
					.map(|layer| layer.primitives.len())
					.sum::<usize>()
			})
			.sum()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditorUiPrimitiveBreakdown {
	pub rects: usize,
	pub borders: usize,
	pub glyph_runs: usize,
	pub images: usize,
	pub clips: usize,
}

impl EditorUiPrimitiveBreakdown {
	pub fn from_submission(
		submission: &EditorUiRenderSubmission,
	) -> Self {
		let mut breakdown = Self::default();

		for surface in &submission.frame.surfaces {
			for layer in &surface.layers {
				for primitive in &layer.primitives {
					match primitive {
						UiPrimitive::Rect(_) => breakdown.rects += 1,
						UiPrimitive::Border(_) => breakdown.borders += 1,
						UiPrimitive::GlyphRun(_) => breakdown.glyph_runs += 1,
						UiPrimitive::Image(_) => breakdown.images += 1,
						UiPrimitive::Clip(_) => breakdown.clips += 1,
					}
				}
			}
		}

		breakdown
	}
}
