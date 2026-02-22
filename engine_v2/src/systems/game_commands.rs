use crate::runtime::EngineData;
use crate::ui::{UiDirty, UiSubmitEvent};
use ecs::EntityHandle;

const CONSOLE_PROMPT: &str = "grotto> ";
const DEFAULT_HISTORY_COUNT: usize = 10;
const MAX_HISTORY_COUNT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub usage: &'static str,
    pub summary: &'static str,
    pub aliases: &'static [&'static str],
}

const COMMAND_REGISTRY: [CommandSpec; 5] = [
    CommandSpec {
        name: "help",
        usage: "help [command]",
        summary: "show available commands or details for one command",
        aliases: &["?"],
    },
    CommandSpec {
        name: "clear",
        usage: "clear",
        summary: "clear the scrollback output",
        aliases: &["cls"],
    },
    CommandSpec {
        name: "echo",
        usage: "echo <text>",
        summary: "print text back to the console",
        aliases: &[],
    },
    CommandSpec {
        name: "history",
        usage: "history [count]",
        summary: "show recent scrollback lines",
        aliases: &["hist"],
    },
    CommandSpec {
        name: "count",
        usage: "count",
        summary: "show how many lines are in scrollback",
        aliases: &["lines"],
    },
];

pub(super) fn command_registry() -> &'static [CommandSpec] {
    &COMMAND_REGISTRY
}

fn find_command_spec(name: &str) -> Option<&'static CommandSpec> {
    command_registry()
        .iter()
        .find(|spec| spec.name == name || spec.aliases.iter().any(|alias| *alias == name))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    Help(Option<String>),
    Clear,
    Echo(String),
    History(usize),
    Count,
    Invalid(String),
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
    let event_entities: Vec<EntityHandle> =
        data.world.entities_with::<GameCommandEvent>().collect();
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
    let cmd = parts.next().unwrap_or_default().to_ascii_lowercase();
    let args: Vec<String> = parts.map(ToString::to_string).collect();
    let rest = args.join(" ");

    let Some(spec) = find_command_spec(&cmd) else {
        return Some(GameCommand::Unknown(cmd));
    };

    match spec.name {
        "help" => Some(GameCommand::Help(args.first().cloned())),
        "clear" => Some(GameCommand::Clear),
        "echo" => Some(GameCommand::Echo(rest)),
        "history" => {
            if let Some(raw) = args.first() {
                match raw.parse::<usize>() {
                    Ok(count) => Some(GameCommand::History(count.clamp(1, MAX_HISTORY_COUNT))),
                    Err(_) => Some(GameCommand::Invalid("usage: history [count]".to_string())),
                }
            } else {
                Some(GameCommand::History(DEFAULT_HISTORY_COUNT))
            }
        }
        "count" => Some(GameCommand::Count),
        _ => Some(GameCommand::Unknown(cmd)),
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
        GameCommand::Help(target) => {
            if let Some(target) = target {
                let key = target.to_ascii_lowercase();
                if let Some(spec) = find_command_spec(&key) {
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
        GameCommand::Invalid(reason) => {
            lines.push(reason);
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
