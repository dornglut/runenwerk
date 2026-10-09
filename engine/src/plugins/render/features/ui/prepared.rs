use crate::plugins::RenderFrameProducerId;
use runenui_runtime::PaintPublication;
use ui_render_data::UiFrame;

use super::RunenUiPublicationId;

#[derive(Debug, Clone, Default)]
pub struct PreparedUiFrameContribution {
    /// Unmigrated UI payloads prepared under their shared compositor positions.
    pub submissions: Vec<PreparedSurfaceFrameSubmission>,
    /// Direct RunenUI paint publications, never predecessor UiFrame payloads.
    pub runenui_submissions: Vec<PreparedRunenUiPaintSubmission>,
    /// One transient ordering domain; never a second render-command IR.
    pub ordered: Vec<PreparedUiSubmissionKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedUiSubmissionKind {
    Legacy(usize),
    RunenUi(usize),
}

#[derive(Debug, Clone)]
pub struct PreparedRunenUiPaintSubmission {
    pub producer_id: RenderFrameProducerId,
    pub publication_id: RunenUiPublicationId,
    pub submission_order: u32,
    pub publication: PaintPublication,
}

impl PreparedUiFrameContribution {
    pub fn is_empty(&self) -> bool {
        self.submissions
            .iter()
            .all(PreparedSurfaceFrameSubmission::is_empty)
            && self
                .runenui_submissions
                .iter()
                .all(|submission| submission.publication.scene().is_empty())
    }

    pub fn first_rect_shader_asset_id(&self) -> Option<&str> {
        self.submissions
            .iter()
            .find_map(|submission| submission.rect_shader_asset_id.as_deref())
    }
}

#[derive(Debug, Clone)]
pub struct PreparedSurfaceFrameSubmission {
    pub producer_id: RenderFrameProducerId,
    /// Position within the mixed legacy/RunenUI compositor sequence.
    pub submission_order: u32,
    pub route: String,
    pub layer: i32,
    pub priority: i32,
    pub frame: UiFrame,
    pub rect_shader_asset_id: Option<String>,
}

impl PreparedSurfaceFrameSubmission {
    pub fn primitive_count_hint(&self) -> usize {
        self.frame
            .surfaces
            .iter()
            .map(|surface| {
                surface
                    .layers
                    .iter()
                    .map(|layer| layer.primitives.len())
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.frame.is_empty()
    }
}
