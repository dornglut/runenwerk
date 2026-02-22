use crate::ui::{ConsoleUiState, initialize_console_ui, load_console_template};
use anyhow::Result;
use ecs::{EntityHandle, World};
use scheduler::{Node, Scheduler, SchedulerBuilder};
use std::time::SystemTime;

mod config;
mod gameplay;
mod lifecycle;
mod manager;

pub use config::{
    GAMEPLAY_CONFIG_PATH, GameplayConfig, gameplay_config_modified, load_gameplay_config,
    load_gameplay_config_with_modified,
};
pub use gameplay::gameplay_apply_live_config;
use gameplay::{
    gameplay_combat_system, gameplay_decide_system, gameplay_emit_ui_system, gameplay_move_system,
    gameplay_resolve_system, gameplay_scene_bootstrap, gameplay_sense_system,
};
pub use lifecycle::{SceneLifecycleEvent, SceneLifecyclePhase};
pub use manager::SceneManager;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SceneLayer {
    World,
    OverlayUi,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SceneId {
    GameplayStub,
    HubStub,
    ConsoleUi,
    HudUi,
}

impl SceneId {
    pub fn layer(self) -> SceneLayer {
        match self {
            Self::GameplayStub | Self::HubStub => SceneLayer::World,
            Self::ConsoleUi | Self::HudUi => SceneLayer::OverlayUi,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::GameplayStub => "gameplay_stub",
            Self::HubStub => "hub_stub",
            Self::ConsoleUi => "console_ui",
            Self::HudUi => "hud_ui",
        }
    }

    pub fn next_overlay(self) -> Self {
        match self {
            Self::ConsoleUi => Self::HudUi,
            Self::HudUi => Self::ConsoleUi,
            Self::GameplayStub | Self::HubStub => Self::ConsoleUi,
        }
    }

    pub fn previous_overlay(self) -> Self {
        match self {
            Self::ConsoleUi => Self::HudUi,
            Self::HudUi => Self::ConsoleUi,
            Self::GameplayStub | Self::HubStub => Self::ConsoleUi,
        }
    }
}

pub fn template_path_for_scene(scene: SceneId) -> Option<&'static str> {
    match scene {
        SceneId::ConsoleUi => Some("assets/ui/console.ron"),
        SceneId::HudUi => Some("assets/ui/hud.ron"),
        SceneId::GameplayStub | SceneId::HubStub => None,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SceneSlot {
    pub active: SceneId,
    pub paused: bool,
    pub visible: bool,
}

impl SceneSlot {
    pub fn new(active: SceneId) -> Self {
        Self {
            active,
            paused: false,
            visible: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SceneCommand {
    ReplaceWorld(SceneId),
    ReplaceOverlay(SceneId),
    PushOverlay(SceneId),
    PopOverlay,
    PauseWorld(bool),
}

#[derive(Debug, Default, Clone)]
pub struct SceneTransitionResult {
    pub world_changed: bool,
    pub overlay_changed: bool,
    pub world_pause_changed: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct WorldFrameCounter {
    pub value: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct WorldDebugPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct WorldDebugVelocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AgentTeam {
    Player,
    Enemy,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Seek,
    Attack,
    Recover,
    Dead,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentPrevPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentVelocity {
    pub speed: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentHealth {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentTarget {
    pub entity: Option<EntityHandle>,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentMoveIntent {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct AgentCombat {
    pub attack_range: f32,
    pub attack_damage: i32,
    pub cooldown_ticks: u32,
    pub cooldown_remaining: u32,
}

pub struct WorldSceneContext {
    pub world: World,
    pub scene: SceneId,
    pub gameplay_config: GameplayConfig,
    pub delta_seconds: f32,
    pub fixed_step_seconds: f32,
    pub fixed_step_accumulator: f32,
    pub gameplay_config_modified: Option<SystemTime>,
    pub overlay_consumed: bool,
    pub overlay_scene: SceneId,
    pub tick_entity: EntityHandle,
    pub debug_entity: EntityHandle,
    pub frame_count: u64,
    pub enemy_kills: u32,
    pub pending_damage: Vec<PendingDamage>,
    pub outbound_notifications: Vec<WorldToOverlayMessage>,
}

#[derive(Debug, Clone)]
pub struct PendingDamage {
    pub source: EntityHandle,
    pub target: EntityHandle,
    pub amount: i32,
    pub critical: bool,
}

pub struct WorldSceneRuntime {
    pub scheduler: Scheduler<WorldSceneContext>,
    pub ctx: WorldSceneContext,
}

pub struct OverlaySceneRuntime {
    pub world: World,
    pub ui: ConsoleUiState,
}

#[derive(Debug, Default, Clone)]
pub struct SceneChannels {
    pub overlay_submit: Vec<OverlaySubmitMessage>,
    pub overlay_command_inputs: Vec<OverlayCommandInput>,
    pub world_to_overlay: Vec<WorldToOverlayMessage>,
    pub overlay_console_lines: Vec<String>,
    pub lifecycle_events: Vec<SceneLifecycleEvent>,
}

#[derive(Debug, Clone)]
pub enum OverlaySubmitMessage {
    Line(String),
}

#[derive(Debug, Clone)]
pub enum OverlayCommandInput {
    Line(String),
}

#[derive(Debug, Clone)]
pub enum WorldToOverlayMessage {
    Tick {
        tick: u64,
        overlay: SceneId,
    },
    Combat {
        source: String,
        target: String,
        damage: u32,
        critical: bool,
    },
    Loot {
        item: String,
        amount: u32,
        rarity: String,
    },
    Quest {
        quest: String,
        state: QuestState,
    },
}

#[derive(Debug, Clone)]
pub enum QuestState {
    Started,
    Progress { current: u32, goal: u32 },
    Completed,
}

fn build_overlay_runtime(
    scene: SceneId,
    screen_size: (f32, f32),
    scale: f32,
) -> Result<OverlaySceneRuntime> {
    let mut world = World::new();
    let mut ui = initialize_console_ui(&mut world);
    ui.screen_size = screen_size;
    ui.scale = scale;
    if let Some(path) = template_path_for_scene(scene) {
        let template = std::path::Path::new(path);
        if template.exists() {
            load_console_template(&mut world, &mut ui, template)?;
        }
    }
    Ok(OverlaySceneRuntime { world, ui })
}

fn world_scene_input_gate_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed {
        return Ok(());
    }
    Ok(())
}

fn world_scene_tick_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed {
        return Ok(());
    }
    if let Some(counter) = ctx
        .world
        .get_component_mut::<WorldFrameCounter>(ctx.tick_entity)
    {
        counter.value = counter.value.saturating_add(1);
        ctx.frame_count = counter.value;
        if counter.value % 60 == 0 {
            ctx.outbound_notifications
                .push(WorldToOverlayMessage::Tick {
                    tick: counter.value,
                    overlay: ctx.overlay_scene,
                });
        }
        if ctx.scene == SceneId::HubStub && counter.value % 180 == 0 {
            ctx.outbound_notifications
                .push(WorldToOverlayMessage::Loot {
                    item: "Rested".to_string(),
                    amount: 1,
                    rarity: "hub".to_string(),
                });
        }
        if ctx.scene == SceneId::HubStub && counter.value % 300 == 0 {
            ctx.outbound_notifications
                .push(WorldToOverlayMessage::Quest {
                    quest: "Prepare For Expedition".to_string(),
                    state: QuestState::Started,
                });
        }
    }
    Ok(())
}

fn world_scene_debug_motion_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed {
        return Ok(());
    }
    let mut velocity = ctx
        .world
        .get_component::<WorldDebugVelocity>(ctx.debug_entity)
        .copied()
        .unwrap_or(WorldDebugVelocity { x: 0.0, y: 0.0 });
    let sim_step = ctx.delta_seconds.clamp(0.0, 0.25);
    if let Some(position) = ctx
        .world
        .get_component_mut::<WorldDebugPosition>(ctx.debug_entity)
    {
        position.x += velocity.x * sim_step;
        position.y += velocity.y * sim_step;
        let min_x = ctx.gameplay_config.bounds.min_x;
        let max_x = ctx.gameplay_config.bounds.max_x;
        let min_y = ctx.gameplay_config.bounds.min_y;
        let max_y = ctx.gameplay_config.bounds.max_y;
        if position.x < min_x || position.x > max_x {
            velocity.x = -velocity.x;
            position.x = position.x.clamp(min_x, max_x);
        }
        if position.y < min_y || position.y > max_y {
            velocity.y = -velocity.y;
            position.y = position.y.clamp(min_y, max_y);
        }
    }
    if let Some(velocity_mut) = ctx
        .world
        .get_component_mut::<WorldDebugVelocity>(ctx.debug_entity)
    {
        *velocity_mut = velocity;
    }
    Ok(())
}

fn build_world_scene_runtime(scene: SceneId) -> Result<WorldSceneRuntime> {
    let (gameplay_config, gameplay_config_modified) = load_gameplay_config_with_modified();
    let mut world = World::new();
    world.register_component::<WorldFrameCounter>();
    world.register_component::<WorldDebugPosition>();
    world.register_component::<WorldDebugVelocity>();
    if scene == SceneId::GameplayStub {
        gameplay_scene_bootstrap(&mut world, &gameplay_config);
    }
    let tick_entity = world.spawn_entity_typed(WorldFrameCounter { value: 0 });
    let debug_entity = world.spawn_entity(vec![
        Box::new(WorldDebugPosition { x: 0.0, y: 0.0 }) as Box<dyn std::any::Any>,
        Box::new(WorldDebugVelocity { x: 1.25, y: 0.75 }) as Box<dyn std::any::Any>,
    ]);
    let ctx = WorldSceneContext {
        world,
        scene,
        gameplay_config,
        delta_seconds: 1.0 / 60.0,
        fixed_step_seconds: 1.0 / 60.0,
        fixed_step_accumulator: 0.0,
        gameplay_config_modified,
        overlay_consumed: false,
        overlay_scene: SceneId::ConsoleUi,
        tick_entity,
        debug_entity,
        frame_count: 0,
        enemy_kills: 0,
        pending_damage: Vec::new(),
        outbound_notifications: Vec::new(),
    };

    let scheduler = SchedulerBuilder::<WorldSceneContext>::new()
        .add_node(
            "world_input_gate",
            Node::new("world_input_gate", world_scene_input_gate_system),
        )
        .add_node_with_edges(
            "world_tick",
            Node::new("world_tick", world_scene_tick_system),
            &["world_input_gate"],
        )
        .add_node_with_edges(
            "world_debug_motion",
            Node::new("world_debug_motion", world_scene_debug_motion_system),
            &["world_tick"],
        )
        .add_node_with_edges(
            "gameplay_sense",
            Node::new("gameplay_sense", gameplay_sense_system),
            &["world_debug_motion"],
        )
        .add_node_with_edges(
            "gameplay_decide",
            Node::new("gameplay_decide", gameplay_decide_system),
            &["gameplay_sense"],
        )
        .add_node_with_edges(
            "gameplay_move",
            Node::new("gameplay_move", gameplay_move_system),
            &["gameplay_decide"],
        )
        .add_node_with_edges(
            "gameplay_combat",
            Node::new("gameplay_combat", gameplay_combat_system),
            &["gameplay_move"],
        )
        .add_node_with_edges(
            "gameplay_resolve",
            Node::new("gameplay_resolve", gameplay_resolve_system),
            &["gameplay_combat"],
        )
        .add_node_with_edges(
            "gameplay_emit_ui",
            Node::new("gameplay_emit_ui", gameplay_emit_ui_system),
            &["gameplay_resolve"],
        )
        .build()?;

    Ok(WorldSceneRuntime { scheduler, ctx })
}

#[cfg(test)]
mod tests {
    use super::{
        OverlaySceneRuntime, SceneCommand, SceneId, SceneLifecyclePhase, SceneManager,
        WorldDebugPosition,
        WorldFrameCounter, WorldToOverlayMessage, build_world_scene_runtime, load_gameplay_config,
    };
    use crate::ui::initialize_console_ui;
    use ecs::World;

    fn make_overlay_runtime() -> OverlaySceneRuntime {
        let mut world = World::new();
        let ui = initialize_console_ui(&mut world);
        OverlaySceneRuntime { world, ui }
    }

    #[test]
    fn replace_overlay_switches_active_overlay() {
        let mut manager =
            SceneManager::new(make_overlay_runtime()).expect("scene manager should build");
        assert_eq!(manager.active_overlay(), SceneId::ConsoleUi);
        manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
        let result = manager.apply_pending().expect("apply should succeed");
        assert!(result.overlay_changed);
        assert_eq!(manager.active_overlay(), SceneId::HudUi);
    }

    #[test]
    fn replace_world_switches_active_world() {
        let mut manager =
            SceneManager::new(make_overlay_runtime()).expect("scene manager should build");
        assert_eq!(manager.world.active, SceneId::GameplayStub);
        manager.queue(SceneCommand::ReplaceWorld(SceneId::HubStub));
        let result = manager.apply_pending().expect("apply should succeed");
        assert!(result.world_changed);
        assert_eq!(manager.world.active, SceneId::HubStub);
    }

    #[test]
    fn pause_world_command_toggles_pause_flag() {
        let mut manager =
            SceneManager::new(make_overlay_runtime()).expect("scene manager should build");
        assert!(!manager.world.paused);

        manager.queue(SceneCommand::PauseWorld(true));
        let paused = manager.apply_pending().expect("apply should succeed");
        assert!(paused.world_pause_changed);
        assert!(manager.world.paused);

        manager.queue(SceneCommand::PauseWorld(false));
        let resumed = manager.apply_pending().expect("apply should succeed");
        assert!(resumed.world_pause_changed);
        assert!(!manager.world.paused);
    }

    #[test]
    fn lifecycle_events_include_pause_and_resume() {
        let mut manager =
            SceneManager::new(make_overlay_runtime()).expect("scene manager should build");
        manager.channels.lifecycle_events.clear();

        manager.queue(SceneCommand::PushOverlay(SceneId::HudUi));
        let _ = manager.apply_pending().expect("push should succeed");
        assert!(manager.channels.lifecycle_events.iter().any(|event| {
            event.scene == SceneId::ConsoleUi && event.phase == SceneLifecyclePhase::Pause
        }));
        assert!(manager.channels.lifecycle_events.iter().any(|event| {
            event.scene == SceneId::HudUi && event.phase == SceneLifecyclePhase::Enter
        }));

        manager.channels.lifecycle_events.clear();
        manager.queue(SceneCommand::PopOverlay);
        let _ = manager.apply_pending().expect("pop should succeed");
        assert!(manager.channels.lifecycle_events.iter().any(|event| {
            event.scene == SceneId::HudUi && event.phase == SceneLifecyclePhase::Exit
        }));
        assert!(manager.channels.lifecycle_events.iter().any(|event| {
            event.scene == SceneId::ConsoleUi && event.phase == SceneLifecyclePhase::Resume
        }));
    }

    #[test]
    fn gameplay_config_loads_with_positive_core_values() {
        let cfg = load_gameplay_config();
        assert!(cfg.player.health > 0);
        assert!(cfg.enemy.health > 0);
        assert!(cfg.enemy_count > 0);
        assert!(cfg.bounds.max_x > cfg.bounds.min_x);
        assert!(cfg.bounds.max_y > cfg.bounds.min_y);
    }

    #[test]
    fn pop_overlay_keeps_at_least_one_overlay() {
        let mut manager =
            SceneManager::new(make_overlay_runtime()).expect("scene manager should build");
        manager.queue(SceneCommand::PopOverlay);
        let result = manager.apply_pending().expect("apply should succeed");
        assert!(!result.overlay_changed);
        assert_eq!(manager.overlays.len(), 1);
        assert_eq!(manager.active_overlay(), SceneId::ConsoleUi);
    }

    #[test]
    fn push_and_pop_overlay_restores_previous_runtime_state() {
        let mut manager =
            SceneManager::new(make_overlay_runtime()).expect("scene manager should build");
        manager.overlay_runtime.ui.editor.status = "editor: custom console".to_string();
        manager.queue(SceneCommand::PushOverlay(SceneId::HudUi));
        let _ = manager.apply_pending().expect("push should succeed");
        manager.overlay_runtime.ui.editor.status = "editor: hud custom".to_string();

        manager.queue(SceneCommand::PopOverlay);
        let _ = manager.apply_pending().expect("pop should succeed");
        assert_eq!(manager.active_overlay(), SceneId::ConsoleUi);
        assert_eq!(
            manager.overlay_runtime.ui.editor.status,
            "editor: custom console"
        );
    }

    #[test]
    fn world_scene_runtime_blocks_when_overlay_consumed() {
        let mut runtime = build_world_scene_runtime(SceneId::GameplayStub)
            .expect("runtime should build");
        runtime.ctx.overlay_consumed = true;
        let initial_pos = runtime
            .ctx
            .world
            .get_component::<WorldDebugPosition>(runtime.ctx.debug_entity)
            .expect("position component should exist")
            .to_owned();
        runtime
            .scheduler
            .run(&mut runtime.ctx)
            .expect("world scheduler should run");
        assert_eq!(runtime.ctx.frame_count, 0);
        let blocked_pos = runtime
            .ctx
            .world
            .get_component::<WorldDebugPosition>(runtime.ctx.debug_entity)
            .expect("position component should exist")
            .to_owned();
        assert_eq!(blocked_pos.x, initial_pos.x);
        assert_eq!(blocked_pos.y, initial_pos.y);

        runtime.ctx.overlay_consumed = false;
        runtime
            .scheduler
            .run(&mut runtime.ctx)
            .expect("world scheduler should run");
        assert_eq!(runtime.ctx.frame_count, 1);
        let value = runtime
            .ctx
            .world
            .get_component::<WorldFrameCounter>(runtime.ctx.tick_entity)
            .expect("counter component should exist")
            .value;
        assert_eq!(value, 1);
        let moved_pos = runtime
            .ctx
            .world
            .get_component::<WorldDebugPosition>(runtime.ctx.debug_entity)
            .expect("position component should exist")
            .to_owned();
        assert_ne!(moved_pos.x, blocked_pos.x);
    }

    #[test]
    fn world_scene_runtime_emits_notifications_on_tick_boundaries() {
        let mut runtime = build_world_scene_runtime(SceneId::GameplayStub)
            .expect("runtime should build");
        runtime.ctx.overlay_consumed = false;
        for _ in 0..60 {
            runtime
                .scheduler
                .run(&mut runtime.ctx)
                .expect("world scheduler should run");
        }
        assert_eq!(runtime.ctx.frame_count, 60);
        assert!(
            runtime
                .ctx
                .outbound_notifications
                .iter()
                .any(|msg| matches!(msg, WorldToOverlayMessage::Tick { tick: 60, .. }))
        );
    }
}
