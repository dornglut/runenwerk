use engine::prelude::{FixedTimeConfig, World};

pub(crate) const FIXED_STEP_MIN_SECONDS: f32 = 1.0 / 240.0;
pub(crate) const FIXED_STEP_MAX_SECONDS: f32 = 1.0 / 15.0;
pub(crate) const DEFAULT_FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;

pub(crate) fn fixed_step_seconds(world: &World) -> f32 {
    world
        .resource::<FixedTimeConfig>()
        .map(|config| config.step_seconds)
        .unwrap_or(DEFAULT_FIXED_STEP_SECONDS)
        .clamp(FIXED_STEP_MIN_SECONDS, FIXED_STEP_MAX_SECONDS)
}
