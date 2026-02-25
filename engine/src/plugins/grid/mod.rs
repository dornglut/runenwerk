use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use anyhow::Result;

pub struct GridPlugin;

#[derive(Debug, Copy, Clone, Default)]
pub struct GridRuntimeConfig {
    pub chunk_size: f32,
    pub chunk_load_radius: u32,
    pub infinite_world: bool,
}

impl EnginePlugin for GridPlugin {
    fn name(&self) -> &'static str {
        "grid"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("grid_prepare", grid_prepare_system, &["world_scene_update"]);
        builder.add_edge("grid_prepare", "frame_render_prepare");
        Ok(())
    }
}

pub fn grid_prepare_system(data: &mut EngineData) -> anyhow::Result<()> {
    let (chunk_size, chunk_load_radius, infinite_world) = {
        let cfg = &data.scene.world_runtime.ctx.gameplay_config;
        (cfg.chunk_size, cfg.chunk_load_radius, cfg.infinite_world)
    };
    let next = GridRuntimeConfig {
        chunk_size,
        chunk_load_radius,
        infinite_world,
    };
    if let Some(existing) = data
        .render_resources
        .get_resource_mut::<GridRuntimeConfig>()
    {
        *existing = next;
    } else {
        data.render_resources.insert_resource(next);
    }
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
