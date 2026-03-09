// Owner: Engine UI Text - File Font Provider
#[derive(Debug, Default)]
pub struct FileFontProvider;

impl FileFontProvider {
    fn msdf_paths() -> [(&'static str, &'static str); 2] {
        [
            (
                "assets/fonts/console_msdf.png",
                "assets/fonts/console_msdf.json",
            ),
            (
                "engine/assets/console_msdf.png",
                "engine/assets/console_msdf.json",
            ),
        ]
    }

    fn ttf_paths() -> [&'static str; 2] {
        [
            "assets/fonts/JetBrainsMono-Regular.ttf",
            "engine/assets/JetBrainsMono-Regular.ttf",
        ]
    }

    fn load_msdf_assets() -> Option<LoadedFontAtlas> {
        for (png_path, json_path) in Self::msdf_paths() {
            if !Path::new(png_path).exists() || !Path::new(json_path).exists() {
                continue;
            }

            let png = image::open(png_path).ok()?;
            let (img_w, img_h) = png.dimensions();
            let rgba = png.to_rgba8().into_raw();

            let json_text = fs::read_to_string(json_path).ok()?;
            let parsed: MsdfAtlasJson = serde_json::from_str(&json_text).ok()?;

            let atlas_w = parsed.atlas.width.unwrap_or(img_w).max(1);
            let atlas_h = parsed.atlas.height.unwrap_or(img_h).max(1);
            let y_origin_bottom = parsed
                .atlas
                .y_origin
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("bottom"))
                .unwrap_or(false);

            let base_size = parsed.atlas.size.unwrap_or(32.0).max(1.0);
            let line_height = parsed
                .metrics
                .as_ref()
                .and_then(|m| m.line_height)
                .unwrap_or(base_size)
                * base_size;
            let ascent = parsed
                .metrics
                .as_ref()
                .and_then(|m| m.ascender)
                .unwrap_or(0.8)
                * base_size;

            let mut glyphs = HashMap::new();
            for glyph in parsed.glyphs {
                let Some(ch) = char::from_u32(glyph.unicode) else {
                    continue;
                };

                let advance_px = glyph.advance * base_size;
                let (size_px, bearing_px, uv_min, uv_max) =
                    if let (Some(plane), Some(atlas)) = (glyph.plane_bounds, glyph.atlas_bounds) {
                        let uv_left = atlas.left / atlas_w as f32;
                        let uv_right = atlas.right / atlas_w as f32;

                        let (uv_top, uv_bottom) = if y_origin_bottom {
                            (
                                1.0 - (atlas.top / atlas_h as f32),
                                1.0 - (atlas.bottom / atlas_h as f32),
                            )
                        } else {
                            (atlas.top / atlas_h as f32, atlas.bottom / atlas_h as f32)
                        };

                        (
                            [
                                (plane.right - plane.left) * base_size,
                                (plane.top - plane.bottom) * base_size,
                            ],
                            [plane.left * base_size, plane.top * base_size],
                            [uv_left, uv_top],
                            [uv_right, uv_bottom],
                        )
                    } else {
                        ([0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0])
                    };

                glyphs.insert(
                    ch,
                    GlyphMetrics {
                        uv_min,
                        uv_max,
                        size_px,
                        bearing_px,
                        advance_px,
                    },
                );
            }

            if glyphs.is_empty() {
                continue;
            }

            return Some(LoadedFontAtlas {
                width: atlas_w,
                height: atlas_h,
                rgba8_pixels: rgba,
                glyphs,
                base_size,
                line_height: line_height.max(1.0),
                ascent,
                sampling: TextSampling::Msdf,
            });
        }

        None
    }

    fn load_ttf_bytes() -> Option<Vec<u8>> {
        for path in Self::ttf_paths() {
            if Path::new(path).exists() {
                if let Ok(bytes) = fs::read(path) {
                    return Some(bytes);
                }
            }
        }
        None
    }

    fn placeholder() -> LoadedFontAtlas {
        let mut glyphs = HashMap::new();
        for ch in 32u8..=126u8 {
            glyphs.insert(
                ch as char,
                GlyphMetrics {
                    uv_min: [0.0, 0.0],
                    uv_max: [1.0, 1.0],
                    size_px: [8.0, 14.0],
                    bearing_px: [0.0, 11.0],
                    advance_px: 8.0,
                },
            );
        }

        LoadedFontAtlas {
            width: 1,
            height: 1,
            rgba8_pixels: vec![255, 255, 255, 255],
            glyphs,
            base_size: 18.0,
            line_height: 22.0,
            ascent: 16.0,
            sampling: TextSampling::Alpha,
        }
    }
}

impl FontAtlasProvider for FileFontProvider {
    fn load_default_font(&self) -> LoadedFontAtlas {
        if let Some(msdf) = Self::load_msdf_assets() {
            return msdf;
        }

        let Some(ttf_bytes) = Self::load_ttf_bytes() else {
            return Self::placeholder();
        };

        build_bitmap_font_atlas(&ttf_bytes).unwrap_or_else(Self::placeholder)
    }
}

#[derive(Debug, Deserialize)]
struct MsdfAtlasJson {
    atlas: MsdfAtlasInfo,
    #[serde(default)]
    metrics: Option<MsdfMetrics>,
    glyphs: Vec<MsdfGlyph>,
}

#[derive(Debug, Deserialize)]
struct MsdfAtlasInfo {
    #[serde(default)]
    size: Option<f32>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(rename = "yOrigin", default)]
    y_origin: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MsdfMetrics {
    #[serde(rename = "lineHeight", default)]
    line_height: Option<f32>,
    #[serde(default)]
    ascender: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct MsdfGlyph {
    unicode: u32,
    advance: f32,
    #[serde(rename = "planeBounds", default)]
    plane_bounds: Option<MsdfBounds>,
    #[serde(rename = "atlasBounds", default)]
    atlas_bounds: Option<MsdfBounds>,
}

#[derive(Debug, Deserialize)]
struct MsdfBounds {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}
