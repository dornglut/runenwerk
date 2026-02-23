pub mod domain;
pub mod manifest;

use self::domain::{
    GAMEPLAY_CONFIG_PATH, QuestState, SceneCommand, SceneId, SceneLayer, SceneLifecycleEvent,
    SceneLifecyclePhase, WorldToOverlayMessage, gameplay_config_modified,
    load_gameplay_config_with_modified_and_error,
};
use crate::plugins::shared::{ReloadStatusPayload, should_reload};
use crate::plugins::ui::domain::UiDirty;
use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use anyhow::Result;

pub struct ScenePlugin;

impl EnginePlugin for ScenePlugin {
    fn name(&self) -> &'static str {
        "scene"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "scene_transition",
            scene_transition_system,
            &["overlay_ui_editor"],
        );
        builder.add_node_with_edges(
            "world_scene_update",
            world_scene_update_system,
            &["scene_transition"],
        );
        builder.add_node_with_edges(
            "scene_overlay_format_messages",
            scene_overlay_format_messages_system,
            &["world_scene_update"],
        );
        builder.add_node_with_edges(
            "scene_overlay_apply_messages",
            scene_overlay_apply_messages_system,
            &["scene_overlay_format_messages"],
        );
        Ok(())
    }
}

pub fn scene_transition_system(data: &mut EngineData) -> anyhow::Result<()> {
    if data.input.toggle_pause_menu {
        let show_overlay = !data.scene.overlay_visible();
        data.scene.set_active_overlay_visible(show_overlay);
        data.scene.queue(SceneCommand::PauseWorld(show_overlay));
        if show_overlay && data.scene.active_overlay() != SceneId::HudUi {
            data.scene
                .queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
        }
    }
    if data.input.scene_next {
        let next = data.scene.active_overlay().next_overlay();
        data.scene.queue(SceneCommand::ReplaceOverlay(next));
    }
    if data.input.scene_prev {
        let prev = data.scene.active_overlay().previous_overlay();
        data.scene.queue(SceneCommand::ReplaceOverlay(prev));
    }
    if data.input.scene_console {
        data.scene
            .queue(SceneCommand::ReplaceOverlay(SceneId::ConsoleUi));
    }
    if data.input.scene_hud {
        data.scene
            .queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
    }
    if data.input.scene_overlay_push {
        let next = data.scene.active_overlay().next_overlay();
        data.scene.queue(SceneCommand::PushOverlay(next));
    }
    if data.input.scene_overlay_pop {
        data.scene.queue(SceneCommand::PopOverlay);
    }

    let result = data.scene.apply_pending()?;
    if result.world_changed {
        data.scene.overlay_runtime.ui.editor.status = format!(
            "editor: world scene switched to {}",
            data.scene.world.active.label()
        );
    }
    if result.overlay_changed {
        let active = data.scene.active_overlay();
        let path = data
            .scene
            .registry
            .ui_template_path(active)
            .unwrap_or("<none>");
        data.scene.overlay_runtime.ui.editor.status = format!(
            "editor: overlay scene switched to {} ({}) [stack={}]",
            active.label(),
            path,
            data.scene.overlays.len()
        );
    }
    if result.world_pause_changed {
        data.scene.overlay_runtime.ui.editor.status = if data.scene.world.paused {
            "editor: world scene paused".to_string()
        } else {
            "editor: world scene resumed".to_string()
        };
    }

    let lifecycle_events = std::mem::take(&mut data.scene.channels.lifecycle_events);
    for event in lifecycle_events {
        let line = format_lifecycle_event(event);
        data.scene.channels.overlay_console_lines.push(line.clone());
        data.scene.overlay_runtime.ui.editor.status = format!("editor: {line}");
    }

    Ok(())
}

pub fn world_scene_update_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.scene.world.visible || data.scene.world.paused {
        return Ok(());
    }

    let active_overlay = data.scene.active_overlay();
    let overlay_visible = data.scene.overlay_visible();
    let world_paused = data.scene.world.paused;
    let runtime = &mut data.scene.world_runtime;
    runtime.ctx.overlay_consumed = data.input.overlay_consumed;
    runtime.ctx.overlay_scene = active_overlay;
    runtime.ctx.player_move_x = (if data.input.world_move_right {
        1.0
    } else {
        0.0
    }) - (if data.input.world_move_left { 1.0 } else { 0.0 });
    runtime.ctx.player_move_y = (if data.input.world_move_up { 1.0 } else { 0.0 })
        - (if data.input.world_move_down { 1.0 } else { 0.0 });
    if !overlay_visible && !world_paused {
        let camera_cfg = &runtime.ctx.gameplay_config.camera;
        let rotate_sensitivity = camera_cfg.rotate_sensitivity.max(0.0);
        let yaw_sign = if camera_cfg.invert_x { 1.0 } else { -1.0 };
        let pitch_sign = if camera_cfg.invert_y { -1.0 } else { 1.0 };
        runtime.ctx.camera_yaw += data.input.mouse_delta.0 * rotate_sensitivity * yaw_sign;
        runtime.ctx.camera_pitch += data.input.mouse_delta.1 * rotate_sensitivity * pitch_sign;
    }
    if !overlay_visible && data.input.scroll_delta.abs() > f32::EPSILON {
        let camera_cfg = &runtime.ctx.gameplay_config.camera;
        let zoom_sensitivity = camera_cfg.zoom_sensitivity.max(0.0);
        let zoom_sign = if camera_cfg.invert_zoom { 1.0 } else { -1.0 };
        runtime.ctx.camera_distance += data.input.scroll_delta * zoom_sensitivity * zoom_sign;
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
            data.scene.world.active.label(),
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
        data.scene
            .channels
            .overlay_console_lines
            .push(format!("[world] {}", payload.line()));
    }

    let fixed_dt = runtime
        .ctx
        .fixed_step_seconds
        .clamp(1.0 / 240.0, 1.0 / 30.0);
    let max_steps = runtime.ctx.gameplay_config.max_catchup_steps.clamp(1, 4) as usize;
    runtime.ctx.fixed_step_accumulator = (runtime.ctx.fixed_step_accumulator
        + data.time.delta_seconds.min(0.25))
    .min(fixed_dt * max_steps as f32);

    let mut steps = 0usize;
    while runtime.ctx.fixed_step_accumulator + f32::EPSILON >= fixed_dt && steps < max_steps {
        runtime.ctx.delta_seconds = fixed_dt;
        runtime.scheduler.run(&mut runtime.ctx)?;
        runtime.ctx.fixed_step_accumulator -= fixed_dt;
        let outbound = std::mem::take(&mut runtime.ctx.outbound_notifications);
        data.scene.channels.world_to_overlay.extend(outbound);
        steps = steps.saturating_add(1);
    }
    if steps == max_steps && runtime.ctx.fixed_step_accumulator >= fixed_dt {
        runtime.ctx.fixed_step_accumulator = 0.0;
        tracing::warn!("world fixed-step loop saturated, dropping accumulated time");
    }

    Ok(())
}

pub fn scene_overlay_format_messages_system(data: &mut EngineData) -> anyhow::Result<()> {
    let messages = std::mem::take(&mut data.scene.channels.world_to_overlay);
    if messages.is_empty() {
        return Ok(());
    }

    for message in messages {
        data.scene
            .channels
            .overlay_console_lines
            .push(format_world_message(message));
    }

    Ok(())
}

pub fn scene_overlay_apply_messages_system(data: &mut EngineData) -> anyhow::Result<()> {
    let messages = std::mem::take(&mut data.scene.channels.overlay_console_lines);
    if messages.is_empty() {
        return Ok(());
    }

    for message in messages {
        if data.scene.overlay_runtime.ui.logs_paused {
            data.scene.overlay_runtime.ui.log_paused_lines.push(message);
        } else {
            data.scene.overlay_runtime.ui.log_lines.push(message);
        }
    }
    clamp_scrollback_lines(
        &mut data.scene.overlay_runtime.ui.log_lines,
        data.scene.overlay_runtime.ui.max_lines,
    );
    data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    if let Some(dirty) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiDirty>(data.scene.overlay_runtime.ui.scrollback)
    {
        dirty.text = true;
    }

    Ok(())
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
            format!("[world] tick={} overlay={}", tick, overlay.label())
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

#[cfg(test)]
mod tests {
    use super::domain::{QuestState, SceneId, WorldToOverlayMessage};
    use super::format_world_message;

    #[test]
    fn format_world_message_renders_all_variants() {
        let tick = format_world_message(WorldToOverlayMessage::Tick {
            tick: 60,
            overlay: SceneId::ConsoleUi,
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
}
