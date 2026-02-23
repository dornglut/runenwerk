use crate::gameplay::{
    count_alive_enemies, gameplay_apply_live_config, player_entity, spawn_enemy, spawn_player,
};
use engine::plugins::scene::domain::{QuestState, SceneLayer, WorldToOverlayMessage};
use engine::runtime::EngineData;

pub fn gameplay_bootstrap_system(data: &mut EngineData) -> anyhow::Result<()> {
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.scene.layer() != SceneLayer::World {
        return Ok(());
    }

    if player_entity(&ctx.world).is_none() {
        let cfg = &ctx.gameplay_config;
        let _ = spawn_player(&mut ctx.world, cfg.player_spawn_x, cfg.player_spawn_y, cfg);
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Quest {
                quest: "Awaken In Grotto".to_string(),
                state: QuestState::Started,
            });
    }

    let alive_enemies = count_alive_enemies(&ctx.world);
    let desired_enemies = ctx.gameplay_config.enemy_count.max(1) as usize;
    if alive_enemies < desired_enemies {
        for idx in alive_enemies..desired_enemies {
            let cfg = &ctx.gameplay_config;
            let x = cfg.enemy_start_x + (idx as f32 * cfg.enemy_spacing);
            let y = cfg.enemy_start_y + (idx as f32 * cfg.enemy_spacing);
            let _ = spawn_enemy(&mut ctx.world, x, y, cfg);
        }
    }

    gameplay_apply_live_config(ctx);
    Ok(())
}
