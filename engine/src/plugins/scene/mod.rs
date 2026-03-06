pub mod domain;
pub mod manifest;

use self::domain::{
    GAMEPLAY_CONFIG_PATH, OverlaySceneRuntime, QuestState, SceneChannels, SceneCommand, SceneId,
    SceneLayer, SceneLifecycleEvent, SceneLifecyclePhase, SceneRegistry, SceneSlot,
    SceneTransitionResult, WorldSceneRuntime, WorldToOverlayMessage, build_overlay_runtime,
    build_world_scene_runtime, gameplay_config_modified,
    load_gameplay_config_with_modified_and_error,
};
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::input::domain::InputState;
use crate::plugins::shared::{ReloadStatusPayload, should_reload};
use crate::plugins::time::domain::Time;
use crate::plugins::ui::domain::UiDirty;
use crate::runtime::{
    CoreSet, FixedTimeConfig, FixedUpdate, PreUpdate, Res, ResMut, Startup, SystemConfigExt,
    Update, WindowState,
};
use crate::state::{GameplayRuntimeConfig, SceneRuntimeState, SessionRuntimeState, UiOverlayState};
use anyhow::{Result, anyhow};
use engine_replay::{ReplayArchive, ReplayJournalFrame, ReplayValidationReport};
use engine_sim::{SimulationCodec, SimulationHash, SimulationTick};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct ScenePlugin;

#[derive(Default)]
pub(crate) struct SceneResource {
    pub(crate) manager: Option<SceneManager>,
}

pub(crate) struct SceneManager {
    pub(crate) world: SceneSlot,
    pub(crate) world_runtime: WorldSceneRuntime,
    pub(crate) overlay_runtime: OverlaySceneRuntime,
    pub(crate) registry: SceneRegistry,
    pub(crate) overlay_back_stack: Vec<(SceneSlot, OverlaySceneRuntime)>,
    pub(crate) channels: SceneChannels,
    pub(crate) overlays: Vec<SceneSlot>,
    pub(crate) pending: Vec<SceneCommand>,
}

include!("internal/manager_impl.rs");

include!("internal/simulation_codec.rs");

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<GameplayRuntimeConfig>();
        app.init_resource::<SessionRuntimeState>();
        app.init_resource::<UiOverlayState>();
        app.add_systems(Startup, scene_setup_system);
        app.add_systems(PreUpdate, scene_transition_system.in_set(CoreSet::Scene));
        app.add_systems(
            FixedUpdate,
            world_scene_update_system.in_set(CoreSet::Scene),
        );
        app.add_systems(Update, scene_overlay_update_system.in_set(CoreSet::Scene));
    }
}

pub(crate) fn capture_scene_simulation_snapshot(
    manager: &SceneManager,
) -> Result<SceneSimulationSnapshotV1> {
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

    Ok(SceneSimulationSnapshotV1 {
        context: SceneWorldContextSnapshotV1 {
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
            session_admitted: ctx.session_admitted,
            session_lobby_id: ctx.session_lobby_id.clone(),
            session_roster_player_codes: ctx.session_roster_player_codes.clone(),
            session_max_players: ctx.session_max_players,
            session_ai_fill_target: ctx.session_ai_fill_target,
            session_settings_json: ctx.session_settings_json.clone(),
        },
        entities: SceneEntitySnapshotV1 {
            frame_counter,
            debug_position,
            debug_velocity,
        },
    })
}

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

pub(crate) fn build_scene_simulation_delta(
    base: &SceneSimulationSnapshotV1,
    current: &SceneSimulationSnapshotV1,
) -> SceneSimulationDeltaV1 {
    let base_ctx = &base.context;
    let current_ctx = &current.context;
    let base_entities = &base.entities;
    let current_entities = &current.entities;

    SceneSimulationDeltaV1 {
        context: SceneWorldContextDeltaV1 {
            world: (base_ctx.world != current_ctx.world).then_some(current_ctx.world),
            overlays: (base_ctx.overlays != current_ctx.overlays)
                .then_some(current_ctx.overlays.clone()),
            world_scene_label: (base_ctx.world_scene_label != current_ctx.world_scene_label)
                .then_some(current_ctx.world_scene_label.clone()),
            overlay_scene_label: (base_ctx.overlay_scene_label != current_ctx.overlay_scene_label)
                .then_some(current_ctx.overlay_scene_label.clone()),
            gameplay_config: (base_ctx.gameplay_config != current_ctx.gameplay_config)
                .then_some(current_ctx.gameplay_config.clone()),
            gameplay_config_modified_millis: (base_ctx.gameplay_config_modified_millis
                != current_ctx.gameplay_config_modified_millis)
                .then_some(current_ctx.gameplay_config_modified_millis),
            gameplay_config_revision: (base_ctx.gameplay_config_revision
                != current_ctx.gameplay_config_revision)
                .then_some(current_ctx.gameplay_config_revision),
            overlay_consumed: (base_ctx.overlay_consumed != current_ctx.overlay_consumed)
                .then_some(current_ctx.overlay_consumed),
            player_move_x: (base_ctx.player_move_x != current_ctx.player_move_x)
                .then_some(current_ctx.player_move_x),
            player_move_y: (base_ctx.player_move_y != current_ctx.player_move_y)
                .then_some(current_ctx.player_move_y),
            camera_yaw: (base_ctx.camera_yaw != current_ctx.camera_yaw)
                .then_some(current_ctx.camera_yaw),
            camera_pitch: (base_ctx.camera_pitch != current_ctx.camera_pitch)
                .then_some(current_ctx.camera_pitch),
            camera_distance: (base_ctx.camera_distance != current_ctx.camera_distance)
                .then_some(current_ctx.camera_distance),
            delta_seconds: (base_ctx.delta_seconds != current_ctx.delta_seconds)
                .then_some(current_ctx.delta_seconds),
            fixed_step_seconds: (base_ctx.fixed_step_seconds != current_ctx.fixed_step_seconds)
                .then_some(current_ctx.fixed_step_seconds),
            fixed_step_accumulator: (base_ctx.fixed_step_accumulator
                != current_ctx.fixed_step_accumulator)
                .then_some(current_ctx.fixed_step_accumulator),
            frame_count: (base_ctx.frame_count != current_ctx.frame_count)
                .then_some(current_ctx.frame_count),
            enemy_kills: (base_ctx.enemy_kills != current_ctx.enemy_kills)
                .then_some(current_ctx.enemy_kills),
            session_admitted: (base_ctx.session_admitted != current_ctx.session_admitted)
                .then_some(current_ctx.session_admitted),
            session_lobby_id: (base_ctx.session_lobby_id != current_ctx.session_lobby_id)
                .then_some(current_ctx.session_lobby_id.clone()),
            session_roster_player_codes: (base_ctx.session_roster_player_codes
                != current_ctx.session_roster_player_codes)
                .then_some(current_ctx.session_roster_player_codes.clone()),
            session_max_players: (base_ctx.session_max_players != current_ctx.session_max_players)
                .then_some(current_ctx.session_max_players),
            session_ai_fill_target: (base_ctx.session_ai_fill_target
                != current_ctx.session_ai_fill_target)
                .then_some(current_ctx.session_ai_fill_target),
            session_settings_json: (base_ctx.session_settings_json
                != current_ctx.session_settings_json)
                .then_some(current_ctx.session_settings_json.clone()),
        },
        entities: SceneEntityDeltaV1 {
            frame_counter: (base_entities.frame_counter != current_entities.frame_counter)
                .then_some(current_entities.frame_counter),
            debug_position: (base_entities.debug_position != current_entities.debug_position)
                .then_some(current_entities.debug_position),
            debug_velocity: (base_entities.debug_velocity != current_entities.debug_velocity)
                .then_some(current_entities.debug_velocity),
        },
    }
}

pub(crate) fn apply_scene_simulation_delta(
    base: &SceneSimulationSnapshotV1,
    delta: &SceneSimulationDeltaV1,
) -> SceneSimulationSnapshotV1 {
    SceneSimulationSnapshotV1 {
        context: SceneWorldContextSnapshotV1 {
            world: delta.context.world.unwrap_or(base.context.world),
            overlays: delta
                .context
                .overlays
                .clone()
                .unwrap_or_else(|| base.context.overlays.clone()),
            world_scene_label: delta
                .context
                .world_scene_label
                .clone()
                .unwrap_or_else(|| base.context.world_scene_label.clone()),
            overlay_scene_label: delta
                .context
                .overlay_scene_label
                .clone()
                .unwrap_or_else(|| base.context.overlay_scene_label.clone()),
            gameplay_config: delta
                .context
                .gameplay_config
                .clone()
                .unwrap_or_else(|| base.context.gameplay_config.clone()),
            gameplay_config_modified_millis: delta
                .context
                .gameplay_config_modified_millis
                .unwrap_or(base.context.gameplay_config_modified_millis),
            gameplay_config_revision: delta
                .context
                .gameplay_config_revision
                .unwrap_or(base.context.gameplay_config_revision),
            overlay_consumed: delta
                .context
                .overlay_consumed
                .unwrap_or(base.context.overlay_consumed),
            player_move_x: delta
                .context
                .player_move_x
                .unwrap_or(base.context.player_move_x),
            player_move_y: delta
                .context
                .player_move_y
                .unwrap_or(base.context.player_move_y),
            camera_yaw: delta.context.camera_yaw.unwrap_or(base.context.camera_yaw),
            camera_pitch: delta
                .context
                .camera_pitch
                .unwrap_or(base.context.camera_pitch),
            camera_distance: delta
                .context
                .camera_distance
                .unwrap_or(base.context.camera_distance),
            delta_seconds: delta
                .context
                .delta_seconds
                .unwrap_or(base.context.delta_seconds),
            fixed_step_seconds: delta
                .context
                .fixed_step_seconds
                .unwrap_or(base.context.fixed_step_seconds),
            fixed_step_accumulator: delta
                .context
                .fixed_step_accumulator
                .unwrap_or(base.context.fixed_step_accumulator),
            frame_count: delta
                .context
                .frame_count
                .unwrap_or(base.context.frame_count),
            enemy_kills: delta
                .context
                .enemy_kills
                .unwrap_or(base.context.enemy_kills),
            session_admitted: delta
                .context
                .session_admitted
                .unwrap_or(base.context.session_admitted),
            session_lobby_id: delta
                .context
                .session_lobby_id
                .clone()
                .unwrap_or_else(|| base.context.session_lobby_id.clone()),
            session_roster_player_codes: delta
                .context
                .session_roster_player_codes
                .clone()
                .unwrap_or_else(|| base.context.session_roster_player_codes.clone()),
            session_max_players: delta
                .context
                .session_max_players
                .unwrap_or(base.context.session_max_players),
            session_ai_fill_target: delta
                .context
                .session_ai_fill_target
                .unwrap_or(base.context.session_ai_fill_target),
            session_settings_json: delta
                .context
                .session_settings_json
                .clone()
                .unwrap_or_else(|| base.context.session_settings_json.clone()),
        },
        entities: SceneEntitySnapshotV1 {
            frame_counter: delta
                .entities
                .frame_counter
                .unwrap_or(base.entities.frame_counter),
            debug_position: delta
                .entities
                .debug_position
                .unwrap_or(base.entities.debug_position),
            debug_velocity: delta
                .entities
                .debug_velocity
                .unwrap_or(base.entities.debug_velocity),
        },
    }
}

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

pub(crate) fn replay_scene_frame(
    manager: &mut SceneManager,
    frame: &SceneReplayCommandFrame,
) -> Result<SimulationHash> {
    if manager.world != frame.world {
        manager.world = frame.world;
        manager.world_runtime = build_world_scene_runtime(frame.world.active)?;
        manager.channels = SceneChannels::default();
        manager.pending.clear();
    }
    if manager.overlays != frame.overlays {
        rebuild_overlay_stack(manager, &frame.overlays)?;
    }

    manager.world = frame.world;
    let ctx = &mut manager.world_runtime.ctx;
    ctx.world_scene_label = frame.world_scene_label.clone();
    ctx.overlay_scene_label = frame.overlay_scene_label.clone();
    ctx.gameplay_config = frame.gameplay_config.clone();
    ctx.gameplay_config_revision = frame.gameplay_config_revision;
    ctx.overlay_consumed = frame.overlay_consumed;
    ctx.player_move_x = frame.player_move_x;
    ctx.player_move_y = frame.player_move_y;
    ctx.camera_yaw = frame.camera_yaw;
    ctx.camera_pitch = frame.camera_pitch;
    ctx.camera_distance = frame.camera_distance;
    ctx.delta_seconds = frame.delta_seconds;
    ctx.fixed_step_seconds = frame.fixed_step_seconds;
    ctx.session_admitted = frame.session_admitted;
    ctx.session_lobby_id = frame.session_lobby_id.clone();
    ctx.session_roster_player_codes = frame.session_roster_player_codes.clone();
    ctx.session_max_players = frame.session_max_players;
    ctx.session_ai_fill_target = frame.session_ai_fill_target;
    ctx.session_settings_json = frame.session_settings_json.clone();
    ctx.outbound_notifications.clear();

    if manager.world.visible && !manager.world.paused {
        manager
            .world_runtime
            .scheduler
            .run(&mut manager.world_runtime.ctx)?;
        let outbound = std::mem::take(&mut manager.world_runtime.ctx.outbound_notifications);
        manager.channels.world_to_overlay.extend(outbound);
    }

    let snapshot = capture_scene_simulation_snapshot(manager)?;
    SceneSimulationCodec::hash(&snapshot)
}

pub(crate) fn republish_scene_resources(world: &mut ecs::World) -> Result<()> {
    let Some((scene_state_value, gameplay_value, overlay_value, session_value)) = world
        .resource::<SceneResource>()
        .ok()
        .and_then(|scene_resource| scene_resource.manager.as_ref())
        .map(snapshot_public_scene_state)
    else {
        return Ok(());
    };

    if let Ok(mut scene_state) = world.resource_mut::<SceneRuntimeState>() {
        *scene_state = scene_state_value;
    }
    if let Ok(mut gameplay) = world.resource_mut::<GameplayRuntimeConfig>() {
        *gameplay = gameplay_value;
    }
    if let Ok(mut overlay) = world.resource_mut::<UiOverlayState>() {
        *overlay = overlay_value;
    }
    if let Ok(mut session) = world.resource_mut::<SessionRuntimeState>() {
        *session = session_value;
    }
    Ok(())
}

pub(crate) fn validate_scene_replay(
    world: &mut ecs::World,
    archive: &SceneReplayArchive,
    target_tick: SimulationTick,
) -> Result<ReplayValidationReport> {
    let checkpoint = archive
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.meta.tick.0 <= target_tick.0)
        .max_by_key(|checkpoint| checkpoint.meta.tick.0)
        .cloned()
        .ok_or_else(|| anyhow!("no replay checkpoint available for tick {}", target_tick.0))?;
    let frames: Vec<ReplayJournalFrame<SceneReplayCommandFrame>> = archive
        .journal
        .iter()
        .filter(|frame| frame.tick.0 > checkpoint.meta.tick.0 && frame.tick.0 <= target_tick.0)
        .cloned()
        .collect();

    {
        let window = world.resource::<WindowState>().ok().cloned();
        let mut scene_resource = world
            .resource_mut::<SceneResource>()
            .map_err(|_| anyhow!("ScenePlugin resource is not available"))?;
        if scene_resource.manager.is_none() {
            let window = window.ok_or_else(|| anyhow!("WindowState is not available"))?;
            scene_resource.manager = Some(SceneManager::new(&window)?);
        }
        let manager = scene_resource
            .manager
            .as_mut()
            .ok_or_else(|| anyhow!("scene manager is not initialized"))?;
        restore_scene_simulation_snapshot(manager, &checkpoint.snapshot)?;
        let mut report = ReplayValidationReport::default();
        for frame in &frames {
            let command = frame
                .commands
                .first()
                .ok_or_else(|| anyhow!("replay journal frame {} has no commands", frame.tick.0))?;
            let actual = replay_scene_frame(manager, command)?;
            if let Some(expected) = frame.post_hash
                && expected != actual
            {
                report
                    .mismatches
                    .push(engine_replay::ReplayMismatch::TickHashMismatch {
                        tick: frame.tick,
                        expected,
                        actual,
                    });
            }
        }
        apply_overlay_messages(manager);
        drop(scene_resource);
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            *tick = target_tick;
        }
        republish_scene_resources(world)?;
        return Ok(report);
    }
}

include!("internal/runtime_helpers.rs");

#[cfg(test)]
mod tests {
    include!("internal/tests.rs");
}
