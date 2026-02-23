use crate::gameplay::{EnemyTag, Health, MoveIntent, MoveSpeed, Position, player_entity};
use engine::runtime::EngineData;

pub fn gameplay_sense_system(data: &mut EngineData) -> anyhow::Result<()> {
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
    let Some(player_pos) = ctx.world.get_component::<Position>(player_entity).copied() else {
        return Ok(());
    };

    let player_speed = ctx
        .world
        .get_component::<MoveSpeed>(player_entity)
        .map(|s| s.value)
        .unwrap_or(1.0);
    if let Some(intent) = ctx.world.get_component_mut::<MoveIntent>(player_entity) {
        intent.dx = ctx.player_move_x * player_speed;
        intent.dy = ctx.player_move_y * player_speed;
    }

    let enemy_entities: Vec<_> = ctx.world.entities_with::<EnemyTag>().collect();
    for entity in enemy_entities {
        let Some(health) = ctx.world.get_component::<Health>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }

        let Some(position) = ctx.world.get_component::<Position>(entity).copied() else {
            continue;
        };
        let speed = ctx
            .world
            .get_component::<MoveSpeed>(entity)
            .map(|s| s.value)
            .unwrap_or(0.0);

        let dx = player_pos.x - position.x;
        let dy = player_pos.y - position.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if let Some(intent) = ctx.world.get_component_mut::<MoveIntent>(entity) {
            if dist > f32::EPSILON {
                intent.dx = (dx / dist) * speed;
                intent.dy = (dy / dist) * speed;
            } else {
                intent.dx = 0.0;
                intent.dy = 0.0;
            }
        }
    }

    Ok(())
}
