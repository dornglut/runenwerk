use crate::plugins::render::features::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedUiFrameContribution,
    PreparedUiFrameSubmission, UiFrameSubmissionRegistryResource,
};
use crate::runtime::{Res, ResMut};
use rusttype::{Font, Scale, point};
use std::collections::HashMap;
use ui_text::{FontAtlasSource, FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas};

pub const DEFAULT_EDITOR_FONT_ID: FontId = FontId(1);
const DEFAULT_EDITOR_FONT_BASE_SIZE: f32 = 32.0;
const DEFAULT_ATLAS_COLUMNS: u32 = 16;
const DEFAULT_GLYPH_PADDING: u32 = 3;
const DEFAULT_EDITOR_FONT_BYTES: &[u8] =
    include_bytes!("../../../../../../assets/fonts/JetBrainsMono-Regular.ttf");

#[derive(Debug, Clone)]
pub struct UiFontAtlasImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct UiFontAtlasResource {
    atlases: HashMap<FontId, MsdfFontAtlas>,
    images: HashMap<FontId, UiFontAtlasImage>,
}

impl UiFontAtlasResource {
    pub fn atlas_image(&self, font_id: FontId) -> Option<&UiFontAtlasImage> {
        self.images.get(&font_id)
    }

    pub fn atlas_for_texture_id(
        &self,
        texture_id: u64,
    ) -> Option<(&MsdfFontAtlas, &UiFontAtlasImage)> {
        let font_id = FontId(texture_id);
        let atlas = self.atlases.get(&font_id)?;
        let image = self.images.get(&font_id)?;
        Some((atlas, image))
    }
}

impl FontAtlasSource for UiFontAtlasResource {
    fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas> {
        self.atlases.get(&font_id)
    }
}

impl Default for UiFontAtlasResource {
    fn default() -> Self {
        let (atlas, image) =
            build_default_editor_font_atlas().expect("default editor font atlas should load");
        let mut atlases = HashMap::new();
        let mut images = HashMap::new();
        atlases.insert(DEFAULT_EDITOR_FONT_ID, atlas);
        images.insert(DEFAULT_EDITOR_FONT_ID, image);
        Self { atlases, images }
    }
}

fn build_default_editor_font_atlas() -> anyhow::Result<(MsdfFontAtlas, UiFontAtlasImage)> {
    let font = Font::try_from_bytes(DEFAULT_EDITOR_FONT_BYTES)
        .ok_or_else(|| anyhow::anyhow!("invalid editor font bytes"))?;
    let scale = Scale::uniform(DEFAULT_EDITOR_FONT_BASE_SIZE);
    let v_metrics = font.v_metrics(scale);
    let charset = default_editor_charset();
    let mut glyph_entries = Vec::with_capacity(charset.len());
    let mut max_width = 1_u32;
    let mut max_height = 1_u32;

    for ch in charset {
        let scaled = font.glyph(ch).scaled(scale);
        let h_metrics = scaled.h_metrics();
        let exact = scaled.exact_bounding_box();
        let positioned = scaled.positioned(point(0.0, 0.0));
        let pixel_bb = positioned.pixel_bounding_box();

        let (bitmap_width, bitmap_height, bitmap) = match pixel_bb {
            Some(bb) => {
                let width = bb.width().max(0) as u32;
                let height = bb.height().max(0) as u32;
                if width == 0 || height == 0 {
                    (0, 0, Vec::new())
                } else {
                    let mut data = vec![0_u8; (width * height) as usize];
                    positioned.draw(|x, y, value| {
                        let index = (y as u32 * width + x as u32) as usize;
                        data[index] = (value.clamp(0.0, 1.0) * 255.0) as u8;
                    });
                    (width, height, data)
                }
            }
            None => (0, 0, Vec::new()),
        };

        max_width = max_width.max(bitmap_width);
        max_height = max_height.max(bitmap_height);

        let (plane_left, plane_top, plane_right, plane_bottom) = exact
            .map(|bb| (bb.min.x, bb.max.y, bb.max.x, bb.min.y))
            .unwrap_or((0.0, 0.0, 0.0, 0.0));

        glyph_entries.push((
            ch,
            h_metrics.advance_width,
            plane_left,
            plane_top,
            plane_right,
            plane_bottom,
            bitmap_width,
            bitmap_height,
            bitmap,
        ));
    }

    let cell_width = max_width + DEFAULT_GLYPH_PADDING * 2;
    let cell_height = max_height + DEFAULT_GLYPH_PADDING * 2;
    let columns = DEFAULT_ATLAS_COLUMNS.max(1);
    let rows = ((glyph_entries.len() as u32).saturating_add(columns - 1)) / columns;
    let atlas_width = (columns * cell_width).max(1);
    let atlas_height = (rows * cell_height).max(1);
    let mut atlas_pixels = vec![0_u8; (atlas_width * atlas_height) as usize];
    let mut glyphs = HashMap::new();

    for (index, entry) in glyph_entries.into_iter().enumerate() {
        let (
            ch,
            advance,
            plane_left,
            plane_top,
            plane_right,
            plane_bottom,
            bitmap_width,
            bitmap_height,
            bitmap,
        ) = entry;
        let col = (index as u32) % columns;
        let row = (index as u32) / columns;
        let x = col * cell_width + DEFAULT_GLYPH_PADDING;
        let y = row * cell_height + DEFAULT_GLYPH_PADDING;

        if bitmap_width > 0 && bitmap_height > 0 {
            for src_y in 0..bitmap_height {
                let dst_y = y + src_y;
                let dst_offset = (dst_y * atlas_width + x) as usize;
                let src_offset = (src_y * bitmap_width) as usize;
                let copy_len = bitmap_width as usize;
                atlas_pixels[dst_offset..dst_offset + copy_len]
                    .copy_from_slice(&bitmap[src_offset..src_offset + copy_len]);
            }
        }

        let atlas_left = x as f32 / atlas_width as f32;
        let atlas_top = y as f32 / atlas_height as f32;
        let atlas_right = (x + bitmap_width.max(1)) as f32 / atlas_width as f32;
        let atlas_bottom = (y + bitmap_height.max(1)) as f32 / atlas_height as f32;

        glyphs.insert(
            ch,
            GlyphMetrics {
                advance,
                plane_left,
                plane_top,
                plane_right,
                plane_bottom,
                atlas_left,
                atlas_top,
                atlas_right,
                atlas_bottom,
            },
        );
    }

    let atlas = MsdfFontAtlas {
        font_id: DEFAULT_EDITOR_FONT_ID,
        texture_width: atlas_width,
        texture_height: atlas_height,
        metrics: FontFaceMetrics {
            ascender: v_metrics.ascent,
            descender: v_metrics.descent,
            line_height: v_metrics.ascent - v_metrics.descent + v_metrics.line_gap,
            base_size: DEFAULT_EDITOR_FONT_BASE_SIZE,
        },
        glyphs,
    };

    let image = UiFontAtlasImage {
        width: atlas_width,
        height: atlas_height,
        pixels: atlas_pixels,
    };

    Ok((atlas, image))
}

fn default_editor_charset() -> Vec<char> {
    let mut chars = (32_u8..=126_u8).map(char::from).collect::<Vec<_>>();
    chars.push('•');
    chars
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedUiFrameResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedUiFrameContribution,
}

impl Default for PreparedUiFrameResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedUiFrameContribution::default(),
        }
    }
}

pub fn prepare_ui_feature_resource_system(
    submissions: Res<UiFrameSubmissionRegistryResource>,
    mut prepared: ResMut<PreparedUiFrameResource>,
) {
    let ordered = submissions.ordered_submissions();
    if ordered.is_empty() {
        prepared.status = FeatureContributionStatus::Missing;
        prepared.payload = PreparedUiFrameContribution::default();
        return;
    }

    prepared.status = FeatureContributionStatus::Ready;
    prepared.payload = PreparedUiFrameContribution {
        submissions: ordered
            .into_iter()
            .map(|submission| PreparedUiFrameSubmission {
                producer_id: submission.producer_id.as_str().to_string(),
                route: submission.route.as_str().to_string(),
                layer: submission.order.layer,
                priority: submission.order.priority,
                frame: submission.frame.clone(),
                rect_shader_asset_id: submission.rect_shader_asset_id.clone(),
            })
            .collect(),
    };
}
