use crate::app::App;

use super::inspect::{RenderDebugConfigResource, RenderDebugControlResource};
use super::{RenderFlow, RenderFlowRegistryResource};

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
        if self
            .world()
            .resource::<RenderFlowRegistryResource>()
            .is_err()
        {
            self.insert_resource(RenderFlowRegistryResource::default());
        }
        if let Ok(registry) = self
            .world_mut()
            .resource_mut::<RenderFlowRegistryResource>()
        {
            registry.upsert_flow(flow);
        }
        self
    }

    fn update_render_debug_control<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugControlResource),
    {
        self.init_resource::<RenderDebugControlResource>();
        if let Ok(control) = self
            .world_mut()
            .resource_mut::<RenderDebugControlResource>()
        {
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

    #[test]
    fn preplugin_flow_registration_is_preserved_without_activating_render() {
        let mut app = App::headless();

        app.add_render_flow(RenderFlow::new("render.owner.preplugin"));

        assert!(!render_integration_is_active(app.world()));
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("owner extension should materialize the flow registry")
                .flow_count(),
            1
        );

        app.add_plugin(RenderPlugin);

        assert!(render_integration_is_active(app.world()));
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .expect("RenderPlugin should preserve the preconfigured flow registry")
                .flow_count(),
            1
        );
    }

    #[test]
    fn preplugin_debug_state_is_preserved_without_activating_render() {
        let mut app = App::headless();

        app.update_render_debug_control(|control| {
            control.provenance_enabled = true;
        });
        app.update_render_debug_config(|config| {
            config
                .capture_selectors
                .push(RenderCaptureSelector::named_pass_surface_color(
                    "render.owner.preplugin",
                    "surface",
                ));
        });

        assert!(!render_integration_is_active(app.world()));
        assert!(
            app.world()
                .resource::<RenderDebugControlResource>()
                .expect("owner extension should materialize debug control")
                .provenance_enabled
        );
        assert_eq!(
            app.world()
                .resource::<RenderDebugConfigResource>()
                .expect("owner extension should materialize debug config")
                .capture_selectors
                .len(),
            1
        );

        app.add_plugin(RenderPlugin);

        assert!(render_integration_is_active(app.world()));
        assert!(
            app.world()
                .resource::<RenderDebugControlResource>()
                .expect("RenderPlugin should preserve preconfigured debug control")
                .provenance_enabled
        );
        assert_eq!(
            app.world()
                .resource::<RenderDebugConfigResource>()
                .expect("RenderPlugin should preserve preconfigured debug config")
                .capture_selectors
                .len(),
            1
        );
    }
}
