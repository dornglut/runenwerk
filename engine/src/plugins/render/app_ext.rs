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
        if self.world().resource::<RenderFlowRegistryResource>().is_err() {
            self.insert_resource(RenderFlowRegistryResource::default());
        }
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
