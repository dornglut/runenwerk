use super::super::domain;
use super::super::runtime::system_time_to_millis;
use super::super::{
    SceneEntitySnapshotV2, SceneManager, SceneSimulationSnapshotV2, SceneWorldContextSnapshotV2,
};
use anyhow::Result;

pub(crate) fn capture_scene_simulation_snapshot(
    manager: &SceneManager,
) -> Result<SceneSimulationSnapshotV2> {
    let ctx = &manager.world_runtime.ctx;
    let frame_counter = ctx
        .world
        .get::<domain::WorldFrameCounter>(ctx.tick_entity)
        .copied()
        .unwrap_or(domain::WorldFrameCounter {
            value: ctx.frame_count,
        });
    let debug_position = ctx
        .world
        .get::<domain::WorldDebugPosition>(ctx.debug_entity)
        .copied()
        .unwrap_or(domain::WorldDebugPosition { x: 0.0, y: 0.0 });
    let debug_velocity = ctx
        .world
        .get::<domain::WorldDebugVelocity>(ctx.debug_entity)
        .copied()
        .unwrap_or(domain::WorldDebugVelocity { x: 0.0, y: 0.0 });

    Ok(SceneSimulationSnapshotV2 {
        context: SceneWorldContextSnapshotV2 {
            world: manager.world,
            overlays: manager.overlays.clone(),
            world_scene_label: ctx.world_scene_label.clone(),
            overlay_scene_label: ctx.overlay_scene_label.clone(),
            gameplay_config: ctx.gameplay_config.clone(),
            gameplay_config_modified_millis: system_time_to_millis(ctx.gameplay_config_modified),
            gameplay_config_revision: ctx.gameplay_config_revision,
            overlay_consumed: ctx.overlay_consumed,
            player_move_x: ctx.player_move_x,
            player_move_y: ctx.player_move_y,
            camera_yaw: ctx.camera_yaw,
            camera_pitch: ctx.camera_pitch,
            camera_distance: ctx.camera_distance,
            delta_seconds: ctx.delta_seconds,
            fixed_step_seconds: ctx.fixed_step_seconds,
            fixed_step_accumulator: ctx.fixed_step_accumulator,
            frame_count: ctx.frame_count,
            enemy_kills: ctx.enemy_kills,
        },
        entities: SceneEntitySnapshotV2 {
            frame_counter,
            debug_position,
            debug_velocity,
        },
    })
}
