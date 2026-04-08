//! File: domain/ui/ui_render_data/src/primitives/glyph_run.rs
//! Purpose: Text glyph run primitive referencing layout output.

use ui_math::UiRect;
use ui_text::GlyphRun;

use crate::{UiDrawKey, UiPaint, UiSortKey};

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRunPrimitive {
	pub glyph_run: GlyphRun,
	pub baseline_origin_clip: Option<UiRect>,
	pub tint: UiPaint,
	pub draw_key: UiDrawKey,
	pub sort_key: UiSortKey,
}

impl GlyphRunPrimitive {
	pub fn new(
		glyph_run: GlyphRun,
		baseline_origin_clip: Option<UiRect>,
		tint: UiPaint,
		draw_key: UiDrawKey,
		sort_key: UiSortKey,
	) -> Self {
		Self {
			glyph_run,
			baseline_origin_clip,
			tint,
			draw_key,
			sort_key,
		}
	}
}