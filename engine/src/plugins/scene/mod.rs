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

impl SceneManager {
    pub(crate) fn new(window: &WindowState) -> Result<Self> {
        let screen_size = (window.size_px.0 as f32, window.size_px.1 as f32);
        let scale = window.scale_factor as f32;
        let registry = SceneRegistry::load();
        let world_scene = SceneId::GameplayStub;
        let mut manager = Self {
            world: SceneSlot::new(world_scene),
            world_runtime: build_world_scene_runtime(world_scene)?,
            overlay_runtime: build_overlay_runtime(
                SceneId::ConsoleUi,
                screen_size,
                scale,
                &registry,
            )?,
            registry,
            overlay_back_stack: Vec::new(),
            channels: SceneChannels::default(),
            overlays: vec![SceneSlot {
                active: SceneId::ConsoleUi,
                paused: false,
                visible: false,
            }],
            pending: Vec::new(),
        };
        manager.emit_lifecycle(world_scene, SceneLifecyclePhase::Enter);
        manager.emit_lifecycle(SceneId::ConsoleUi, SceneLifecyclePhase::Enter);
        Ok(manager)
    }

    fn emit_lifecycle(&mut self, scene: SceneId, phase: SceneLifecyclePhase) {
        self.channels.lifecycle_events.push(SceneLifecycleEvent {
            scene,
            layer: scene.layer(),
            phase,
        });
    }

    pub(crate) fn active_overlay(&self) -> SceneId {
        self.overlays
            .last()
            .map(|slot| slot.active)
            .unwrap_or(SceneId::ConsoleUi)
    }

    pub(crate) fn overlay_visible(&self) -> bool {
        self.overlays
            .last()
            .map(|slot| slot.visible)
            .unwrap_or(false)
    }

    pub(crate) fn set_active_overlay_visible(&mut self, visible: bool) {
        if let Some(slot) = self.overlays.last_mut() {
            slot.visible = visible;
        }
    }

    pub(crate) fn queue(&mut self, command: SceneCommand) {
        self.pending.push(command);
    }

    fn overlay_viewport(&self) -> ((f32, f32), f32) {
        (
            self.overlay_runtime.ui.screen_size,
            self.overlay_runtime.ui.scale,
        )
    }

    pub(crate) fn set_overlay_viewport(&mut self, screen_size: (f32, f32), scale: f32) {
        self.overlay_runtime.ui.screen_size = screen_size;
        self.overlay_runtime.ui.scale = scale;
        self.overlay_runtime.ui.layout_dirty = true;
        for (_, runtime) in &mut self.overlay_back_stack {
            runtime.ui.screen_size = screen_size;
            runtime.ui.scale = scale;
            runtime.ui.layout_dirty = true;
        }
    }

    pub(crate) fn apply_pending(&mut self) -> Result<SceneTransitionResult> {
        let mut result = SceneTransitionResult::default();
        let pending = std::mem::take(&mut self.pending);
        for command in pending {
            let command = match command {
                SceneCommand::ReplaceWorldByLabel(label) => {
                    let Some(scene) = SceneId::from_label(&label) else {
                        tracing::warn!(scene_label = %label, "unknown world scene label");
                        continue;
                    };
                    SceneCommand::ReplaceWorld(scene)
                }
                SceneCommand::ReplaceOverlayByLabel(label) => {
                    let Some(scene) = SceneId::from_label(&label) else {
                        tracing::warn!(scene_label = %label, "unknown overlay scene label");
                        continue;
                    };
                    SceneCommand::ReplaceOverlay(scene)
                }
                SceneCommand::PushOverlayByLabel(label) => {
                    let Some(scene) = SceneId::from_label(&label) else {
                        tracing::warn!(scene_label = %label, "unknown overlay scene label");
                        continue;
                    };
                    SceneCommand::PushOverlay(scene)
                }
                other => other,
            };
            match command {
                SceneCommand::ReplaceWorld(scene) => {
                    if scene.layer() != SceneLayer::World {
                        continue;
                    }
                    let before = self.world.active;
                    if before != scene {
                        self.emit_lifecycle(before, SceneLifecyclePhase::Exit);
                        self.world_runtime = build_world_scene_runtime(scene)?;
                        self.world.active = scene;
                        self.emit_lifecycle(scene, SceneLifecyclePhase::Enter);
                        result.world_changed = true;
                    }
                }
                SceneCommand::ReplaceOverlay(scene) => {
                    if scene.layer() != SceneLayer::OverlayUi {
                        continue;
                    }
                    let before = self.active_overlay();
                    if before != scene {
                        self.emit_lifecycle(before, SceneLifecyclePhase::Exit);
                        let (screen_size, scale) = self.overlay_viewport();
                        self.overlay_runtime =
                            build_overlay_runtime(scene, screen_size, scale, &self.registry)?;
                        if let Some(slot) = self.overlays.last_mut() {
                            slot.active = scene;
                        } else {
                            self.overlays.push(SceneSlot::new(scene));
                        }
                        self.emit_lifecycle(scene, SceneLifecyclePhase::Enter);
                    }
                    result.overlay_changed |= before != self.active_overlay();
                }
                SceneCommand::PushOverlay(scene) => {
                    if scene.layer() != SceneLayer::OverlayUi {
                        continue;
                    }
                    let current_slot = self
                        .overlays
                        .last()
                        .copied()
                        .unwrap_or_else(|| SceneSlot::new(self.active_overlay()));
                    let (screen_size, scale) = self.overlay_viewport();
                    let next_runtime =
                        build_overlay_runtime(scene, screen_size, scale, &self.registry)?;
                    let previous_runtime =
                        std::mem::replace(&mut self.overlay_runtime, next_runtime);
                    self.overlay_back_stack
                        .push((current_slot, previous_runtime));
                    self.emit_lifecycle(current_slot.active, SceneLifecyclePhase::Pause);
                    self.overlays.push(SceneSlot::new(scene));
                    self.emit_lifecycle(scene, SceneLifecyclePhase::Enter);
                    result.overlay_changed = true;
                }
                SceneCommand::PopOverlay => {
                    if self.overlays.len() > 1
                        && let Some((restored_slot, restored_runtime)) =
                            self.overlay_back_stack.pop()
                    {
                        if let Some(popped_slot) = self.overlays.last().copied() {
                            self.emit_lifecycle(popped_slot.active, SceneLifecyclePhase::Exit);
                        }
                        self.overlays.pop();
                        self.overlay_runtime = restored_runtime;
                        if let Some(last) = self.overlays.last_mut() {
                            *last = restored_slot;
                        } else {
                            self.overlays.push(restored_slot);
                        }
                        self.emit_lifecycle(restored_slot.active, SceneLifecyclePhase::Resume);
                        result.overlay_changed = true;
                    }
                }
                SceneCommand::PauseWorld(paused) => {
                    if self.world.paused != paused {
                        self.world.paused = paused;
                        self.emit_lifecycle(
                            self.world.active,
                            if paused {
                                SceneLifecyclePhase::Pause
                            } else {
                                SceneLifecyclePhase::Resume
                            },
                        );
                        result.world_pause_changed = true;
                    }
                }
                SceneCommand::ReplaceWorldByLabel(_)
                | SceneCommand::ReplaceOverlayByLabel(_)
                | SceneCommand::PushOverlayByLabel(_) => {}
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEntitySnapshotV1 {
    pub frame_counter: domain::WorldFrameCounter,
    pub debug_position: domain::WorldDebugPosition,
    pub debug_velocity: domain::WorldDebugVelocity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneWorldContextSnapshotV1 {
    pub world: SceneSlot,
    pub overlays: Vec<SceneSlot>,
    pub world_scene_label: String,
    pub overlay_scene_label: String,
    pub gameplay_config: domain::GameplayConfig,
    pub gameplay_config_modified_millis: Option<u64>,
    pub gameplay_config_revision: u64,
    pub overlay_consumed: bool,
    pub player_move_x: f32,
    pub player_move_y: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub delta_seconds: f32,
    pub fixed_step_seconds: f32,
    pub fixed_step_accumulator: f32,
    pub frame_count: u64,
    pub enemy_kills: u32,
    pub session_admitted: bool,
    pub session_lobby_id: Option<String>,
    pub session_roster_player_codes: Vec<String>,
    pub session_max_players: u8,
    pub session_ai_fill_target: u8,
    pub session_settings_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSimulationSnapshotV1 {
    pub context: SceneWorldContextSnapshotV1,
    pub entities: SceneEntitySnapshotV1,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneEntityDeltaV1 {
    pub frame_counter: Option<domain::WorldFrameCounter>,
    pub debug_position: Option<domain::WorldDebugPosition>,
    pub debug_velocity: Option<domain::WorldDebugVelocity>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneWorldContextDeltaV1 {
    pub world: Option<SceneSlot>,
    pub overlays: Option<Vec<SceneSlot>>,
    pub world_scene_label: Option<String>,
    pub overlay_scene_label: Option<String>,
    pub gameplay_config: Option<domain::GameplayConfig>,
    pub gameplay_config_modified_millis: Option<Option<u64>>,
    pub gameplay_config_revision: Option<u64>,
    pub overlay_consumed: Option<bool>,
    pub player_move_x: Option<f32>,
    pub player_move_y: Option<f32>,
    pub camera_yaw: Option<f32>,
    pub camera_pitch: Option<f32>,
    pub camera_distance: Option<f32>,
    pub delta_seconds: Option<f32>,
    pub fixed_step_seconds: Option<f32>,
    pub fixed_step_accumulator: Option<f32>,
    pub frame_count: Option<u64>,
    pub enemy_kills: Option<u32>,
    pub session_admitted: Option<bool>,
    pub session_lobby_id: Option<Option<String>>,
    pub session_roster_player_codes: Option<Vec<String>>,
    pub session_max_players: Option<u8>,
    pub session_ai_fill_target: Option<u8>,
    pub session_settings_json: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneSimulationDeltaV1 {
    pub context: SceneWorldContextDeltaV1,
    pub entities: SceneEntityDeltaV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneReplayCommandFrame {
    pub tick: SimulationTick,
    pub world: SceneSlot,
    pub overlays: Vec<SceneSlot>,
    pub world_scene_label: String,
    pub overlay_scene_label: String,
    pub gameplay_config: domain::GameplayConfig,
    pub gameplay_config_revision: u64,
    pub overlay_consumed: bool,
    pub player_move_x: f32,
    pub player_move_y: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub delta_seconds: f32,
    pub fixed_step_seconds: f32,
    pub session_admitted: bool,
    pub session_lobby_id: Option<String>,
    pub session_roster_player_codes: Vec<String>,
    pub session_max_players: u8,
    pub session_ai_fill_target: u8,
    pub session_settings_json: Option<String>,
}

pub type SceneReplayArchive = ReplayArchive<SceneSimulationSnapshotV1, SceneReplayCommandFrame>;

pub(crate) struct SceneSimulationCodec;

impl SimulationCodec for SceneSimulationCodec {
    type Host = SceneManager;
    type Snapshot = SceneSimulationSnapshotV1;

    fn codec_id() -> &'static str {
        "scene_runtime_v1"
    }

    fn capture(host: &Self::Host) -> Result<Self::Snapshot> {
        capture_scene_simulation_snapshot(host)
    }

    fn restore(host: &mut Self::Host, snapshot: &Self::Snapshot) -> Result<()> {
        restore_scene_simulation_snapshot(host, snapshot)
    }
}

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

fn rebuild_overlay_stack(manager: &mut SceneManager, slots: &[SceneSlot]) -> Result<()> {
    let slots = if slots.is_empty() {
        vec![SceneSlot {
            active: SceneId::ConsoleUi,
            paused: false,
            visible: false,
        }]
    } else {
        slots.to_vec()
    };
    let screen_size = manager.overlay_runtime.ui.screen_size;
    let scale = manager.overlay_runtime.ui.scale;
    let last_index = slots.len().saturating_sub(1);
    let mut stack = Vec::new();
    for slot in slots.iter().take(last_index) {
        stack.push((
            *slot,
            build_overlay_runtime(slot.active, screen_size, scale, &manager.registry)?,
        ));
    }
    let active_slot = slots[last_index];
    manager.overlay_runtime =
        build_overlay_runtime(active_slot.active, screen_size, scale, &manager.registry)?;
    manager.overlay_back_stack = stack;
    manager.overlays = slots;
    Ok(())
}

fn snapshot_public_scene_state(
    manager: &SceneManager,
) -> (
    SceneRuntimeState,
    GameplayRuntimeConfig,
    UiOverlayState,
    SessionRuntimeState,
) {
    let gameplay = GameplayRuntimeConfig {
        chunk_size: manager.world_runtime.ctx.gameplay_config.chunk_size,
        chunk_load_radius: manager.world_runtime.ctx.gameplay_config.chunk_load_radius,
        infinite_world: manager.world_runtime.ctx.gameplay_config.infinite_world,
    };
    let scene_state = SceneRuntimeState {
        world_scene_label: manager.world.active.label().to_string(),
        overlay_scene_label: manager.active_overlay().label().to_string(),
        overlay_visible: manager.overlay_visible(),
        world_paused: manager.world.paused,
        enemy_kills: manager.world_runtime.ctx.enemy_kills,
        gameplay,
    };
    let overlay = UiOverlayState {
        screen_size: manager.overlay_runtime.ui.screen_size,
        scale: manager.overlay_runtime.ui.scale,
        ..UiOverlayState::default()
    };
    let session = SessionRuntimeState {
        admitted: manager.world_runtime.ctx.session_admitted,
        lobby_id: manager.world_runtime.ctx.session_lobby_id.clone(),
        roster_player_codes: manager
            .world_runtime
            .ctx
            .session_roster_player_codes
            .clone(),
        max_players: manager.world_runtime.ctx.session_max_players,
        ai_fill_target: manager.world_runtime.ctx.session_ai_fill_target,
        settings_json: manager.world_runtime.ctx.session_settings_json.clone(),
    };
    (scene_state, gameplay, overlay, session)
}

fn system_time_to_millis(value: Option<SystemTime>) -> Option<u64> {
    value.and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
    })
}

fn millis_to_system_time(value: Option<u64>) -> Option<SystemTime> {
    value.map(|millis| UNIX_EPOCH + Duration::from_millis(millis))
}

fn scene_setup_system(
    window: Res<WindowState>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    session: Res<SessionRuntimeState>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(SceneManager::new(&window)?);
    }
    if let Some(manager) = scene_resource.manager.as_mut() {
        sync_overlay_viewport(manager, &window);
        sync_world_scene_context_from_session(manager, &session);
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    }
    Ok(())
}

fn scene_transition_system(
    input: Res<InputState>,
    window: Res<WindowState>,
    time: Res<Time>,
    fixed_time: Res<FixedTimeConfig>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };

    sync_overlay_viewport(manager, &window);
    sync_world_scene_context_from_input(
        manager,
        &input,
        time.delta_seconds,
        fixed_time.step_seconds,
    );

    if input.toggle_pause_menu {
        let show_overlay = !manager.overlay_visible();
        manager.set_active_overlay_visible(show_overlay);
        manager.queue(SceneCommand::PauseWorld(show_overlay));
        if show_overlay && manager.active_overlay() != SceneId::HudUi {
            manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
        }
    }
    if input.scene_next {
        let next = manager.active_overlay().next_overlay();
        manager.queue(SceneCommand::ReplaceOverlay(next));
    }
    if input.scene_prev {
        let prev = manager.active_overlay().previous_overlay();
        manager.queue(SceneCommand::ReplaceOverlay(prev));
    }
    if input.scene_console {
        manager.queue(SceneCommand::ReplaceOverlay(SceneId::ConsoleUi));
    }
    if input.scene_hud {
        manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
    }
    if input.scene_overlay_push {
        let next = manager.active_overlay().next_overlay();
        manager.queue(SceneCommand::PushOverlay(next));
    }
    if input.scene_overlay_pop {
        manager.queue(SceneCommand::PopOverlay);
    }

    let result = manager.apply_pending()?;
    if result.world_changed {
        manager.overlay_runtime.ui.editor.status = format!(
            "editor: world scene switched to {}",
            manager.world.active.label()
        );
    }
    if result.overlay_changed {
        let active = manager.active_overlay();
        let path = manager
            .registry
            .ui_template_path(active)
            .unwrap_or("<none>");
        manager.overlay_runtime.ui.editor.status = format!(
            "editor: overlay scene switched to {} ({}) [stack={}]",
            active.label(),
            path,
            manager.overlays.len()
        );
    }
    if result.world_pause_changed {
        manager.overlay_runtime.ui.editor.status = if manager.world.paused {
            "editor: world scene paused".to_string()
        } else {
            "editor: world scene resumed".to_string()
        };
    }

    flush_lifecycle_status(manager);
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}

fn world_scene_update_system(
    fixed_time: Res<FixedTimeConfig>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    session: Res<SessionRuntimeState>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    sync_world_scene_context_from_session(manager, &session);
    if !manager.world.visible || manager.world.paused {
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
        return Ok(());
    }

    manager.world_runtime.ctx.delta_seconds =
        fixed_time.step_seconds.clamp(1.0 / 240.0, 1.0 / 30.0);
    manager.world_runtime.ctx.fixed_step_seconds = manager.world_runtime.ctx.delta_seconds;
    manager
        .world_runtime
        .scheduler
        .run(&mut manager.world_runtime.ctx)?;
    let outbound = std::mem::take(&mut manager.world_runtime.ctx.outbound_notifications);
    manager.channels.world_to_overlay.extend(outbound);
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}

fn scene_overlay_update_system(
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };

    apply_overlay_messages(manager);
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}

fn sync_overlay_viewport(manager: &mut SceneManager, window: &WindowState) {
    manager.set_overlay_viewport(
        (window.size_px.0 as f32, window.size_px.1 as f32),
        window.scale_factor as f32,
    );
}

fn sync_world_scene_context_from_input(
    manager: &mut SceneManager,
    input: &InputState,
    frame_delta_seconds: f32,
    fixed_step_seconds: f32,
) {
    let active_overlay_label = manager.active_overlay().label().to_string();
    let overlay_visible = manager.overlay_visible();
    let world_paused = manager.world.paused;
    let runtime = &mut manager.world_runtime;
    runtime.ctx.overlay_consumed = input.overlay_consumed;
    runtime.ctx.overlay_scene_label = active_overlay_label;
    runtime.ctx.player_move_x = (if input.world_move_right { 1.0 } else { 0.0 })
        - (if input.world_move_left { 1.0 } else { 0.0 });
    runtime.ctx.player_move_y = (if input.world_move_up { 1.0 } else { 0.0 })
        - (if input.world_move_down { 1.0 } else { 0.0 });
    runtime.ctx.fixed_step_seconds = fixed_step_seconds;

    if !overlay_visible && !world_paused {
        let camera_cfg = &runtime.ctx.gameplay_config.camera;
        let rotate_sensitivity = camera_cfg.rotate_sensitivity.max(0.0);
        let yaw_sign = if camera_cfg.invert_x { 1.0 } else { -1.0 };
        let pitch_sign = if camera_cfg.invert_y { -1.0 } else { 1.0 };
        runtime.ctx.camera_yaw += input.mouse_delta.0 * rotate_sensitivity * yaw_sign;
        runtime.ctx.camera_pitch += input.mouse_delta.1 * rotate_sensitivity * pitch_sign;
    }
    if !overlay_visible && input.scroll_delta.abs() > f32::EPSILON {
        let camera_cfg = &runtime.ctx.gameplay_config.camera;
        let zoom_sensitivity = camera_cfg.zoom_sensitivity.max(0.0);
        let zoom_sign = if camera_cfg.invert_zoom { 1.0 } else { -1.0 };
        runtime.ctx.camera_distance += input.scroll_delta * zoom_sensitivity * zoom_sign;
    }
    let camera_cfg = &runtime.ctx.gameplay_config.camera;
    let pitch_min = camera_cfg.pitch_min.min(camera_cfg.pitch_max);
    let pitch_max = camera_cfg.pitch_min.max(camera_cfg.pitch_max);
    let distance_min = camera_cfg
        .distance_min
        .min(camera_cfg.distance_max)
        .max(0.1);
    let distance_max = camera_cfg
        .distance_min
        .max(camera_cfg.distance_max)
        .max(distance_min);
    runtime.ctx.camera_pitch = runtime.ctx.camera_pitch.clamp(pitch_min, pitch_max);
    runtime.ctx.camera_distance = runtime
        .ctx
        .camera_distance
        .clamp(distance_min, distance_max);

    let latest_modified = gameplay_config_modified();
    if should_reload(
        true,
        false,
        runtime.ctx.gameplay_config_modified,
        latest_modified,
    ) {
        let (config, modified, error) = load_gameplay_config_with_modified_and_error();
        runtime.ctx.gameplay_config = config;
        runtime.ctx.gameplay_config_modified = modified;
        runtime.ctx.gameplay_config_revision =
            runtime.ctx.gameplay_config_revision.saturating_add(1);
        let payload = ReloadStatusPayload::new(
            "gameplay_config",
            manager.world.active.label(),
            if error.is_some() {
                "fallback"
            } else {
                "reloaded"
            },
            GAMEPLAY_CONFIG_PATH,
            runtime.ctx.gameplay_config_revision,
            true,
            modified,
            error,
            None,
        );
        manager
            .channels
            .overlay_console_lines
            .push(format!("[world] {}", payload.line()));
    }

    runtime.ctx.delta_seconds = frame_delta_seconds.max(0.0);
}

fn sync_world_scene_context_from_session(
    manager: &mut SceneManager,
    session: &SessionRuntimeState,
) {
    let runtime = &mut manager.world_runtime;
    runtime.ctx.session_admitted = session.admitted;
    runtime.ctx.session_lobby_id = session.lobby_id.clone();
    runtime.ctx.session_roster_player_codes = session.roster_player_codes.clone();
    runtime.ctx.session_max_players = session.max_players.max(1);
    runtime.ctx.session_ai_fill_target = session
        .ai_fill_target
        .clamp(1, runtime.ctx.session_max_players);
    runtime.ctx.session_settings_json = session.settings_json.clone();
}

fn publish_scene_state(
    manager: &SceneManager,
    scene_state: &mut SceneRuntimeState,
    gameplay: &mut GameplayRuntimeConfig,
    overlay: &mut UiOverlayState,
) {
    let (scene_state_value, gameplay_value, overlay_value, _) =
        snapshot_public_scene_state(manager);
    *scene_state = scene_state_value;
    *gameplay = gameplay_value;
    *overlay = overlay_value;
}

fn with_scene_manager_mut<T>(
    world: &mut ecs::World,
    f: impl FnOnce(&mut SceneManager) -> Result<T>,
) -> Result<T> {
    if !world.has_resource::<SceneResource>() {
        return Err(anyhow!("ScenePlugin is not installed"));
    }
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
        .ok_or_else(|| anyhow!("scene manager failed to initialize"))?;
    f(manager)
}

pub fn switch_scene_by_id(world: &mut ecs::World, scene_id: &str) -> Result<bool> {
    let normalized = normalize_scene_label_alias(scene_id);
    let Some(scene) = SceneId::from_label(&normalized) else {
        return Ok(false);
    };
    with_scene_manager_mut(world, |manager| {
        match scene.layer() {
            SceneLayer::World => {
                manager.queue(SceneCommand::ReplaceWorldByLabel(normalized));
                manager.queue(SceneCommand::PauseWorld(false));
            }
            SceneLayer::OverlayUi => {
                manager.queue(SceneCommand::ReplaceOverlayByLabel(normalized));
                manager.queue(SceneCommand::PauseWorld(true));
            }
        }
        Ok(true)
    })
}

pub fn set_world_by_id(world: &mut ecs::World, scene_id: &str) -> Result<bool> {
    let normalized = normalize_scene_label_alias(scene_id);
    let Some(scene) = SceneId::from_label(&normalized) else {
        return Ok(false);
    };
    if scene.layer() != SceneLayer::World {
        return Ok(false);
    }
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::ReplaceWorldByLabel(normalized));
        manager.queue(SceneCommand::PauseWorld(false));
        Ok(true)
    })
}

pub fn push_overlay_by_id(world: &mut ecs::World, scene_id: &str) -> Result<bool> {
    let normalized = normalize_scene_label_alias(scene_id);
    let Some(scene) = SceneId::from_label(&normalized) else {
        return Ok(false);
    };
    if scene.layer() != SceneLayer::OverlayUi {
        return Ok(false);
    }
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PushOverlayByLabel(normalized));
        manager.queue(SceneCommand::PauseWorld(true));
        Ok(true)
    })
}

pub fn pop_overlay(world: &mut ecs::World) -> Result<()> {
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PopOverlay);
        manager.queue(SceneCommand::PauseWorld(false));
        Ok(())
    })
}

pub fn set_world_paused(world: &mut ecs::World, paused: bool) -> Result<()> {
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PauseWorld(paused));
        Ok(())
    })
}

pub fn toggle_world_pause(world: &mut ecs::World) -> Result<()> {
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PauseWorld(!manager.world.paused));
        Ok(())
    })
}

fn flush_lifecycle_status(manager: &mut SceneManager) {
    let lifecycle_events = std::mem::take(&mut manager.channels.lifecycle_events);
    for event in lifecycle_events {
        let line = format_lifecycle_event(event);
        manager.channels.overlay_console_lines.push(line.clone());
        manager.overlay_runtime.ui.editor.status = format!("editor: {line}");
    }
}

fn apply_overlay_messages(manager: &mut SceneManager) {
    let messages = std::mem::take(&mut manager.channels.world_to_overlay);
    for message in messages {
        manager
            .channels
            .overlay_console_lines
            .push(format_world_message(message));
    }

    let messages = std::mem::take(&mut manager.channels.overlay_console_lines);
    for message in messages {
        if manager.overlay_runtime.ui.logs_paused {
            manager.overlay_runtime.ui.log_paused_lines.push(message);
        } else {
            manager.overlay_runtime.ui.log_lines.push(message);
        }
    }
    clamp_scrollback_lines(
        &mut manager.overlay_runtime.ui.log_lines,
        manager.overlay_runtime.ui.max_lines,
    );
    manager.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    if let Ok(mut entity) = manager
        .overlay_runtime
        .world
        .entity_mut(manager.overlay_runtime.ui.scrollback)
        && let Some(mut dirty) = entity.get_mut::<UiDirty>()
    {
        dirty.text = true;
    }
}

fn clamp_scrollback_lines(lines: &mut Vec<String>, max_lines: usize) {
    if max_lines == 0 {
        lines.clear();
        return;
    }
    let overflow = lines.len().saturating_sub(max_lines);
    if overflow > 0 {
        lines.drain(..overflow);
    }
}

fn format_world_message(message: WorldToOverlayMessage) -> String {
    match message {
        WorldToOverlayMessage::Tick { tick, overlay } => {
            format!("[world] tick={} overlay={}", tick, overlay)
        }
        WorldToOverlayMessage::Combat {
            source,
            target,
            damage,
            critical,
        } => {
            if critical {
                format!("[combat] {source} crits {target} for {damage}")
            } else {
                format!("[combat] {source} hits {target} for {damage}")
            }
        }
        WorldToOverlayMessage::Loot {
            item,
            amount,
            rarity,
        } => {
            format!("[loot] +{amount} {item} ({rarity})")
        }
        WorldToOverlayMessage::Quest { quest, state } => match state {
            QuestState::Started => format!("[quest] started: {quest}"),
            QuestState::Progress { current, goal } => {
                format!("[quest] {quest}: {current}/{goal}")
            }
            QuestState::Completed => format!("[quest] completed: {quest}"),
        },
    }
}

fn format_lifecycle_event(event: SceneLifecycleEvent) -> String {
    let phase = match event.phase {
        SceneLifecyclePhase::Enter => "enter",
        SceneLifecyclePhase::Exit => "exit",
        SceneLifecyclePhase::Pause => "pause",
        SceneLifecyclePhase::Resume => "resume",
    };
    let layer = match event.layer {
        SceneLayer::World => "world",
        SceneLayer::OverlayUi => "overlay",
    };
    format!("[world] scene:{layer} {} {phase}", event.scene.label())
}

fn normalize_scene_label_alias(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "gameplay" => "gameplay_stub".to_string(),
        "hub" => "hub_stub".to_string(),
        "console" => "console_ui".to_string(),
        "hud" | "pause" => "hud_ui".to_string(),
        "inventory" | "inv" => "inventory_ui".to_string(),
        other => other.replace('-', "_"),
    }
}

#[cfg(test)]
mod tests {
    use super::domain::{QuestState, WorldToOverlayMessage};
    use super::{
        ScenePlugin, SceneResource, apply_scene_simulation_delta, build_scene_simulation_delta,
        capture_scene_simulation_snapshot, format_world_message, switch_scene_by_id,
    };
    use crate::prelude::*;

    #[test]
    fn format_world_message_renders_all_variants() {
        let tick = format_world_message(WorldToOverlayMessage::Tick {
            tick: 60,
            overlay: "console_ui".to_string(),
        });
        assert!(tick.contains("tick=60"));

        let combat = format_world_message(WorldToOverlayMessage::Combat {
            source: "Scout".to_string(),
            target: "Bat".to_string(),
            damage: 9,
            critical: true,
        });
        assert!(combat.contains("crits"));

        let loot = format_world_message(WorldToOverlayMessage::Loot {
            item: "Glowshard".to_string(),
            amount: 2,
            rarity: "rare".to_string(),
        });
        assert!(loot.contains("[loot]"));

        let quest = format_world_message(WorldToOverlayMessage::Quest {
            quest: "Map".to_string(),
            state: QuestState::Progress {
                current: 2,
                goal: 3,
            },
        });
        assert!(quest.contains("2/3"));
    }

    #[test]
    fn scene_plugin_toggles_pause_overlay_and_updates_public_state() {
        let mut app = App::headless();
        app.add_plugin(ScenePlugin);
        app.world_mut()
            .resource_mut::<InputState>()
            .expect("input state should exist")
            .toggle_pause_menu = true;

        let app = app.run_for_frames(1).expect("scene plugin should run");
        let scene = app
            .world()
            .resource::<SceneRuntimeState>()
            .expect("scene state should exist");
        assert_eq!(scene.overlay_scene_label, "hud_ui");
        assert!(scene.overlay_visible);
        assert!(scene.world_paused);
    }

    #[test]
    fn scene_helper_switches_world_scene_by_label() {
        let mut app = App::headless();
        app.add_plugin(ScenePlugin);
        switch_scene_by_id(app.world_mut(), "hub").expect("scene switch should queue");

        let app = app.run_for_frames(1).expect("scene plugin should run");
        let scene = app
            .world()
            .resource::<SceneRuntimeState>()
            .expect("scene state should exist");
        assert_eq!(scene.world_scene_label, "hub_stub");
        assert!(!scene.world_paused);
    }

    #[test]
    fn scene_plugin_routes_world_tick_messages_into_overlay_log() {
        let mut app = App::headless();
        app.add_plugin(ScenePlugin);

        let app = app.run_for_ticks(60).expect("scene plugin should run");
        let scene = app
            .world()
            .resource::<SceneResource>()
            .expect("scene resource should exist");
        let manager = scene
            .manager
            .as_ref()
            .expect("scene manager should be initialized");
        assert!(
            manager
                .overlay_runtime
                .ui
                .log_lines
                .iter()
                .any(|line| line.contains("tick=60"))
        );
    }

    #[test]
    fn scene_simulation_delta_round_trips_back_to_the_current_snapshot() {
        let mut app = App::headless();
        app.add_plugin(ScenePlugin);
        let mut app = app
            .run_for_frames(1)
            .expect("scene plugin should initialize");

        let base_snapshot = {
            let scene = app
                .world()
                .resource::<SceneResource>()
                .expect("scene resource should exist");
            let manager = scene
                .manager
                .as_ref()
                .expect("scene manager should be initialized");
            capture_scene_simulation_snapshot(manager).expect("base snapshot should capture")
        };

        {
            let mut scene = app
                .world_mut()
                .resource_mut::<SceneResource>()
                .expect("scene resource should exist");
            let manager = scene
                .manager
                .as_mut()
                .expect("scene manager should be initialized");
            manager.world_runtime.ctx.player_move_x = -0.25;
            manager.world_runtime.ctx.player_move_y = 0.75;
            manager.world_runtime.ctx.frame_count = 7;
            if let Ok(mut entity) = manager
                .world_runtime
                .ctx
                .world
                .entity_mut(manager.world_runtime.ctx.debug_entity)
                && let Some(mut position) = entity.get_mut::<super::domain::WorldDebugPosition>()
            {
                position.x = 4.0;
                position.y = -2.0;
            }
        }

        let current_snapshot = {
            let scene = app
                .world()
                .resource::<SceneResource>()
                .expect("scene resource should exist");
            let manager = scene
                .manager
                .as_ref()
                .expect("scene manager should be initialized");
            capture_scene_simulation_snapshot(manager).expect("current snapshot should capture")
        };

        let delta = build_scene_simulation_delta(&base_snapshot, &current_snapshot);
        let rebuilt_snapshot = apply_scene_simulation_delta(&base_snapshot, &delta);

        assert_eq!(rebuilt_snapshot, current_snapshot);
        assert_eq!(delta.context.player_move_x, Some(-0.25));
        assert_eq!(delta.context.player_move_y, Some(0.75));
        assert_eq!(delta.context.frame_count, Some(7));
        assert_eq!(
            delta.entities.debug_position,
            Some(current_snapshot.entities.debug_position)
        );
        assert_eq!(delta.context.world_scene_label, None);
    }
}
