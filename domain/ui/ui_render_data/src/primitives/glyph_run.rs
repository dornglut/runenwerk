//! File: domain/ui/ui_render_data/src/primitives/glyph_run.rs
//! Purpose: Renderer-neutral text visual-run primitive referencing layout evidence.

use ui_math::UiRect;
use ui_text::{
    GlyphRun, TextBlockId, TextBlockLayoutResult, TextClusterRange, TextDirectionPolicy,
    TextGlyph, TextOverflowEvidence, TextRunId, TextStyle, TextVisualRun,
};

use crate::{UiDrawKey, UiPaint, UiSortKey};

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRunPrimitive {
    pub block_id: TextBlockId,
    pub visual_runs: Vec<TextVisualRun>,
    pub line_count: usize,
    pub glyph_count: usize,
    pub overflow_evidence: TextOverflowEvidence,
    pub baseline_origin_clip: Option<UiRect>,
    pub tint: UiPaint,
    pub draw_key: UiDrawKey,
    pub sort_key: UiSortKey,
}

impl GlyphRunPrimitive {
    pub fn new(
        payload: impl Into<GlyphRunPrimitivePayload>,
        baseline_origin_clip: Option<UiRect>,
        tint: UiPaint,
        draw_key: UiDrawKey,
        sort_key: UiSortKey,
    ) -> Self {
        let payload = payload.into();
        let (block_id, visual_runs, line_count, glyph_count, overflow_evidence) = match payload {
            GlyphRunPrimitivePayload::Layout(layout) => (
                layout.block_id,
                layout.visual_runs,
                layout.line_count,
                layout.glyph_count,
                layout.overflow_evidence,
            ),
            GlyphRunPrimitivePayload::Positioned(run) => positioned_run_to_visual_evidence(run),
        };
        Self { block_id, visual_runs, line_count, glyph_count, overflow_evidence, baseline_origin_clip, tint, draw_key, sort_key }
    }

    pub fn from_visual_runs(
        block_id: TextBlockId,
        visual_runs: Vec<TextVisualRun>,
        overflow_evidence: TextOverflowEvidence,
        baseline_origin_clip: Option<UiRect>,
        tint: UiPaint,
        draw_key: UiDrawKey,
        sort_key: UiSortKey,
    ) -> Self {
        let line_count = visual_runs.iter().map(|run| run.line_index).max().map_or(0, |index| index as usize + 1);
        let glyph_count = visual_runs.iter().map(|run| run.glyphs.len()).sum();
        Self { block_id, visual_runs, line_count, glyph_count, overflow_evidence, baseline_origin_clip, tint, draw_key, sort_key }
    }
}

pub enum GlyphRunPrimitivePayload {
    Layout(TextBlockLayoutResult),
    Positioned(GlyphRun),
}

impl From<TextBlockLayoutResult> for GlyphRunPrimitivePayload {
    fn from(value: TextBlockLayoutResult) -> Self {
        Self::Layout(value)
    }
}

impl From<GlyphRun> for GlyphRunPrimitivePayload {
    fn from(value: GlyphRun) -> Self {
        Self::Positioned(value)
    }
}

fn positioned_run_to_visual_evidence(
    run: GlyphRun,
) -> (TextBlockId, Vec<TextVisualRun>, usize, usize, TextOverflowEvidence) {
    let glyph_count = run.glyphs.len();
    let style = TextStyle { font_id: run.font_id, font_size: run.font_size, ..TextStyle::default() };
    let mut glyphs = Vec::with_capacity(run.glyphs.len());
    for (index, glyph) in run.glyphs.into_iter().enumerate() {
        glyphs.push(TextGlyph {
            draw_order: index as u32,
            line_index: 0,
            run_id: TextRunId(0),
            span_id: None,
            font_id: run.font_id,
            glyph_key: format!("char:{}", glyph.ch),
            cluster_range: TextClusterRange::new(index as u32, index as u32 + 1),
            origin: glyph.origin,
            advance: glyph.advance,
            bounds: UiRect::new(glyph.origin.x, glyph.origin.y - run.font_size, glyph.advance, run.font_size),
            source_text_preview: glyph.ch.to_string(),
            replacement: false,
        });
    }
    let visual_runs = vec![TextVisualRun {
        visual_run_id: 0,
        line_index: 0,
        run_id: TextRunId(0),
        span_id: None,
        font_id: run.font_id,
        style,
        direction: TextDirectionPolicy::Ltr,
        glyphs,
        bounds: UiRect::new(0.0, 0.0, run.size.width, run.size.height),
    }];
    (TextBlockId(0), visual_runs, 1, glyph_count, TextOverflowEvidence::none())
}
