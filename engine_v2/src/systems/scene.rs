use crate::runtime::{
    EngineData, QuestState, SceneCommand, SceneId, SceneLifecycleEvent, SceneLifecyclePhase,
    WorldToOverlayMessage, template_path_for_scene,
};
use crate::ui::UiDirty;

pub fn scene_transition_system(data: &mut EngineData) -> anyhow::Result<()> {
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
        let path = template_path_for_scene(active).unwrap_or("<none>");
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

    data.scene.world_runtime.ctx.overlay_consumed = data.input.overlay_consumed;
    data.scene.world_runtime.ctx.overlay_scene = data.scene.active_overlay();
    data.scene
        .world_runtime
        .scheduler
        .run(&mut data.scene.world_runtime.ctx)?;
    let outbound = std::mem::take(&mut data.scene.world_runtime.ctx.outbound_notifications);
    data.scene.channels.world_to_overlay.extend(outbound);

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
        crate::runtime::SceneLayer::World => "world",
        crate::runtime::SceneLayer::OverlayUi => "overlay",
    };
    format!("[world] scene:{layer} {} {phase}", event.scene.label())
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

#[cfg(test)]
mod tests {
    use super::format_world_message;
    use crate::runtime::{QuestState, SceneId, WorldToOverlayMessage};

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
