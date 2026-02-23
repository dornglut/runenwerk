use crate::gameplay::{EnemyTag, Health, PlayerTag, Position, PreviousPosition};
use engine::plugins::render::domain::{MAX_WORLD_RENDER_AGENTS, WorldRenderAgent};
use engine::runtime::EngineData;

pub fn world_render_extract_system(data: &mut EngineData) -> anyhow::Result<()> {
    let frame = &mut data.world_render;
    frame.agents.clear();
    frame.model_proxies.clear();
    frame.world_scene_label = data.scene.world.active.label().to_string();
    frame.overlay_scene_label = data.scene.active_overlay().label().to_string();
    frame.scene_render_graph_passes = data
        .scene
        .registry
        .render_graph_contributions(data.scene.world.active, data.scene.active_overlay());
    frame.world_paused = data.scene.world.paused;
    frame.camera_yaw = data.scene.world_runtime.ctx.camera_yaw;
    frame.camera_pitch = data.scene.world_runtime.ctx.camera_pitch;
    frame.camera_distance = data.scene.world_runtime.ctx.camera_distance;
    frame.camera_pitch_min = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .pitch_min;
    frame.camera_pitch_max = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .pitch_max;
    frame.camera_distance_min = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .distance_min;
    frame.camera_distance_max = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .distance_max;
    frame.camera_follow_dampening = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .camera
        .follow_dampening;
    frame.chunk_size = data.scene.world_runtime.ctx.gameplay_config.chunk_size;
    frame.chunk_load_radius = data
        .scene
        .world_runtime
        .ctx
        .gameplay_config
        .chunk_load_radius;
    frame.infinite_world = data.scene.world_runtime.ctx.gameplay_config.infinite_world;

    let bounds = &data.scene.world_runtime.ctx.gameplay_config.bounds;
    frame.world_bounds = [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y];

    if !data.scene.world.visible {
        return Ok(());
    }

    let world = &data.scene.world_runtime.ctx.world;
    let fixed_dt = data
        .scene
        .world_runtime
        .ctx
        .fixed_step_seconds
        .max(1.0 / 240.0);
    let interp_alpha =
        (data.scene.world_runtime.ctx.fixed_step_accumulator / fixed_dt).clamp(0.0, 1.0);
    let entities: Vec<_> = world.entities_with::<Position>().collect();
    for entity in entities.into_iter().take(MAX_WORLD_RENDER_AGENTS) {
        let Some(position) = world.get_component::<Position>(entity).copied() else {
            continue;
        };
        let prev = world
            .get_component::<PreviousPosition>(entity)
            .copied()
            .unwrap_or(PreviousPosition {
                x: position.x,
                y: position.y,
            });
        let render_x = prev.x + (position.x - prev.x) * interp_alpha;
        let render_y = prev.y + (position.y - prev.y) * interp_alpha;
        let health = world
            .get_component::<Health>(entity)
            .copied()
            .unwrap_or(Health { current: 1, max: 1 });
        let health_ratio = if health.max > 0 {
            (health.current as f32 / health.max as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let team_id = if world.get_component::<PlayerTag>(entity).is_some() {
            0
        } else if world.get_component::<EnemyTag>(entity).is_some() {
            1
        } else {
            1
        };
        frame.agents.push(WorldRenderAgent {
            x: render_x,
            y: render_y,
            radius: 0.95,
            health_ratio,
            team: team_id,
        });
    }

    if data.scene.world_runtime.ctx.gameplay_config.infinite_world {
        let center = frame
            .agents
            .iter()
            .find(|agent| agent.team == 0)
            .or_else(|| frame.agents.first())
            .map(|a| (a.x, a.y))
            .unwrap_or((0.0, 0.0));
        let chunk = data
            .scene
            .world_runtime
            .ctx
            .gameplay_config
            .chunk_size
            .max(4.0);
        let radius_chunks = data
            .scene
            .world_runtime
            .ctx
            .gameplay_config
            .chunk_load_radius
            .max(2) as f32;
        let extent = chunk * (radius_chunks + 2.0);
        frame.world_bounds = [
            center.0 - extent,
            center.1 - extent,
            center.0 + extent,
            center.1 + extent,
        ];
    }

    Ok(())
}
