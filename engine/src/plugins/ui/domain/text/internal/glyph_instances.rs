// Owner: Engine UI Text - Glyph Instance Builder
pub fn build_glyph_instances(
    draw_list: &UiDrawList,
    glyphs: &HashMap<char, GlyphMetrics>,
    base_size: f32,
    line_height: f32,
    ascent: f32,
    sampling: TextSampling,
) -> Vec<GlyphInstanceRaw> {
    let mut instances = Vec::new();
    let fallback = glyphs.get(&' ').copied().unwrap_or(GlyphMetrics {
        uv_min: [0.0, 0.0],
        uv_max: [0.0, 0.0],
        size_px: [0.0, 0.0],
        bearing_px: [0.0, 0.0],
        advance_px: 8.0,
    });

    for cmd in &draw_list.commands {
        let UiDrawCmd::Text {
            x,
            y,
            content,
            color,
            size,
            ..
        } = cmd
        else {
            continue;
        };

        let mut pen_x = *x;
        let scale = if base_size > 0.0 {
            (*size / base_size).max(0.1)
        } else {
            1.0
        };
        let scaled_line_height = line_height * scale;
        let scaled_ascent = ascent * scale;
        let mut baseline_y = *y + scaled_ascent;

        for ch in content.chars() {
            if ch == '\n' {
                pen_x = *x;
                baseline_y += scaled_line_height;
                continue;
            }

            let glyph = glyphs.get(&ch).copied().unwrap_or(fallback);
            let rect_w = glyph.size_px[0] * scale;
            let rect_h = glyph.size_px[1] * scale;

            if rect_w > 0.0 && rect_h > 0.0 {
                let rect_x = pen_x + glyph.bearing_px[0] * scale;
                let rect_y = baseline_y - glyph.bearing_px[1] * scale;
                let (rect_x, rect_y) = if matches!(sampling, TextSampling::Alpha) {
                    // Pixel-snap raster glyph quads to reduce blur when upscaled.
                    (rect_x.round(), rect_y.round())
                } else {
                    (rect_x, rect_y)
                };

                instances.push(GlyphInstanceRaw {
                    rect: [rect_x, rect_y, rect_w, rect_h],
                    uv: [
                        glyph.uv_min[0],
                        glyph.uv_min[1],
                        glyph.uv_max[0],
                        glyph.uv_max[1],
                    ],
                    color: *color,
                });
            }

            pen_x += glyph.advance_px * scale;
        }
    }

    instances
}
