use crate::plugins::render::features::{
    FeatureFallbackPolicy, RenderFeatureDescriptor, SCENE_ROUTE_RENDER_FEATURE_ID,
    UI_RENDER_FEATURE_ID,
};

pub const UI_RENDER_FEATURE_LABEL: &str = "ui";

pub fn ui_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(UI_RENDER_FEATURE_ID, UI_RENDER_FEATURE_LABEL)
        .depends_on(SCENE_ROUTE_RENDER_FEATURE_ID)
        .with_order_hint(0)
        .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses)
}
