use crate::plugins::render::features::world::sdf_raymarch::{
    RenderSdfRaymarchAccelerationConfig, RenderSdfRaymarchAccelerationReport,
    RenderSdfRaymarchAccelerationResource, inspect_sdf_raymarch_acceleration,
};
use crate::plugins::render::features::world::sdf_residency::RenderSdfResidencyResource;

pub fn inspect_render_sdf_raymarch_acceleration(
    residency: &RenderSdfResidencyResource,
    config: RenderSdfRaymarchAccelerationConfig,
) -> RenderSdfRaymarchAccelerationReport {
    inspect_sdf_raymarch_acceleration(residency, config)
}

pub fn inspect_last_render_sdf_raymarch_acceleration(
    acceleration: &RenderSdfRaymarchAccelerationResource,
) -> &RenderSdfRaymarchAccelerationReport {
    acceleration.last_report()
}
