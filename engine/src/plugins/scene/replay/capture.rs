use super::super::SceneManager;
use super::codec::SceneReplayCommandFrame;
use crate::runtime::SimulationTick;

pub(crate) fn capture_scene_replay_command_frame(
    manager: &SceneManager,
    tick: SimulationTick,
) -> SceneReplayCommandFrame {
    let ctx = &manager.world_runtime.ctx;
    SceneReplayCommandFrame {
        tick,
        world: manager.world,
        overlays: manager.overlays.clone(),
        world_scene_label: ctx.world_scene_label.clone(),
        overlay_scene_label: ctx.overlay_scene_label.clone(),
        gameplay_config: ctx.gameplay_config.clone(),
        gameplay_config_revision: ctx.gameplay_config_revision,
        overlay_consumed: ctx.overlay_consumed,
        player_move_x: ctx.player_move_x,
        player_move_y: ctx.player_move_y,
        camera_yaw: ctx.camera_yaw,
        camera_pitch: ctx.camera_pitch,
        camera_distance: ctx.camera_distance,
        delta_seconds: ctx.delta_seconds,
        fixed_step_seconds: ctx.fixed_step_seconds,
        session_admitted: ctx.session_admitted,
        session_lobby_id: ctx.session_lobby_id.clone(),
        session_roster_player_codes: ctx.session_roster_player_codes.clone(),
        session_max_players: ctx.session_max_players,
        session_ai_fill_target: ctx.session_ai_fill_target,
        session_settings_json: ctx.session_settings_json.clone(),
    }
}
