use ui_render_data::UiFrame;
use crate::plugins::UiFrameProducerId;

#[derive(Debug, Clone, Default)]
pub struct PreparedUiFrameContribution {
    pub submissions: Vec<PreparedUiFrameSubmission>,
}

impl PreparedUiFrameContribution {
    pub fn is_empty(&self) -> bool {
        self.submissions
            .iter()
            .all(PreparedUiFrameSubmission::is_empty)
    }

    pub fn first_rect_shader_asset_id(&self) -> Option<&str> {
        self.submissions
            .iter()
            .find_map(|submission| submission.rect_shader_asset_id.as_deref())
    }
}

#[derive(Debug, Clone, Default)]
pub struct PreparedUiFrameSubmission {
    pub producer_id: UiFrameProducerId,
    pub route: String,
    pub layer: i32,
    pub priority: i32,
    pub frame: UiFrame,
    pub rect_shader_asset_id: Option<String>,
}

impl PreparedUiFrameSubmission {
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
