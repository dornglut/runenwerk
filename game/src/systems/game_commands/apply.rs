use super::spec::{GameCommand, command_registry, find_command_spec};
use engine::plugins::ui::domain::ConsoleUiState;

pub(crate) fn clamp_scrollback_lines(lines: &mut Vec<String>, max_lines: usize) {
    if max_lines == 0 {
        lines.clear();
        return;
    }
    let overflow = lines.len().saturating_sub(max_lines);
    if overflow > 0 {
        lines.drain(..overflow);
    }
}

pub(crate) fn flush_paused_logs(ui: &mut ConsoleUiState) {
    if ui.log_paused_lines.is_empty() {
        return;
    }
    ui.log_lines.append(&mut ui.log_paused_lines);
    clamp_scrollback_lines(&mut ui.log_lines, ui.max_lines);
    ui.log_scroll_lines_from_bottom = 0;
}

pub(crate) fn apply_game_command(lines: &mut Vec<String>, command: GameCommand) {
    match command {
        GameCommand::Help(target) => {
            if let Some(target) = target {
                if let Some(spec) = find_command_spec(&target) {
                    lines.push(format!("{} - {}", spec.usage, spec.summary));
                } else {
                    lines.push(format!("unknown command: {target}"));
                    lines.push("type 'help' for available commands".to_string());
                }
                return;
            }

            let names = command_registry()
                .iter()
                .map(|spec| spec.name)
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("commands: {names}"));
            lines.push("type 'help <command>' for details".to_string());
        }
        GameCommand::Clear => {
            lines.clear();
            lines.push("grotto> screen cleared".to_string());
        }
        GameCommand::Echo(text) => {
            lines.push(format!("grotto> {text}"));
        }
        GameCommand::History(count) => {
            let take = count.min(lines.len());
            lines.push(format!("history (last {take} lines):"));
            let start = lines.len().saturating_sub(1 + take);
            let snapshot = lines[start..lines.len() - 1].to_vec();
            for (idx, line) in snapshot.iter().enumerate() {
                lines.push(format!("{:>3}: {}", idx + 1, line));
            }
        }
        GameCommand::Count => {
            lines.push(format!("scrollback lines: {}", lines.len()));
        }
        GameCommand::SetWorld(scene) => {
            lines.push(format!("queued world switch: {scene}"));
        }
        GameCommand::SetScene(scene_id) => {
            lines.push(format!("queued scene switch: {scene_id}"));
        }
        GameCommand::PushOverlay(scene) => {
            lines.push(format!("queued overlay push: {scene}"));
        }
        GameCommand::PopOverlay => {
            lines.push("queued overlay pop".to_string());
        }
        GameCommand::PauseLogs => {
            lines.push("logs window paused".to_string());
        }
        GameCommand::ResumeLogs => {
            lines.push("logs window resumed".to_string());
        }
        GameCommand::ToggleLogs => {
            lines.push("logs window toggled".to_string());
        }
        GameCommand::FreezeTime => {
            lines.push("world simulation frozen".to_string());
        }
        GameCommand::ResumeTime => {
            lines.push("world simulation resumed".to_string());
        }
        GameCommand::ToggleTime => {
            lines.push("world simulation toggled".to_string());
        }
        GameCommand::Pipelines => {
            lines.push("pass slots: world_compute, world_compose, ui".to_string());
            lines.push(
                "pipeline keys: world_compute_basic, world_compute_high_contrast, world_compose_fullscreen, ui_composite_sdf"
                    .to_string(),
            );
        }
        GameCommand::SetPipeline { slot, key } => {
            lines.push(format!("set pipeline: {} -> {}", slot.label(), key.label()));
        }
        GameCommand::ReloadShaders => {
            lines.push("shader reload requested".to_string());
        }
        GameCommand::ShaderWatch(enabled) => {
            lines.push(format!(
                "shader_watch {}",
                if enabled { "on" } else { "off" }
            ));
        }
        GameCommand::ShaderStatus => {
            lines.push("shader status requested".to_string());
        }
        GameCommand::Models => {
            lines.push("model list requested".to_string());
        }
        GameCommand::ReloadModels => {
            lines.push("model reload requested".to_string());
        }
        GameCommand::ModelWatch(enabled) => {
            lines.push(format!(
                "model_watch {}",
                if enabled { "on" } else { "off" }
            ));
        }
        GameCommand::ModelStatus => {
            lines.push("model status requested".to_string());
        }
        GameCommand::Invalid(reason) => {
            lines.push(reason);
        }
        GameCommand::Unknown(name) => {
            lines.push(format!("unknown command: {name}"));
            lines.push("type 'help' for available commands".to_string());
        }
    }
}
