//! File: domain/ui/ui_text/src/layout.rs
//! Purpose: Renderer-independent text layout result contracts.

use ui_math::{UiPoint, UiSize};

use crate::{FontAtlasSource, FontId, TextStyle};

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayoutRequest<'a> {
    pub text: &'a str,
    pub style: &'a TextStyle,
    pub max_width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionedGlyph {
    pub ch: char,
    pub origin: UiPoint,
    pub advance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    pub font_id: FontId,
    pub font_size: f32,
    pub glyphs: Vec<PositionedGlyph>,
    pub size: UiSize,
}

pub trait TextLayouter: Send + Sync {
    fn layout(
        &self,
        atlas_source: &dyn FontAtlasSource,
        request: TextLayoutRequest<'_>,
    ) -> Option<GlyphRun>;
}
