//! Host-neutral metrics for the one logical primary presentation.

pub(crate) fn ensure_primary_presentation_metrics(world: &mut runen_ecs::World) {
    if !world.has_resource::<PrimaryPresentationMetricsResource>() {
        world.insert_resource(PrimaryPresentationMetricsResource::default());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, runen_ecs::Component, runen_ecs::Resource)]
pub struct PrimaryPresentationMetricsResource {
    size_px: (u32, u32),
    scale_factor: f64,
}

impl PrimaryPresentationMetricsResource {
    pub fn new(size_px: (u32, u32), scale_factor: f64) -> Self {
        Self {
            size_px: normalize_extent(size_px),
            scale_factor,
        }
    }

    pub fn size_px(&self) -> (u32, u32) {
        self.size_px
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn update(&mut self, size_px: (u32, u32), scale_factor: f64) {
        self.size_px = normalize_extent(size_px);
        self.scale_factor = scale_factor;
    }
}

impl Default for PrimaryPresentationMetricsResource {
    fn default() -> Self {
        Self::new((1280, 720), 1.0)
    }
}

fn normalize_extent(size_px: (u32, u32)) -> (u32, u32) {
    (size_px.0.max(1), size_px.1.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn bare_apps_do_not_manufacture_primary_presentation_metrics() {
        for app in [App::new(), App::headless()] {
            assert!(
                app.world().resource::<PrimaryPresentationMetricsResource>().is_err(),
                "bare App must not manufacture logical presentation state"
            );
        }
    }

    #[test]
    fn presentation_activation_preserves_caller_supplied_metrics() {
        let mut world = runen_ecs::World::new();
        let expected = PrimaryPresentationMetricsResource::new((960, 540), 1.5);
        world.insert_resource(expected);
        ensure_primary_presentation_metrics(&mut world);
        assert_eq!(
            world.resource::<PrimaryPresentationMetricsResource>().unwrap(),
            &expected
        );
    }

    #[test]
    fn primary_presentation_metrics_default_is_host_neutral() {
        assert_eq!(
            PrimaryPresentationMetricsResource::default(),
            PrimaryPresentationMetricsResource {
                size_px: (1280, 720),
                scale_factor: 1.0,
            }
        );
    }

    #[test]
    fn primary_presentation_metrics_normalize_zero_extent() {
        let mut metrics = PrimaryPresentationMetricsResource::new((0, 720), 1.0);
        assert_eq!(metrics.size_px, (1, 720));

        metrics.update((1920, 0), 1.5);
        assert_eq!(metrics.size_px, (1920, 1));
        assert_eq!(metrics.scale_factor, 1.5);
    }
}
