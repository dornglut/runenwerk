// Owner: Engine UI Text - Tests
#[cfg(test)]
mod tests {
    use super::{GlyphMetrics, TextSampling, build_glyph_instances};
    use crate::plugins::ui::domain::{UiDrawCmd, UiDrawList};
    use std::collections::HashMap;

    #[test]
    fn build_glyph_instances_advances_horizontally() {
        let mut glyphs = HashMap::new();
        glyphs.insert(
            'a',
            GlyphMetrics {
                uv_min: [0.0, 0.0],
                uv_max: [0.5, 0.5],
                size_px: [8.0, 10.0],
                bearing_px: [0.0, 8.0],
                advance_px: 9.0,
            },
        );
        glyphs.insert(
            'b',
            GlyphMetrics {
                uv_min: [0.5, 0.0],
                uv_max: [1.0, 0.5],
                size_px: [8.0, 10.0],
                bearing_px: [0.0, 8.0],
                advance_px: 9.0,
            },
        );
        glyphs.insert(
            ' ',
            GlyphMetrics {
                uv_min: [0.0, 0.5],
                uv_max: [0.5, 1.0],
                size_px: [4.0, 10.0],
                bearing_px: [0.0, 8.0],
                advance_px: 5.0,
            },
        );

        let draw = UiDrawList {
            commands: vec![UiDrawCmd::Text {
                x: 10.0,
                y: 20.0,
                content: "ab".to_string(),
                color: [1.0, 1.0, 1.0, 1.0],
                size: 14.0,
                clip: None,
            }],
        };

        let instances =
            build_glyph_instances(&draw, &glyphs, 14.0, 16.0, 12.0, TextSampling::Alpha);
        assert_eq!(instances.len(), 2);
        assert!((instances[0].rect[0] - 10.0).abs() < 0.001);
        assert!((instances[1].rect[0] - 19.0).abs() < 0.001);
    }
}
