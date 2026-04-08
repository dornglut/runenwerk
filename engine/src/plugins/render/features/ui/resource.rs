use crate::plugins::render::features::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedUiFrameContribution,
    PreparedUiFrameSubmission, UiFrameSubmissionRegistryResource,
};
use crate::runtime::{Res, ResMut};

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
