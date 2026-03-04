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
use crate::state::{GameplayRuntimeConfig, SceneRuntimeState, UiOverlayState};
use anyhow::{Result, anyhow};

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

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<GameplayRuntimeConfig>();
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

fn scene_setup_system(
    window: Res<WindowState>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(SceneManager::new(&window)?);
    }
    if let Some(manager) = scene_resource.manager.as_mut() {
        sync_overlay_viewport(manager, &window);
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
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
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

fn publish_scene_state(
    manager: &SceneManager,
    scene_state: &mut SceneRuntimeState,
    gameplay: &mut GameplayRuntimeConfig,
    overlay: &mut UiOverlayState,
) {
    *gameplay = GameplayRuntimeConfig {
        chunk_size: manager.world_runtime.ctx.gameplay_config.chunk_size,
        chunk_load_radius: manager.world_runtime.ctx.gameplay_config.chunk_load_radius,
        infinite_world: manager.world_runtime.ctx.gameplay_config.infinite_world,
    };
    *scene_state = SceneRuntimeState {
        world_scene_label: manager.world.active.label().to_string(),
        overlay_scene_label: manager.active_overlay().label().to_string(),
        overlay_visible: manager.overlay_visible(),
        world_paused: manager.world.paused,
        enemy_kills: manager.world_runtime.ctx.enemy_kills,
        gameplay: *gameplay,
    };
    overlay.screen_size = manager.overlay_runtime.ui.screen_size;
    overlay.scale = manager.overlay_runtime.ui.scale;
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
    use super::{ScenePlugin, SceneResource, format_world_message, switch_scene_by_id};
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
}
