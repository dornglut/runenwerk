use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use anyhow::Result;

pub struct GridPlugin;

impl EnginePlugin for GridPlugin {
    fn name(&self) -> &'static str {
        "grid"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("grid_prepare", grid_prepare_system, &["world_scene_update"]);
        Ok(())
    }
}

pub fn grid_prepare_system(data: &mut EngineData) -> anyhow::Result<()> {
    let (chunk_size, chunk_load_radius, infinite_world) = {
        let cfg = &data.scene.world_runtime.ctx.gameplay_config;
        (cfg.chunk_size, cfg.chunk_load_radius, cfg.infinite_world)
    };
    let world_render = data.world_render_mut();
    world_render.chunk_size = chunk_size;
    world_render.chunk_load_radius = chunk_load_radius;
    world_render.infinite_world = infinite_world;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::GridPlugin;
    use crate::runtime::{EnginePlugin, EngineScheduleBuilder};

    #[test]
    fn grid_plugin_requires_scene_plugin_nodes() {
        let mut builder = EngineScheduleBuilder::new();
        GridPlugin
            .configure(&mut builder)
            .expect("grid plugin should configure");
        assert!(builder.build_scheduler().is_err());
    }
}
