use super::super::domain::{self, SceneChannels, build_world_scene_runtime};
use super::super::runtime::{millis_to_system_time, rebuild_overlay_stack};
use super::super::{SceneManager, SceneSimulationSnapshotV1};
use anyhow::Result;

pub(crate) fn restore_scene_simulation_snapshot(
    manager: &mut SceneManager,
    snapshot: &SceneSimulationSnapshotV1,
) -> Result<()> {
    manager.world = snapshot.context.world;
    manager.world_runtime = build_world_scene_runtime(snapshot.context.world.active)?;
    rebuild_overlay_stack(manager, &snapshot.context.overlays)?;
    manager.channels = SceneChannels::default();
    manager.pending.clear();

    let ctx = &mut manager.world_runtime.ctx;
    ctx.world_scene_label = snapshot.context.world_scene_label.clone();
    ctx.overlay_scene_label = snapshot.context.overlay_scene_label.clone();
    ctx.gameplay_config = snapshot.context.gameplay_config.clone();
    ctx.gameplay_config_modified =
        millis_to_system_time(snapshot.context.gameplay_config_modified_millis);
    ctx.gameplay_config_revision = snapshot.context.gameplay_config_revision;
    ctx.overlay_consumed = snapshot.context.overlay_consumed;
    ctx.player_move_x = snapshot.context.player_move_x;
    ctx.player_move_y = snapshot.context.player_move_y;
    ctx.camera_yaw = snapshot.context.camera_yaw;
    ctx.camera_pitch = snapshot.context.camera_pitch;
    ctx.camera_distance = snapshot.context.camera_distance;
    ctx.delta_seconds = snapshot.context.delta_seconds;
    ctx.fixed_step_seconds = snapshot.context.fixed_step_seconds;
    ctx.fixed_step_accumulator = snapshot.context.fixed_step_accumulator;
    ctx.frame_count = snapshot.context.frame_count;
    ctx.enemy_kills = snapshot.context.enemy_kills;
    ctx.session_admitted = snapshot.context.session_admitted;
    ctx.session_lobby_id = snapshot.context.session_lobby_id.clone();
    ctx.session_roster_player_codes = snapshot.context.session_roster_player_codes.clone();
    ctx.session_max_players = snapshot.context.session_max_players;
    ctx.session_ai_fill_target = snapshot.context.session_ai_fill_target;
    ctx.session_settings_json = snapshot.context.session_settings_json.clone();
    ctx.outbound_notifications.clear();

    if let Ok(mut entity) = ctx.world.entity_mut(ctx.tick_entity)
        && let Some(mut counter) = entity.get_mut::<domain::WorldFrameCounter>()
    {
        *counter = snapshot.entities.frame_counter;
    }
    if let Ok(mut entity) = ctx.world.entity_mut(ctx.debug_entity) {
        if let Some(mut position) = entity.get_mut::<domain::WorldDebugPosition>() {
            *position = snapshot.entities.debug_position;
        }
        if let Some(mut velocity) = entity.get_mut::<domain::WorldDebugVelocity>() {
            *velocity = snapshot.entities.debug_velocity;
        }
    }

    Ok(())
}
