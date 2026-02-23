use crate::gameplay::{
    Attack, EnemyTag, Health, Position, PreviousPosition, can_attack, deal_damage, distance,
    nearest_alive_enemy, player_entity, set_attack_cooldown, tick_cooldown,
};
use engine::plugins::scene::domain::{QuestState, WorldToOverlayMessage};
use engine::runtime::EngineData;

pub fn gameplay_combat_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.scene.world.visible || data.scene.world.paused {
        return Ok(());
    }
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.overlay_consumed {
        return Ok(());
    }

    let Some(player_entity) = player_entity(&ctx.world) else {
        return Ok(());
    };

    tick_cooldown(&mut ctx.world, player_entity);
    let enemy_entities: Vec<_> = ctx.world.entities_with::<EnemyTag>().collect();
    for entity in &enemy_entities {
        tick_cooldown(&mut ctx.world, *entity);
    }

    let Some(player_pos) = ctx.world.get_component::<Position>(player_entity).copied() else {
        return Ok(());
    };

    if can_attack(&ctx.world, player_entity)
        && let Some(target) = nearest_alive_enemy(&ctx.world, player_pos)
    {
        let target_pos = ctx
            .world
            .get_component::<Position>(target)
            .copied()
            .unwrap_or(player_pos);
        let range = ctx
            .world
            .get_component::<Attack>(player_entity)
            .map(|a| a.range)
            .unwrap_or(0.0);
        let dist = distance(player_pos, target_pos);
        if dist <= range {
            let damage = ctx
                .world
                .get_component::<Attack>(player_entity)
                .map(|a| a.damage)
                .unwrap_or(1);
            set_attack_cooldown(&mut ctx.world, player_entity);
            deal_damage(
                &mut ctx.world,
                target,
                damage,
                &mut data.scene.channels.world_to_overlay,
                "Player",
                "Enemy",
            );
        }
    }

    for enemy in enemy_entities {
        if !can_attack(&ctx.world, enemy) {
            continue;
        }
        let Some(enemy_pos) = ctx.world.get_component::<Position>(enemy).copied() else {
            continue;
        };
        let range = ctx
            .world
            .get_component::<Attack>(enemy)
            .map(|a| a.range)
            .unwrap_or(0.0);
        if distance(enemy_pos, player_pos) > range {
            continue;
        }

        let damage = ctx
            .world
            .get_component::<Attack>(enemy)
            .map(|a| a.damage)
            .unwrap_or(1);
        set_attack_cooldown(&mut ctx.world, enemy);
        deal_damage(
            &mut ctx.world,
            player_entity,
            damage,
            &mut data.scene.channels.world_to_overlay,
            "Enemy",
            "Player",
        );
    }

    let mut removed = 0u32;
    let enemies_now: Vec<_> = ctx.world.entities_with::<EnemyTag>().collect();
    for enemy in enemies_now {
        let alive = ctx
            .world
            .get_component::<Health>(enemy)
            .map(|h| h.current > 0)
            .unwrap_or(false);
        if !alive {
            ctx.world.remove_entity(enemy);
            removed = removed.saturating_add(1);
        }
    }
    if removed > 0 {
        ctx.enemy_kills = ctx.enemy_kills.saturating_add(removed);
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Loot {
                item: "Glowshard".to_string(),
                amount: removed,
                rarity: "common".to_string(),
            });

        if ctx.enemy_kills == 1 {
            data.scene
                .channels
                .world_to_overlay
                .push(WorldToOverlayMessage::Quest {
                    quest: "Cull The Swarm".to_string(),
                    state: QuestState::Started,
                });
        }
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Quest {
                quest: "Cull The Swarm".to_string(),
                state: QuestState::Progress {
                    current: ctx.enemy_kills,
                    goal: 20,
                },
            });
        if ctx.enemy_kills >= 20 {
            data.scene
                .channels
                .world_to_overlay
                .push(WorldToOverlayMessage::Quest {
                    quest: "Cull The Swarm".to_string(),
                    state: QuestState::Completed,
                });
        }
    }

    let player_dead = ctx
        .world
        .get_component::<Health>(player_entity)
        .map(|h| h.current <= 0)
        .unwrap_or(false);
    if player_dead {
        let spawn_x = ctx.gameplay_config.player_spawn_x;
        let spawn_y = ctx.gameplay_config.player_spawn_y;
        if let Some(health) = ctx.world.get_component_mut::<Health>(player_entity) {
            health.current = health.max;
        }
        if let Some(pos) = ctx.world.get_component_mut::<Position>(player_entity) {
            pos.x = spawn_x;
            pos.y = spawn_y;
        }
        if let Some(prev) = ctx
            .world
            .get_component_mut::<PreviousPosition>(player_entity)
        {
            prev.x = spawn_x;
            prev.y = spawn_y;
        }
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Combat {
                source: "Enemy".to_string(),
                target: "Player".to_string(),
                damage: 0,
                critical: false,
            });
    }

    Ok(())
}
