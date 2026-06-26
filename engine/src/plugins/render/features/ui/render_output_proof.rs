//! File: engine/src/plugins/render/features/ui/render_output_proof.rs
//! Purpose: Backend-side proof that UI frame submissions can carry renderer-neutral output evidence.

use crate::plugins::render::api::ids::UiFrameProducerId;
use ui_render_data::UiRenderOutputEvidence;

use super::{UiFrameRoute, UiFrameSubmission};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiFrameSubmissionRenderOutputProof {
    pub producer_id: UiFrameProducerId,
    pub route: UiFrameRoute,
    pub evidence_id: String,
    pub submitted_primitive_count: u32,
    pub evidenced_primitive_count: u32,
    pub diagnostic_count: u32,
}

impl UiFrameSubmissionRenderOutputProof {
    pub fn from_submission(
        submission: &UiFrameSubmission,
        evidence: &UiRenderOutputEvidence,
    ) -> Self {
        Self {
            producer_id: submission.producer_id,
            route: submission.route,
            evidence_id: evidence.evidence_id.clone(),
            submitted_primitive_count: submission.primitive_count_hint() as u32,
            evidenced_primitive_count: evidence.frame_summary.primitive_count,
            diagnostic_count: evidence.diagnostics.len() as u32,
        }
    }

    pub fn primitive_counts_match(&self) -> bool {
        self.submitted_primitive_count == self.evidenced_primitive_count
    }

    pub fn is_valid(&self) -> bool {
        self.primitive_counts_match() && self.diagnostic_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_math::{UiRect, UiSize};
    use ui_render_data::{
        RectPrimitive, UiDrawKey, UiExpectedPrimitiveCount, UiFrame, UiLayer, UiLayerId, UiPaint,
        UiPrimitive, UiPrimitiveFamily, UiRenderOutputProvenance, UiSortKey, UiSurface,
        UiSurfaceId,
    };

    #[test]
    fn render_output_proof_consumes_submission_evidence_without_backend_semantics() {
        let frame = frame_with_rect();
        let evidence = UiRenderOutputEvidence::from_frame(
            "runenwerk.engine.render.ui.evidence.rect",
            UiRenderOutputProvenance::new("ui_runtime.build_ui_frame", "engine.proof"),
            &frame,
            [UiExpectedPrimitiveCount::exactly(UiPrimitiveFamily::Rect, 1)],
        );
        let submission = UiFrameSubmission::new(UiFrameProducerId::try_from_raw(7).unwrap())
            .with_route(UiFrameRoute::Screen)
            .with_frame(frame);

        let proof = UiFrameSubmissionRenderOutputProof::from_submission(&submission, &evidence);

        assert!(proof.is_valid(), "{:?}", proof);
        assert_eq!(proof.producer_id, UiFrameProducerId::try_from_raw(7).unwrap());
        assert_eq!(proof.submitted_primitive_count, 1);
        assert_eq!(proof.evidenced_primitive_count, 1);
    }

    fn frame_with_rect() -> UiFrame {
        let primitive = UiPrimitive::Rect(RectPrimitive::new(
            UiRect::new(0.0, 0.0, 10.0, 10.0),
            0.0,
            UiPaint::WHITE,
            UiDrawKey::new(0, None),
            UiSortKey::new(0, 0, 0),
        ));
        UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            UiSize::new(10.0, 10.0),
            vec![UiLayer::with_primitives(UiLayerId(0), vec![primitive])],
        )])
    }
}
