use crate::gameplay::{Health, MoveIntent, Position, PreviousPosition};
use engine::runtime::EngineData;

pub fn gameplay_move_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.scene.world.visible || data.scene.world.paused {
        return Ok(());
    }
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.overlay_consumed {
        return Ok(());
    }

    let dt = ctx.delta_seconds.clamp(0.0, 0.25);
    let bounds = &ctx.gameplay_config.bounds;
    let entities: Vec<_> = ctx.world.entities_with::<Position>().collect();
    for entity in entities {
        let Some(health) = ctx.world.get_component::<Health>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }

        let Some(intent) = ctx.world.get_component::<MoveIntent>(entity).copied() else {
            continue;
        };
        let Some(pos) = ctx.world.get_component::<Position>(entity).copied() else {
            continue;
        };

        if let Some(prev) = ctx.world.get_component_mut::<PreviousPosition>(entity) {
            prev.x = pos.x;
            prev.y = pos.y;
        }

        if let Some(next) = ctx.world.get_component_mut::<Position>(entity) {
            next.x = (next.x + intent.dx * dt).clamp(bounds.min_x, bounds.max_x);
            next.y = (next.y + intent.dy * dt).clamp(bounds.min_y, bounds.max_y);
        }
    }

    Ok(())
}
