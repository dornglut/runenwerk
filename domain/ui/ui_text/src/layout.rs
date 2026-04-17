//! File: domain/ui/ui_text/src/layout.rs
//! Purpose: Renderer-independent text layout result contracts.

use ui_math::{UiPoint, UiSize};

use crate::{FontAtlasSource, FontId, TextOverflow, TextStyle};

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
        let line_height = request
            .style
            .line_height_or_default(default_line_height)
            .max(default_line_height);
        let baseline = atlas.metrics.ascender * scale;
        let max_width = request.max_width.unwrap_or(f32::INFINITY);

        let mut glyphs = Vec::with_capacity(request.text.chars().count());
        let mut pen_x = 0.0_f32;
        let mut max_advance = 0.0_f32;
        let mut truncated = false;
        let mut chars = request.text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\n' {
                truncated = chars.peek().is_some();
                break;
            }

            let metrics = atlas.glyphs.get(&ch).or_else(|| atlas.glyphs.get(&'?'))?;
            let advance = metrics.advance.max(0.0) * scale;
            if pen_x + advance > max_width {
                truncated = true;
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

        if truncated && matches!(request.style.overflow, TextOverflow::Ellipsis) {
            let ellipsis_chars: Vec<char> = if atlas.glyphs.contains_key(&'…') {
                vec!['…']
            } else {
                vec!['.', '.', '.']
            };

            let mut ellipsis_width = 0.0_f32;
            for ch in &ellipsis_chars {
                let metrics = atlas.glyphs.get(ch).or_else(|| atlas.glyphs.get(&'?'))?;
                ellipsis_width += metrics.advance.max(0.0) * scale;
            }

            if ellipsis_width <= max_width {
                while !glyphs.is_empty() && pen_x + ellipsis_width > max_width {
                    if let Some(last) = glyphs.pop() {
                        pen_x = (pen_x - last.advance).max(0.0);
                    }
                }

                if pen_x + ellipsis_width <= max_width {
                    for ch in ellipsis_chars {
                        let metrics = atlas.glyphs.get(&ch).or_else(|| atlas.glyphs.get(&'?'))?;
                        let advance = metrics.advance.max(0.0) * scale;
                        glyphs.push(PositionedGlyph {
                            ch,
                            origin: UiPoint::new(pen_x, baseline),
                            advance,
                        });
                        pen_x += advance;
                    }
                    max_advance = max_advance.max(pen_x);
                }
            }
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
    use crate::{
        FontFaceMetrics, GlyphMetrics, MsdfFontAtlas, TextAlign, TextOverflow, TextStyle, TextWrap,
    };

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

    #[test]
    fn atlas_text_layouter_applies_single_char_ellipsis_on_truncation() {
        let source = TestAtlasSource {
            atlas: Some(test_atlas_with_chars(&['A', '?', '…', '.'])),
        };

        let mut style = TextStyle::default();
        style.font_id = FontId(1);
        style.font_size = 12.0;
        style.overflow = TextOverflow::Ellipsis;

        let run = AtlasTextLayouter
            .layout(
                &source,
                TextLayoutRequest {
                    text: "AAAA",
                    style: &style,
                    max_width: Some(25.0),
                },
            )
            .expect("layouter should produce a run");

        let rendered = run.glyphs.iter().map(|glyph| glyph.ch).collect::<String>();
        assert_eq!(rendered, "A…");
    }

    #[test]
    fn atlas_text_layouter_uses_three_dot_fallback_for_ellipsis() {
        let source = TestAtlasSource {
            atlas: Some(test_atlas_with_chars(&['A', '?', '.'])),
        };

        let mut style = TextStyle::default();
        style.font_id = FontId(1);
        style.font_size = 12.0;
        style.overflow = TextOverflow::Ellipsis;

        let run = AtlasTextLayouter
            .layout(
                &source,
                TextLayoutRequest {
                    text: "AAAA",
                    style: &style,
                    max_width: Some(35.0),
                },
            )
            .expect("layouter should produce a run");

        let rendered = run.glyphs.iter().map(|glyph| glyph.ch).collect::<String>();
        assert_eq!(rendered, "...");
    }

    #[test]
    fn atlas_text_layouter_keeps_single_baseline_for_mixed_word_glyphs() {
        let source = TestAtlasSource {
            atlas: Some(test_atlas_with_chars(&['N', 'o', 'n', 'g', 'e', '?'])),
        };
        let style = TextStyle {
            font_id: FontId(1),
            font_size: 12.0,
            color: [1.0, 1.0, 1.0, 1.0],
            line_height: Some(10.0),
            align: TextAlign::Start,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        };

        let run = AtlasTextLayouter
            .layout(
                &source,
                TextLayoutRequest {
                    text: "Nonge",
                    style: &style,
                    max_width: None,
                },
            )
            .expect("layouter should produce a run");

        let baseline = run.glyphs[0].origin.y;
        assert!(
            run.glyphs
                .iter()
                .all(|glyph| (glyph.origin.y - baseline).abs() <= f32::EPSILON),
            "all glyphs in one run must share the same baseline origin",
        );
        assert!(
            run.size.height >= 12.0,
            "line height should never fall below scaled font default",
        );
    }

    fn test_atlas_with_chars(chars: &[char]) -> MsdfFontAtlas {
        let mut glyphs = HashMap::new();
        for ch in chars {
            glyphs.insert(
                *ch,
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
        }

        MsdfFontAtlas {
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
        }
    }
}
