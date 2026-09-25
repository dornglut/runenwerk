use crate::app::App;

use super::composition::RenderFlowRegistryResource;
use super::inspect::{RenderDebugConfigResource, RenderDebugControlResource};
use super::RenderFlow;

pub trait AppRenderExt {
    fn add_render_flow(&mut self, flow: RenderFlow) -> &mut Self;

    fn update_render_debug_control<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugControlResource);

    fn update_render_debug_config<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugConfigResource);
}

impl AppRenderExt for App {
    fn add_render_flow(&mut self, flow: RenderFlow) -> &mut Self {
        self.init_resource::<RenderFlowRegistryResource>();
        if let Ok(registry) = self.world_mut().resource_mut::<RenderFlowRegistryResource>() {
            registry.upsert_flow(flow);
        }
        self
    }

    fn update_render_debug_control<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugControlResource),
    {
        self.init_resource::<RenderDebugControlResource>();
        if let Ok(control) = self.world_mut().resource_mut::<RenderDebugControlResource>() {
            update(control);
        }
        self
    }

    fn update_render_debug_config<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugConfigResource),
    {
        self.init_resource::<RenderDebugConfigResource>();
        if let Ok(config) = self.world_mut().resource_mut::<RenderDebugConfigResource>() {
            update(config);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::inspect::RenderCaptureSelector;
    use crate::plugins::render::{RenderPlugin, render_integration_is_active};

    fn minimal_flow(label: &str) -> RenderFlow {
        RenderFlow::new(label)
            .with_surface_color()
            .expect("render flow authoring should succeed")
            .fullscreen_pass("compose")
            .write_surface_color()
            .expect("render flow authoring should succeed")
            .finish()
            .validate()
            .expect("owner-extension proof flow should validate")
    }

    #[test]
    fn render_owner_configuration_is_preserved_without_implicit_activation() {
        let mut app = App::headless();
        app.add_render_flow(minimal_flow("owner.preconfigured"));
        app.update_render_debug_control(|control| {
            control.provenance_enabled = true;
            control.capture_enabled = true;
        });
        app.update_render_debug_config(|config| {
            config.capture_selectors.push(
                RenderCaptureSelector::named_pass_surface_color(
                    "owner.preconfigured",
                    "compose",
                ),
            );
        });

        assert!(!render_integration_is_active(app.world()));
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("Render owner flow state should exist")
                .flow_count(),
            1
        );

        app.add_plugin(RenderPlugin);

        assert!(render_integration_is_active(app.world()));
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("RenderPlugin should preserve preconfigured flow state")
                .flow_count(),
            1
        );
        let control = app
            .world()
            .resource::<RenderDebugControlResource>()
            .expect("RenderPlugin should preserve preconfigured debug control");
        assert!(control.provenance_enabled);
        assert!(control.capture_enabled);
        assert_eq!(
            app.world()
                .resource::<RenderDebugConfigResource>()
                .expect("RenderPlugin should preserve preconfigured debug config")
                .capture_selectors
                .len(),
            1
        );
    }

    #[test]
    fn render_flow_registration_remains_valid_after_app_startup() {
        let mut app = App::headless();
        app.add_plugin(RenderPlugin);
        let mut app = app
            .run_for_frames(0)
            .expect("Render-selected App should admit and run Startup");

        app.add_render_flow(minimal_flow("owner.runtime"));

        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("Render flow registry should remain mutable after Startup")
                .flow_count(),
            1
        );
    }
}
