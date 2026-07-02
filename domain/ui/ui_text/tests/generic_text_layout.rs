use std::collections::HashMap;

use ui_text::{
    AtlasTextLayouter, FontAtlasSource, FontFaceMetrics, FontId, GlyphMetrics, MsdfFontAtlas,
    TextBlock, TextBlockLayoutRequest, TextDecoration, TextDirectionPolicy, TextEllipsisPlacement,
    TextHorizontalAlign, TextLayoutPolicy, TextLayouter, TextOverflowPolicy, TextSemanticRole,
    TextSpan, TextSpanId, TextSpanStyle, TextSourceRange, TextWhitespacePolicy,
    TextWidthConstraint, TextWrapPolicy,
};

#[test]
fn simple_label_emits_line_metrics_and_glyph_evidence() {
    let result = layout(TextBlock::label("Label"));
    assert_eq!(result.input_run_count, 1);
    assert_eq!(result.line_count, 1);
    assert_eq!(result.glyph_count, 5);
    assert_eq!(result.line_metrics[0].baseline_y, 9.0);
}

#[test]
fn inline_spans_keep_span_and_semantic_evidence() {
    let result = layout(TextBlock::inline_spans(
        "Heading body helper",
        vec![
            TextSpan::new(TextSpanId(1), TextSourceRange::new(0, 7))
                .with_style(TextSpanStyle::inherit().with_decoration(TextDecoration::underline()))
                .with_semantic_role(TextSemanticRole::Heading),
            TextSpan::new(TextSpanId(2), TextSourceRange::new(8, 12))
                .with_semantic_role(TextSemanticRole::Body),
            TextSpan::new(TextSpanId(3), TextSourceRange::new(13, 19))
                .with_semantic_role(TextSemanticRole::Helper),
        ],
    ));
    let span_ids = result
        .visual_runs
        .iter()
        .flat_map(|run| run.glyphs.iter().filter_map(|glyph| glyph.span_id))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(span_ids.len(), 3);
}

#[test]
fn explicit_newline_creates_explicit_break_line() {
    let result = layout(TextBlock::label("one\ntwo"));
    assert_eq!(result.line_count, 2);
    assert!(result.line_metrics[0].is_explicit_break);
}

#[test]
fn word_wrap_prefers_whitespace_boundary() {
    let result = layout(TextBlock::body("alpha beta gamma", 48.0));
    assert!(result.line_count > 1);
    assert!(result.line_metrics.iter().any(|line| line.is_wrapped));
    assert!(!result.diagnostics.iter().any(|diagnostic| diagnostic.code == "ui.text.wrap.word_fell_back_to_character"));
}

#[test]
fn character_wrap_breaks_unbreakable_text() {
    let result = layout(TextBlock::label("aaaaaaaa").with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Max(16.0),
        wrap: TextWrapPolicy::Character,
        ..TextLayoutPolicy::default()
    }));
    assert!(result.line_count > 1);
}

#[test]
fn alignment_shifts_line_origins() {
    let center = layout(TextBlock::label("center").with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Exact(100.0),
        horizontal_align: TextHorizontalAlign::Center,
        ..TextLayoutPolicy::default()
    }));
    let end = layout(TextBlock::label("end").with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Exact(100.0),
        horizontal_align: TextHorizontalAlign::End,
        ..TextLayoutPolicy::default()
    }));
    assert!(center.line_metrics[0].origin.x > 0.0);
    assert!(end.line_metrics[0].origin.x > center.line_metrics[0].origin.x);
}

#[test]
fn clip_overflow_records_without_ellipsis() {
    let result = layout(TextBlock::label("overflow").with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Max(16.0),
        overflow: TextOverflowPolicy::Clip,
        ..TextLayoutPolicy::default()
    }));
    assert!(result.overflow_evidence.clipped);
    assert!(!result.overflow_evidence.ellipsized);
}

#[test]
fn end_ellipsis_is_overflow_decision() {
    let result = layout(TextBlock::label("overflow").with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Max(32.0),
        overflow: TextOverflowPolicy::Ellipsis(TextEllipsisPlacement::End),
        ..TextLayoutPolicy::default()
    }));
    assert!(result.overflow_evidence.ellipsized);
    assert_eq!(result.overflow_evidence.ellipsis_placement, Some(TextEllipsisPlacement::End));
}

#[test]
fn max_line_clamp_is_recorded_before_ellipsis() {
    let result = layout(TextBlock::body("one two three four five", 32.0).with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Max(32.0),
        wrap: TextWrapPolicy::Word,
        whitespace: TextWhitespacePolicy::CollapseRuns,
        max_lines: Some(1),
        overflow: TextOverflowPolicy::Ellipsis(TextEllipsisPlacement::End),
        ..TextLayoutPolicy::default()
    }));
    assert_eq!(result.line_count, 1);
    assert!(result.overflow_evidence.max_lines_applied);
    assert!(result.overflow_evidence.ellipsized);
}

#[test]
fn fallback_glyph_evidence_is_recorded() {
    let result = layout(TextBlock::label("Omega Ω"));
    assert!(result.fallback_evidence.iter().any(|row| row.replacement_glyph_count > 0));
    assert!(result.visual_runs.iter().flat_map(|run| run.glyphs.iter()).any(|glyph| glyph.replacement));
}

#[test]
fn unsupported_policies_emit_diagnostics() {
    let start_ellipsis = layout(TextBlock::label("overflow").with_layout(TextLayoutPolicy {
        width_constraint: TextWidthConstraint::Max(16.0),
        overflow: TextOverflowPolicy::Ellipsis(TextEllipsisPlacement::Start),
        ..TextLayoutPolicy::default()
    }));
    let rtl = layout(TextBlock::label("rtl").with_layout(TextLayoutPolicy {
        text_direction: TextDirectionPolicy::Rtl,
        ..TextLayoutPolicy::default()
    }));
    assert!(start_ellipsis.diagnostics.iter().any(|diagnostic| diagnostic.code == "ui.text.ellipsis.unsupported_placement"));
    assert!(rtl.diagnostics.iter().any(|diagnostic| diagnostic.code == "ui.text.direction.rtl_unsupported"));
}

fn layout(block: TextBlock) -> ui_text::TextBlockLayoutResult {
    AtlasTextLayouter.layout(&TestAtlasSource::new(), TextBlockLayoutRequest::new(block))
}

struct TestAtlasSource { atlas: MsdfFontAtlas }
impl TestAtlasSource {
    fn new() -> Self {
        let mut glyphs = HashMap::new();
        for ch in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .-:?".chars() {
            glyphs.insert(ch, GlyphMetrics { advance: if ch == ' ' { 4.0 } else { 8.0 }, plane_left: 0.0, plane_top: 8.0, plane_right: 8.0, plane_bottom: -2.0, atlas_left: 0.0, atlas_top: 0.0, atlas_right: 0.1, atlas_bottom: 0.1 });
        }
        Self { atlas: MsdfFontAtlas { font_id: FontId(0), texture_width: 256, texture_height: 256, metrics: FontFaceMetrics { ascender: 9.0, descender: -3.0, line_height: 12.0, base_size: 12.0 }, glyphs } }
    }
}
impl FontAtlasSource for TestAtlasSource { fn atlas(&self, _font_id: FontId) -> Option<&MsdfFontAtlas> { Some(&self.atlas) } }
