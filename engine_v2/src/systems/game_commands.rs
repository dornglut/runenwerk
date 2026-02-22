use crate::runtime::EngineData;
use crate::ui::{UiDirty, UiSubmitEvent};
use ecs::EntityHandle;

const CONSOLE_PROMPT: &str = "grotto> ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    Help,
    Clear,
    Echo(String),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct GameCommandEvent {
    pub command: GameCommand,
}

fn drain_submit_events(data: &mut EngineData) -> Vec<(EntityHandle, String)> {
    let event_entities: Vec<EntityHandle> = data.world.entities_with::<UiSubmitEvent>().collect();
    let mut lines = Vec::with_capacity(event_entities.len());

    for entity in event_entities {
        if let Some(event) = data.world.get_component::<UiSubmitEvent>(entity) {
            lines.push((entity, event.line.clone()));
        }
    }

    lines
}

fn drain_game_command_events(data: &mut EngineData) -> Vec<(EntityHandle, GameCommand)> {
    let event_entities: Vec<EntityHandle> = data.world.entities_with::<GameCommandEvent>().collect();
    let mut commands = Vec::with_capacity(event_entities.len());

    for entity in event_entities {
        if let Some(event) = data.world.get_component::<GameCommandEvent>(entity) {
            commands.push((entity, event.command.clone()));
        }
    }

    commands
}

pub(super) fn parse_command_line(line: &str) -> Option<GameCommand> {
    let without_prompt = line.strip_prefix(CONSOLE_PROMPT).unwrap_or(line);
    let trimmed = without_prompt.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next().unwrap_or_default();
    let rest = parts.collect::<Vec<_>>().join(" ");

    match cmd {
        "help" => Some(GameCommand::Help),
        "clear" => Some(GameCommand::Clear),
        "echo" => Some(GameCommand::Echo(rest)),
        other => Some(GameCommand::Unknown(other.to_string())),
    }
}

pub(super) fn clamp_scrollback_lines(lines: &mut Vec<String>, max_lines: usize) {
    if max_lines == 0 {
        lines.clear();
        return;
    }
    let overflow = lines.len().saturating_sub(max_lines);
    if overflow > 0 {
        lines.drain(..overflow);
    }
}

pub(super) fn apply_game_command(lines: &mut Vec<String>, command: GameCommand) {
    match command {
        GameCommand::Help => {
            lines.push("commands: help, clear, echo <text>".to_string());
        }
        GameCommand::Clear => {
            lines.clear();
            lines.push("grotto> screen cleared".to_string());
        }
        GameCommand::Echo(text) => {
            lines.push(format!("grotto> {text}"));
        }
        GameCommand::Unknown(name) => {
            lines.push(format!("unknown command: {name}"));
            lines.push("type 'help' for available commands".to_string());
        }
    }
}

pub fn game_command_apply_system(data: &mut EngineData) -> anyhow::Result<()> {
    let events = drain_submit_events(data);
    if events.is_empty() {
        return Ok(());
    }

    for (entity, line) in events {
        if let Some(command) = parse_command_line(&line) {
            data.world.spawn_entity_typed(GameCommandEvent { command });
        }
        data.world.remove_entity(entity);
    }

    Ok(())
}

pub fn game_command_execute_system(data: &mut EngineData) -> anyhow::Result<()> {
    let events = drain_game_command_events(data);
    if events.is_empty() {
        return Ok(());
    }

    for (entity, command) in events {
        apply_game_command(&mut data.ui.lines, command);
        clamp_scrollback_lines(&mut data.ui.lines, data.ui.max_lines);
        data.world.remove_entity(entity);
    }
    data.ui.scroll_lines_from_bottom = 0;
    let scroll_entity = data.ui.scrollback;
    if let Some(dirty) = data.world.get_component_mut::<UiDirty>(scroll_entity) {
        dirty.text = true;
    }

    Ok(())
}
