use crate::gameplay::{
    gameplay_bootstrap_system, gameplay_combat_system, gameplay_move_system, gameplay_sense_system,
};
use crate::systems::{
    game_command_apply_system, game_command_execute_system, world_render_extract_system,
};
use anyhow::Result;
use engine::plugins::default_engine_plugins;
use engine::runtime::{EnginePlugin, EngineScheduleBuilder};

pub struct GameSimulationPlugin;
pub struct GameCommandPlugin;

impl EnginePlugin for GameSimulationPlugin {
    fn name(&self) -> &'static str {
        "game_simulation"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "gameplay_bootstrap",
            gameplay_bootstrap_system,
            &["world_scene_update"],
        );
        builder.add_node_with_edges(
            "gameplay_sense",
            gameplay_sense_system,
            &["gameplay_bootstrap"],
        );
        builder.add_node_with_edges("gameplay_move", gameplay_move_system, &["gameplay_sense"]);
        builder.add_node_with_edges(
            "gameplay_combat",
            gameplay_combat_system,
            &["gameplay_move"],
        );
        builder.add_node_with_edges(
            "world_render_extract",
            world_render_extract_system,
            &["gameplay_combat"],
        );
        // Keep scene message formatting/apply in the engine ScenePlugin, but enforce
        // gameplay+render extraction completion before scene messages are flushed to UI.
        builder.add_edge("world_render_extract", "scene_overlay_format_messages");
        Ok(())
    }
}

impl EnginePlugin for GameCommandPlugin {
    fn name(&self) -> &'static str {
        "game_commands"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "game_command_apply",
            game_command_apply_system,
            &["scene_overlay_apply_messages"],
        );
        builder.add_node_with_edges(
            "game_command_execute",
            game_command_execute_system,
            &["game_command_apply"],
        );
        // Ensure engine-owned UI layout runs after game-owned command processing.
        builder.add_edge("game_command_execute", "overlay_ui_layout");
        Ok(())
    }
}

pub fn default_game_plugins() -> Vec<Box<dyn EnginePlugin>> {
    vec![Box::new(GameSimulationPlugin), Box::new(GameCommandPlugin)]
}

pub fn full_game_plugins() -> Vec<Box<dyn EnginePlugin>> {
    let mut plugins = default_engine_plugins();
    plugins.extend(default_game_plugins());
    plugins
}

#[cfg(test)]
mod tests {
    use super::{default_game_plugins, full_game_plugins};
    use engine::runtime::EngineScheduleBuilder;

    #[test]
    fn full_game_plugins_build_scheduler_successfully() {
        let plugins = full_game_plugins();
        let mut builder = EngineScheduleBuilder::new();
        for plugin in &plugins {
            plugin
                .configure(&mut builder)
                .expect("plugin configure should succeed");
        }
        assert!(builder.build_scheduler().is_ok());
    }

    #[test]
    fn game_plugins_require_engine_plugins_for_prereq_nodes() {
        let plugins = default_game_plugins();
        let mut builder = EngineScheduleBuilder::new();
        for plugin in &plugins {
            plugin
                .configure(&mut builder)
                .expect("plugin configure should succeed");
        }
        assert!(builder.build_scheduler().is_err());
    }
}
