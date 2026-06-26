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
