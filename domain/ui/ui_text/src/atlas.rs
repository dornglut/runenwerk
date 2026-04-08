//! File: domain/ui/ui_text/src/atlas.rs
//! Purpose: MSDF atlas contracts.

use std::collections::HashMap;

use crate::{FontFaceMetrics, FontId, GlyphMetrics};

#[derive(Debug, Clone)]
pub struct MsdfFontAtlas {
    pub font_id: FontId,
    pub texture_width: u32,
    pub texture_height: u32,
    pub metrics: FontFaceMetrics,
    pub glyphs: HashMap<char, GlyphMetrics>,
}

pub trait FontAtlasSource: Send + Sync {
    fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas>;
}
