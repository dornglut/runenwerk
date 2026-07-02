use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_text::{FontId, GlyphRun, PositionedGlyph};

use super::{BASE_CONTROLS_TEXT_EDITING_PROOF_ID, TextEditingReport};

#[derive(Debug, Clone, PartialEq)]
pub struct TextEditingVisualProof {
    pub proof_id: String,
    pub report: TextEditingReport,
}

pub fn text_editing_report_to_visual_proof(report: TextEditingReport) -> TextEditingVisualProof {
    TextEditingVisualProof {
        proof_id: BASE_CONTROLS_TEXT_EDITING_PROOF_ID.to_owned(),
        report,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextEditingProofRenderFrame {
    pub proof_id: String,
    pub frame: UiFrame,
    pub summary: TextEditingProofRenderSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditingProofRenderSummary {
    pub descriptor_rows: usize,
    pub accepted_intent_rows: usize,
    pub suppressed_intent_rows: usize,
    pub lifecycle_rows: usize,
    pub caret_rows: usize,
    pub selection_rows: usize,
    pub composition_rows: usize,
    pub has_main_inspector_and_report: bool,
    pub no_bypass_proven: bool,
}

pub fn text_editing_visual_proof_to_frame(
    proof: &TextEditingVisualProof,
) -> TextEditingProofRenderFrame {
    let report = &proof.report;
    let size = UiSize::new(940.0, 660.0);
    let mut primitives = Vec::new();
    let mut order = 0_u32;

    panel(
        &mut primitives,
        &mut order,
        UiRect::new(16.0, 16.0, 288.0, 620.0),
        "main: editable target",
    );
    panel(
        &mut primitives,
        &mut order,
        UiRect::new(324.0, 16.0, 288.0, 620.0),
        "inspector: text editing",
    );
    panel(
        &mut primitives,
        &mut order,
        UiRect::new(632.0, 16.0, 288.0, 620.0),
        "report: evidence",
    );

    label(
        &mut primitives,
        &mut order,
        32.0,
        58.0,
        &format!("targets={}", report.descriptor_evidence.len()),
    );
    label(
        &mut primitives,
        &mut order,
        32.0,
        78.0,
        &format!("caret={}", report.caret_evidence.len()),
    );
    label(
        &mut primitives,
        &mut order,
        32.0,
        98.0,
        &format!("selection={}", report.selection_evidence.len()),
    );
    label(
        &mut primitives,
        &mut order,
        32.0,
        118.0,
        &format!("composition={}", report.composition_evidence.len()),
    );
    label(
        &mut primitives,
        &mut order,
        340.0,
        58.0,
        &format!("modes={}", mode_list(report)),
    );
    label(
        &mut primitives,
        &mut order,
        340.0,
        78.0,
        &format!("accepted={}", report.accepted_edit_intents.len()),
    );
    label(
        &mut primitives,
        &mut order,
        340.0,
        98.0,
        &format!("suppressed={}", report.suppressed_edit_intents.len()),
    );
    label(
        &mut primitives,
        &mut order,
        648.0,
        58.0,
        &format!("steps={}", report.input_steps.len()),
    );
    label(
        &mut primitives,
        &mut order,
        648.0,
        78.0,
        &format!("lifecycle={}", report.lifecycle_transitions.len()),
    );
    label(
        &mut primitives,
        &mut order,
        648.0,
        98.0,
        &format!(
            "no_bypass={}",
            report.boundary_assertions.no_bypass_evidence()
        ),
    );

    let mut surface = UiSurface::new(UiSurfaceId(14), size);
    surface.push_layer(UiLayer::with_primitives(UiLayerId(0), primitives));
    TextEditingProofRenderFrame {
        proof_id: proof.proof_id.clone(),
        frame: UiFrame::with_surfaces(vec![surface]),
        summary: TextEditingProofRenderSummary {
            descriptor_rows: report.descriptor_evidence.len(),
            accepted_intent_rows: report.accepted_edit_intents.len(),
            suppressed_intent_rows: report.suppressed_edit_intents.len(),
            lifecycle_rows: report.lifecycle_transitions.len(),
            caret_rows: report.caret_evidence.len(),
            selection_rows: report.selection_evidence.len(),
            composition_rows: report.composition_evidence.len(),
            has_main_inspector_and_report: !report.descriptor_evidence.is_empty()
                && !report.input_steps.is_empty()
                && !report.accepted_edit_intents.is_empty(),
            no_bypass_proven: report.boundary_assertions.no_bypass_evidence(),
        },
    }
}

pub fn base_controls_text_editing_proof_frame(
    report: TextEditingReport,
) -> TextEditingProofRenderFrame {
    let proof = text_editing_report_to_visual_proof(report);
    text_editing_visual_proof_to_frame(&proof)
}

fn mode_list(report: &TextEditingReport) -> String {
    let mut modes = report
        .descriptor_evidence
        .iter()
        .map(|descriptor| descriptor.mode.clone())
        .collect::<Vec<_>>();
    modes.sort();
    modes.dedup();
    modes.join(",")
}

fn panel(primitives: &mut Vec<UiPrimitive>, order: &mut u32, area: UiRect, title: &str) {
    primitives.push(
        RectPrimitive::new(
            area,
            3.0,
            UiPaint::rgba(0.10, 0.11, 0.13, 1.0),
            UiDrawKey::new(1401, None),
            sort_key(order),
        )
        .into(),
    );
    primitives.push(
        BorderPrimitive::new(
            area,
            3.0,
            1.0,
            UiPaint::WHITE,
            UiDrawKey::new(1402, None),
            sort_key(order),
        )
        .into(),
    );
    label(primitives, order, area.x + 12.0, area.y + 20.0, title);
}

fn label(primitives: &mut Vec<UiPrimitive>, order: &mut u32, x: f32, y: f32, text: &str) {
    primitives.push(
        GlyphRunPrimitive::new(
            glyph_run(text, UiPoint::new(x, y)),
            Some(UiRect::new(x, y - 12.0, 250.0, 16.0)),
            UiPaint::WHITE,
            UiDrawKey::new(1403, None),
            sort_key(order),
        )
        .into(),
    );
}

fn sort_key(order: &mut u32) -> UiSortKey {
    let key = UiSortKey::new(0, 0, *order);
    *order += 1;
    key
}

fn glyph_run(text: &str, origin: UiPoint) -> GlyphRun {
    let advance = 7.0;
    let glyphs = text
        .chars()
        .enumerate()
        .map(|(index, ch)| PositionedGlyph {
            ch,
            origin: UiPoint::new(origin.x + index as f32 * advance, origin.y),
            advance,
        })
        .collect();
    GlyphRun {
        font_id: FontId(14),
        font_size: 12.0,
        size: UiSize::new(text.chars().count() as f32 * advance, 14.0),
        glyphs,
    }
}
