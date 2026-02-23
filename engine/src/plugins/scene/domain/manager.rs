use super::{
    OverlaySceneRuntime, SceneCommand, SceneId, SceneLayer, SceneLifecycleEvent,
    SceneLifecyclePhase, SceneRegistry, SceneSlot, SceneTransitionResult, WorldSceneRuntime,
    build_overlay_runtime, build_world_scene_runtime,
};
use anyhow::Result;

pub struct SceneManager {
    pub world: SceneSlot,
    pub world_runtime: WorldSceneRuntime,
    pub overlay_runtime: OverlaySceneRuntime,
    pub registry: SceneRegistry,
    pub overlay_back_stack: Vec<(SceneSlot, OverlaySceneRuntime)>,
    pub channels: super::SceneChannels,
    pub overlays: Vec<SceneSlot>,
    pub pending: Vec<SceneCommand>,
}

impl SceneManager {
    fn emit_lifecycle(&mut self, scene: SceneId, phase: SceneLifecyclePhase) {
        self.channels.lifecycle_events.push(SceneLifecycleEvent {
            scene,
            layer: scene.layer(),
            phase,
        });
    }

    pub fn new(overlay_runtime: OverlaySceneRuntime) -> Result<Self> {
        let world_scene = SceneId::GameplayStub;
        let registry = SceneRegistry::load();
        let mut manager = Self {
            world: SceneSlot::new(world_scene),
            world_runtime: build_world_scene_runtime(world_scene)?,
            overlay_runtime,
            registry,
            overlay_back_stack: Vec::new(),
            channels: super::SceneChannels::default(),
            overlays: vec![SceneSlot {
                active: SceneId::ConsoleUi,
                paused: false,
                visible: false,
            }],
            pending: Vec::new(),
        };
        if let Some(path) = manager.registry.ui_template_path(SceneId::ConsoleUi) {
            let template = std::path::Path::new(path);
            if template.exists() {
                crate::plugins::ui::domain::load_console_template(
                    &mut manager.overlay_runtime.world,
                    &mut manager.overlay_runtime.ui,
                    template,
                )?;
            }
        }
        manager.emit_lifecycle(world_scene, SceneLifecyclePhase::Enter);
        manager.emit_lifecycle(SceneId::ConsoleUi, SceneLifecyclePhase::Enter);
        Ok(manager)
    }

    pub fn active_overlay(&self) -> SceneId {
        self.overlays
            .last()
            .map(|s| s.active)
            .unwrap_or(SceneId::ConsoleUi)
    }

    pub fn queue(&mut self, command: SceneCommand) {
        self.pending.push(command);
    }

    pub fn overlay_visible(&self) -> bool {
        self.overlays.last().map(|s| s.visible).unwrap_or(false)
    }

    pub fn set_active_overlay_visible(&mut self, visible: bool) {
        if let Some(slot) = self.overlays.last_mut() {
            slot.visible = visible;
        }
    }

    fn overlay_viewport(&self) -> ((f32, f32), f32) {
        (
            self.overlay_runtime.ui.screen_size,
            self.overlay_runtime.ui.scale,
        )
    }

    pub fn set_overlay_viewport(&mut self, screen_size: (f32, f32), scale: f32) {
        self.overlay_runtime.ui.screen_size = screen_size;
        self.overlay_runtime.ui.scale = scale;
        self.overlay_runtime.ui.layout_dirty = true;
        for (_, runtime) in &mut self.overlay_back_stack {
            runtime.ui.screen_size = screen_size;
            runtime.ui.scale = scale;
            runtime.ui.layout_dirty = true;
        }
    }

    pub fn apply_pending(&mut self) -> Result<SceneTransitionResult> {
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
