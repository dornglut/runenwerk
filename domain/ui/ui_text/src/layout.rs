//! File: domain/ui/ui_text/src/layout.rs
//! Purpose: Renderer-independent text block layout request/result contracts.

use ui_math::{UiPoint, UiSize};

use crate::{FontAtlasSource, FontId, TextBlock, TextBlockLayoutResult};

#[derive(Debug, Clone, PartialEq)]
pub struct TextBlockLayoutRequest {
    pub block: TextBlock,
}

impl TextBlockLayoutRequest {
    pub fn new(block: TextBlock) -> Self {
        Self { block }
    }
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
        request: TextBlockLayoutRequest,
    ) -> TextBlockLayoutResult;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AtlasTextLayouter;

impl TextLayouter for AtlasTextLayouter {
    fn layout(
        &self,
        atlas_source: &dyn FontAtlasSource,
        request: TextBlockLayoutRequest,
    ) -> TextBlockLayoutResult {
        crate::proof_layout::layout_text_block(atlas_source, request)
    }
}
