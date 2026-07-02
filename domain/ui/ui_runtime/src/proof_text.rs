//! File: domain/ui/ui_runtime/src/proof_text.rs
//! Purpose: Runtime-local helpers for projecting renderer-neutral text proof evidence.

use std::collections::HashMap;

use ui_math::{UiPoint, UiRect};
use ui_text::{
    AtlasTextLayouter, FontAtlasSource, FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas,
    TextBlock, TextBlockId, TextBlockLayoutRequest, TextBlockLayoutResult, TextDirectionPolicy,
    TextHeightConstraint, TextHorizontalAlign, TextLayoutPolicy, TextLayouter,
    TextLineHeightPolicy, TextRun, TextRunId, TextSemanticRole, TextStyle, TextVerticalAlign,
    TextWhitespacePolicy, TextWidthConstraint, TextWrapPolicy,
};

pub(crate) fn layout_text_in_bounds(
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    block_id: TextBlockId,
    text: &str,
    style: &TextStyle,
    bounds: UiRect,
    horizontal_align: TextHorizontalAlign,
    vertical_align: TextVerticalAlign,
) -> TextBlockLayoutResult {
    let block = text_block(
        block_id,
        text,
        style.clone(),
        bounds.width,
        horizontal_align,
    );
    let mut layout = layouter.layout(atlas_source, TextBlockLayoutRequest::new(block));
    let vertical_offset = vertical_offset(&layout, bounds, vertical_align);
    translate_layout(
        &mut layout,
        UiPoint::new(bounds.x, bounds.y + vertical_offset),
    );
    layout
}

pub(crate) fn proof_label_layout(
    text: &str,
    origin: UiPoint,
    width: f32,
    font_id: FontId,
    sort_order: u32,
) -> TextBlockLayoutResult {
    let style = TextStyle {
        font_id,
        font_size: 12.0,
        line_height: TextLineHeightPolicy::Absolute(14.0),
        ..TextStyle::default()
    };
    let atlas = ProofTextAtlasSource::new(font_id);
    let layouter = AtlasTextLayouter;
    layout_text_in_bounds(
        &atlas,
        &layouter,
        TextBlockId(u64::from(sort_order) + 1),
        text,
        &style,
        UiRect::new(origin.x, origin.y - 12.0, width.max(0.0), 16.0),
        TextHorizontalAlign::Start,
        TextVerticalAlign::Start,
    )
}

#[cfg(test)]
pub(crate) fn text_from_primitive(run: &ui_render_data::GlyphRunPrimitive) -> String {
    run.visual_runs
        .iter()
        .flat_map(|visual_run| visual_run.glyphs.iter())
        .map(|glyph| glyph.source_text_preview.as_str())
        .collect::<String>()
}

fn text_block(
    block_id: TextBlockId,
    text: &str,
    style: TextStyle,
    max_width: f32,
    horizontal_align: TextHorizontalAlign,
) -> TextBlock {
    TextBlock::new(block_id, style)
        .with_run(TextRun::new(TextRunId(1), text).with_semantic_role(TextSemanticRole::Label))
        .with_semantic_role(TextSemanticRole::Label)
        .with_layout(TextLayoutPolicy {
            width_constraint: TextWidthConstraint::Exact(max_width.max(0.0)),
            height_constraint: TextHeightConstraint::Unconstrained,
            wrap: TextWrapPolicy::NoWrap,
            whitespace: TextWhitespacePolicy::Preserve,
            horizontal_align,
            vertical_align: TextVerticalAlign::Start,
            overflow: ui_text::TextOverflowPolicy::Clip,
            max_lines: Some(1),
            text_direction: TextDirectionPolicy::Ltr,
        })
}

fn vertical_offset(
    layout: &TextBlockLayoutResult,
    bounds: UiRect,
    vertical_align: TextVerticalAlign,
) -> f32 {
    match vertical_align {
        TextVerticalAlign::Start | TextVerticalAlign::Baseline => 0.0,
        TextVerticalAlign::Center => ((bounds.height - layout.measured_size.height) * 0.5).max(0.0),
        TextVerticalAlign::End => (bounds.height - layout.measured_size.height).max(0.0),
    }
}

fn translate_layout(layout: &mut TextBlockLayoutResult, origin: UiPoint) {
    layout.content_bounds = translate_rect(layout.content_bounds, origin);
    layout.ink_bounds = translate_rect(layout.ink_bounds, origin);
    for line in &mut layout.line_metrics {
        line.origin = translate_point(line.origin, origin);
        line.baseline_y += origin.y;
        line.line_box = translate_rect(line.line_box, origin);
        line.ink_bounds = translate_rect(line.ink_bounds, origin);
    }
    for visual_run in &mut layout.visual_runs {
        visual_run.bounds = translate_rect(visual_run.bounds, origin);
        for glyph in &mut visual_run.glyphs {
            glyph.origin = translate_point(glyph.origin, origin);
            glyph.bounds = translate_rect(glyph.bounds, origin);
        }
    }
}

fn translate_point(point: UiPoint, origin: UiPoint) -> UiPoint {
    UiPoint::new(point.x + origin.x, point.y + origin.y)
}

fn translate_rect(rect: UiRect, origin: UiPoint) -> UiRect {
    UiRect::new(
        rect.x + origin.x,
        rect.y + origin.y,
        rect.width,
        rect.height,
    )
}

struct ProofTextAtlasSource {
    atlas: MsdfFontAtlas,
}

impl ProofTextAtlasSource {
    fn new(font_id: FontId) -> Self {
        let mut glyphs = HashMap::new();
        for ch in 32_u8..=126_u8 {
            glyphs.insert(
                char::from(ch),
                GlyphMetrics {
                    advance: 7.0,
                    plane_left: 0.0,
                    plane_top: 9.0,
                    plane_right: 7.0,
                    plane_bottom: -3.0,
                    atlas_left: 0.0,
                    atlas_top: 0.0,
                    atlas_right: 0.1,
                    atlas_bottom: 0.1,
                },
            );
        }
        Self {
            atlas: MsdfFontAtlas {
                font_id,
                texture_width: 256,
                texture_height: 256,
                metrics: FontFaceMetrics {
                    ascender: 9.0,
                    descender: -3.0,
                    line_height: 14.0,
                    base_size: 12.0,
                },
                glyphs,
            },
        }
    }
}

impl FontAtlasSource for ProofTextAtlasSource {
    fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas> {
        (self.atlas.font_id == font_id).then_some(&self.atlas)
    }
}
