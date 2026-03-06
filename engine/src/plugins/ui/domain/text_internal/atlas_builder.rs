// Owner: Engine UI Text - Bitmap Atlas Builder
#[derive(Clone, Copy)]
struct GlyphPackMeta {
    ch: char,
    width: u32,
    height: u32,
    bearing_x: f32,
    bearing_y: f32,
    advance: f32,
    x: u32,
    y: u32,
}

fn build_bitmap_font_atlas(ttf_bytes: &[u8]) -> Option<LoadedFontAtlas> {
    let font = Font::try_from_bytes(ttf_bytes)?;
    let base_size = 42.0_f32;
    let scale = Scale::uniform(base_size);
    let v_metrics = font.v_metrics(scale);
    let ascent = v_metrics.ascent;
    let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap).max(1.0);

    let mut metas = Vec::new();
    for code in 32u8..=126u8 {
        let ch = code as char;
        let glyph = font.glyph(ch).scaled(scale);
        let advance = glyph.h_metrics().advance_width;

        let positioned = glyph.positioned(point(0.0, 0.0));
        let bb = positioned.pixel_bounding_box();
        let (width, height, bearing_x, bearing_y) = if let Some(bb) = bb {
            (
                bb.width().max(0) as u32,
                bb.height().max(0) as u32,
                bb.min.x as f32,
                (-bb.min.y) as f32,
            )
        } else {
            (0, 0, 0.0, 0.0)
        };

        metas.push(GlyphPackMeta {
            ch,
            width,
            height,
            bearing_x,
            bearing_y,
            advance,
            x: 0,
            y: 0,
        });
    }

    let atlas_width = 2048_u32;
    let padding = 1_u32;
    let mut cursor_x = padding;
    let mut cursor_y = padding;
    let mut row_h = 0_u32;

    for meta in &mut metas {
        if meta.width == 0 || meta.height == 0 {
            continue;
        }

        if cursor_x + meta.width + padding > atlas_width {
            cursor_x = padding;
            cursor_y += row_h + padding;
            row_h = 0;
        }

        meta.x = cursor_x;
        meta.y = cursor_y;
        cursor_x += meta.width + padding;
        row_h = row_h.max(meta.height);
    }

    let atlas_height = (cursor_y + row_h + padding).max(1);
    let mut pixels = vec![0u8; (atlas_width * atlas_height * 4) as usize];

    for meta in &metas {
        if meta.width == 0 || meta.height == 0 {
            continue;
        }

        let glyph = font
            .glyph(meta.ch)
            .scaled(scale)
            .positioned(point(-meta.bearing_x, meta.bearing_y));

        glyph.draw(|x, y, v| {
            let px = meta.x + x;
            let py = meta.y + y;
            if px >= atlas_width || py >= atlas_height {
                return;
            }
            let idx = ((py * atlas_width + px) * 4) as usize;
            let d = (v.clamp(0.0, 1.0) * 255.0) as u8;
            pixels[idx] = d;
            pixels[idx + 1] = d;
            pixels[idx + 2] = d;
            pixels[idx + 3] = 255;
        });
    }

    let mut glyphs = HashMap::new();
    for meta in metas {
        if meta.width == 0 || meta.height == 0 {
            glyphs.insert(
                meta.ch,
                GlyphMetrics {
                    uv_min: [0.0, 0.0],
                    uv_max: [0.0, 0.0],
                    size_px: [0.0, 0.0],
                    bearing_px: [meta.bearing_x, meta.bearing_y],
                    advance_px: meta.advance,
                },
            );
            continue;
        }

        glyphs.insert(
            meta.ch,
            GlyphMetrics {
                uv_min: [
                    meta.x as f32 / atlas_width as f32,
                    meta.y as f32 / atlas_height as f32,
                ],
                uv_max: [
                    (meta.x + meta.width) as f32 / atlas_width as f32,
                    (meta.y + meta.height) as f32 / atlas_height as f32,
                ],
                size_px: [meta.width as f32, meta.height as f32],
                bearing_px: [meta.bearing_x, meta.bearing_y],
                advance_px: meta.advance,
            },
        );
    }

    Some(LoadedFontAtlas {
        width: atlas_width,
        height: atlas_height,
        rgba8_pixels: pixels,
        glyphs,
        base_size,
        line_height,
        ascent,
        sampling: TextSampling::Alpha,
    })
}
