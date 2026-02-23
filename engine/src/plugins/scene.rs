use crate::runtime::{EnginePlugin, EngineScheduleBuilder};
use crate::systems::{
    scene_overlay_apply_messages_system, scene_overlay_format_messages_system,
    scene_transition_system, world_scene_update_system,
};
use anyhow::Result;

pub struct ScenePlugin;

impl EnginePlugin for ScenePlugin {
    fn name(&self) -> &'static str {
        "scene"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "scene_transition",
            scene_transition_system,
            &["overlay_ui_editor"],
        );
        builder.add_node_with_edges(
            "world_scene_update",
            world_scene_update_system,
            &["scene_transition"],
        );
        builder.add_node_with_edges(
            "scene_overlay_format_messages",
            scene_overlay_format_messages_system,
            &["world_scene_update"],
        );
        builder.add_node_with_edges(
            "scene_overlay_apply_messages",
            scene_overlay_apply_messages_system,
            &["scene_overlay_format_messages"],
        );
        Ok(())
    }
}
