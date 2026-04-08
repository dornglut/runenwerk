use crate::plugins::SceneManager;
use crate::prelude::domain::{
    QuestState, SceneLayer, SceneLifecycleEvent, SceneLifecyclePhase, WorldToOverlayMessage,
};

// Owner: Engine Scene Plugin - Runtime Messaging
pub(crate) fn flush_lifecycle_status(manager: &mut SceneManager) {
    let lifecycle_events = std::mem::take(&mut manager.channels.lifecycle_events);
    for event in lifecycle_events {
        let line = format_lifecycle_event(event);
        manager.channels.overlay_console_lines.push(line);
    }
}

pub(crate) fn apply_overlay_messages(manager: &mut SceneManager) {
    let messages = std::mem::take(&mut manager.channels.world_to_overlay);
    for message in messages {
        manager
            .channels
            .overlay_console_lines
            .push(format_world_message(message));
    }

    let messages = std::mem::take(&mut manager.channels.overlay_console_lines);
    for message in messages {
        manager.overlay_runtime.ui.log_lines.push(message);
    }
    clamp_scrollback_lines(
        &mut manager.overlay_runtime.ui.log_lines,
        manager.overlay_runtime.ui.max_lines,
    );
    manager.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
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

pub(crate) fn format_world_message(message: WorldToOverlayMessage) -> String {
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

pub(crate) fn normalize_scene_label_alias(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "gameplay" => "gameplay_stub".to_string(),
        "hub" => "hub_stub".to_string(),
        "console" => "console_ui".to_string(),
        "hud" | "pause" => "hud_ui".to_string(),
        "inventory" | "inv" => "inventory_ui".to_string(),
        other => other.replace('-', "_"),
    }
}
