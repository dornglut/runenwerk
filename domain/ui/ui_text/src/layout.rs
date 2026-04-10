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

#[derive(Debug, Clone, Copy, Default)]
pub struct AtlasTextLayouter;

impl TextLayouter for AtlasTextLayouter {
    fn layout(
        &self,
        atlas_source: &dyn FontAtlasSource,
        request: TextLayoutRequest<'_>,
    ) -> Option<GlyphRun> {
        let atlas = atlas_source.atlas(request.style.font_id)?;
        let base_size = atlas.metrics.base_size.max(f32::EPSILON);
        let scale = request.style.font_size / base_size;
        let default_line_height = atlas.metrics.line_height.max(1.0) * scale;
        let line_height = request.style.line_height_or_default(default_line_height);
        let baseline = atlas.metrics.ascender * scale;
        let max_width = request.max_width.unwrap_or(f32::INFINITY);

        let mut glyphs = Vec::with_capacity(request.text.chars().count());
        let mut pen_x = 0.0_f32;
        let mut max_advance = 0.0_f32;

        for ch in request.text.chars() {
            if ch == '\n' {
                break;
            }

            let metrics = atlas.glyphs.get(&ch).or_else(|| atlas.glyphs.get(&'?'))?;
            let advance = metrics.advance.max(0.0) * scale;
            if pen_x + advance > max_width {
                break;
            }

            glyphs.push(PositionedGlyph {
                ch,
                origin: UiPoint::new(pen_x, baseline),
                advance,
            });
            pen_x += advance;
            max_advance = max_advance.max(pen_x);
        }

        Some(GlyphRun {
            font_id: request.style.font_id,
            font_size: request.style.font_size,
            glyphs,
            size: UiSize::new(max_advance, line_height.max(1.0)),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{FontFaceMetrics, GlyphMetrics, MsdfFontAtlas, TextAlign, TextOverflow, TextWrap};

    #[derive(Default)]
    struct TestAtlasSource {
        atlas: Option<MsdfFontAtlas>,
    }

    impl FontAtlasSource for TestAtlasSource {
        fn atlas(&self, _font_id: FontId) -> Option<&MsdfFontAtlas> {
            self.atlas.as_ref()
        }
    }

    #[test]
    fn atlas_text_layouter_emits_positioned_glyphs() {
        let mut glyphs = HashMap::new();
        glyphs.insert(
            'A',
            GlyphMetrics {
                advance: 10.0,
                plane_left: 0.0,
                plane_top: 8.0,
                plane_right: 8.0,
                plane_bottom: -2.0,
                atlas_left: 0.0,
                atlas_top: 0.0,
                atlas_right: 0.1,
                atlas_bottom: 0.1,
            },
        );
        glyphs.insert(
            '?',
            GlyphMetrics {
                advance: 10.0,
                plane_left: 0.0,
                plane_top: 8.0,
                plane_right: 8.0,
                plane_bottom: -2.0,
                atlas_left: 0.0,
                atlas_top: 0.0,
                atlas_right: 0.1,
                atlas_bottom: 0.1,
            },
        );

        let source = TestAtlasSource {
            atlas: Some(MsdfFontAtlas {
                font_id: FontId(1),
                texture_width: 256,
                texture_height: 256,
                metrics: FontFaceMetrics {
                    ascender: 9.0,
                    descender: -3.0,
                    line_height: 12.0,
                    base_size: 12.0,
                },
                glyphs,
            }),
        };

        let style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: None,
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        };

        let run = AtlasTextLayouter
            .layout(
                &source,
                TextLayoutRequest {
                    text: "AAA",
                    style: &style,
                    max_width: None,
                },
            )
            .expect("layouter should produce a run");

        assert_eq!(run.glyphs.len(), 3);
        assert_eq!(run.glyphs[1].origin.x, 10.0);
        assert_eq!(run.glyphs[2].origin.x, 20.0);
        assert!(run.size.width >= 30.0);
    }
}
