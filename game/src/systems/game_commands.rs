use engine::render::{PassSlot, PipelineKey};
use engine::runtime::{
    EngineData, OverlayCommandInput, OverlaySubmitMessage, SceneCommand, SceneId,
};
use engine::ui::UiDirty;

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

const COMMAND_REGISTRY: [CommandSpec; 23] = [
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
    CommandSpec {
        name: "set_world",
        usage: "set_world <gameplay|hub>",
        summary: "switch active world scene",
        aliases: &["world"],
    },
    CommandSpec {
        name: "push_overlay",
        usage: "push_overlay [console|hud|inventory|pause]",
        summary: "push an overlay scene onto the stack",
        aliases: &["overlay_push"],
    },
    CommandSpec {
        name: "pop_overlay",
        usage: "pop_overlay",
        summary: "pop the active overlay scene",
        aliases: &["overlay_pop"],
    },
    CommandSpec {
        name: "pause_logs",
        usage: "pause_logs",
        summary: "pause the separate logs window",
        aliases: &["logs_pause"],
    },
    CommandSpec {
        name: "resume_logs",
        usage: "resume_logs",
        summary: "resume the logs window and flush buffered messages",
        aliases: &["logs_resume"],
    },
    CommandSpec {
        name: "toggle_logs",
        usage: "toggle_logs",
        summary: "toggle logs window pause state",
        aliases: &["logs_toggle"],
    },
    CommandSpec {
        name: "freeze_time",
        usage: "freeze_time",
        summary: "freeze world simulation time",
        aliases: &["pause_time"],
    },
    CommandSpec {
        name: "resume_time",
        usage: "resume_time",
        summary: "resume world simulation time",
        aliases: &["unpause_time"],
    },
    CommandSpec {
        name: "toggle_time",
        usage: "toggle_time",
        summary: "toggle world simulation pause state",
        aliases: &[],
    },
    CommandSpec {
        name: "pipelines",
        usage: "pipelines",
        summary: "list available render/compute pass slots and pipeline keys",
        aliases: &["list_pipelines"],
    },
    CommandSpec {
        name: "set_pipeline",
        usage: "set_pipeline <world_compute|world_compose|ui> <pipeline_key>",
        summary: "switch the active pipeline used by a pass slot",
        aliases: &["pipeline_set"],
    },
    CommandSpec {
        name: "reload_shaders",
        usage: "reload_shaders",
        summary: "force reload all file-backed shaders",
        aliases: &["shader_reload"],
    },
    CommandSpec {
        name: "shader_watch",
        usage: "shader_watch <on|off>",
        summary: "enable or disable automatic shader file hot-reload polling",
        aliases: &[],
    },
    CommandSpec {
        name: "shader_status",
        usage: "shader_status",
        summary: "show loaded shader revisions and fallback/error state",
        aliases: &[],
    },
    CommandSpec {
        name: "models",
        usage: "models",
        summary: "list hot-reloaded model assets and proxy stats",
        aliases: &["list_models"],
    },
    CommandSpec {
        name: "reload_models",
        usage: "reload_models",
        summary: "force reload all file-backed .glb models",
        aliases: &["model_reload"],
    },
    CommandSpec {
        name: "model_watch",
        usage: "model_watch <on|off>",
        summary: "enable or disable automatic model file hot-reload polling",
        aliases: &[],
    },
    CommandSpec {
        name: "model_status",
        usage: "model_status",
        summary: "show loaded model revisions and parse/error state",
        aliases: &[],
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
    SetWorld(SceneId),
    PushOverlay(SceneId),
    PopOverlay,
    PauseLogs,
    ResumeLogs,
    ToggleLogs,
    FreezeTime,
    ResumeTime,
    ToggleTime,
    Pipelines,
    SetPipeline { slot: PassSlot, key: PipelineKey },
    ReloadShaders,
    ShaderWatch(bool),
    ShaderStatus,
    Models,
    ReloadModels,
    ModelWatch(bool),
    ModelStatus,
    Invalid(String),
    Unknown(String),
}

pub(super) fn flush_paused_logs(ui: &mut engine::ui::ConsoleUiState) {
    if ui.log_paused_lines.is_empty() {
        return;
    }
    ui.log_lines.append(&mut ui.log_paused_lines);
    clamp_scrollback_lines(&mut ui.log_lines, ui.max_lines);
    ui.log_scroll_lines_from_bottom = 0;
}

fn drain_submit_lines(data: &mut EngineData) -> Vec<OverlaySubmitMessage> {
    std::mem::take(&mut data.scene.channels.overlay_submit)
}

fn drain_command_inputs(data: &mut EngineData) -> Vec<OverlayCommandInput> {
    std::mem::take(&mut data.scene.channels.overlay_command_inputs)
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
        "set_world" => {
            let Some(target) = args.first() else {
                return Some(GameCommand::Invalid(
                    "usage: set_world <gameplay|hub>".to_string(),
                ));
            };
            let target = target.to_ascii_lowercase();
            match target.as_str() {
                "gameplay" => Some(GameCommand::SetWorld(SceneId::GameplayStub)),
                "hub" => Some(GameCommand::SetWorld(SceneId::HubStub)),
                _ => Some(GameCommand::Invalid(
                    "usage: set_world <gameplay|hub>".to_string(),
                )),
            }
        }
        "push_overlay" => {
            let target = args
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|| "hud".to_string());
            let overlay = match target.as_str() {
                "console" => SceneId::ConsoleUi,
                "inventory" | "inv" => SceneId::InventoryUi,
                "hud" | "pause" => SceneId::HudUi,
                _ => {
                    return Some(GameCommand::Invalid(
                        "usage: push_overlay [console|hud|inventory|pause]".to_string(),
                    ));
                }
            };
            Some(GameCommand::PushOverlay(overlay))
        }
        "pop_overlay" => Some(GameCommand::PopOverlay),
        "pause_logs" => Some(GameCommand::PauseLogs),
        "resume_logs" => Some(GameCommand::ResumeLogs),
        "toggle_logs" => Some(GameCommand::ToggleLogs),
        "freeze_time" => Some(GameCommand::FreezeTime),
        "resume_time" => Some(GameCommand::ResumeTime),
        "toggle_time" => Some(GameCommand::ToggleTime),
        "pipelines" => Some(GameCommand::Pipelines),
        "set_pipeline" => {
            let Some(slot_raw) = args.first() else {
                return Some(GameCommand::Invalid(
                    "usage: set_pipeline <world_compute|world_compose|ui> <pipeline_key>"
                        .to_string(),
                ));
            };
            let Some(key_raw) = args.get(1) else {
                return Some(GameCommand::Invalid(
                    "usage: set_pipeline <world_compute|world_compose|ui> <pipeline_key>"
                        .to_string(),
                ));
            };
            let Some(slot) = parse_pipeline_slot(slot_raw) else {
                return Some(GameCommand::Invalid(
                    "invalid pass slot (use: world_compute, world_compose, ui)".to_string(),
                ));
            };
            let Some(key) = parse_pipeline_key(key_raw) else {
                return Some(GameCommand::Invalid(
                    "invalid pipeline key (use 'pipelines' to list options)".to_string(),
                ));
            };
            Some(GameCommand::SetPipeline { slot, key })
        }
        "reload_shaders" => Some(GameCommand::ReloadShaders),
        "shader_watch" => {
            let Some(mode) = args.first() else {
                return Some(GameCommand::Invalid(
                    "usage: shader_watch <on|off>".to_string(),
                ));
            };
            match mode.to_ascii_lowercase().as_str() {
                "on" => Some(GameCommand::ShaderWatch(true)),
                "off" => Some(GameCommand::ShaderWatch(false)),
                _ => Some(GameCommand::Invalid(
                    "usage: shader_watch <on|off>".to_string(),
                )),
            }
        }
        "shader_status" => Some(GameCommand::ShaderStatus),
        "models" => Some(GameCommand::Models),
        "reload_models" => Some(GameCommand::ReloadModels),
        "model_watch" => {
            let Some(mode) = args.first() else {
                return Some(GameCommand::Invalid(
                    "usage: model_watch <on|off>".to_string(),
                ));
            };
            match mode.to_ascii_lowercase().as_str() {
                "on" => Some(GameCommand::ModelWatch(true)),
                "off" => Some(GameCommand::ModelWatch(false)),
                _ => Some(GameCommand::Invalid(
                    "usage: model_watch <on|off>".to_string(),
                )),
            }
        }
        "model_status" => Some(GameCommand::ModelStatus),
        _ => Some(GameCommand::Unknown(cmd)),
    }
}

fn parse_pipeline_slot(raw: &str) -> Option<PassSlot> {
    match raw.to_ascii_lowercase().as_str() {
        "world_compute" | "compute" => Some(PassSlot::WorldCompute),
        "world_compose" | "compose" => Some(PassSlot::WorldCompose),
        "ui" | "ui_composite" => Some(PassSlot::UiComposite),
        _ => None,
    }
}

fn parse_pipeline_key(raw: &str) -> Option<PipelineKey> {
    match raw.to_ascii_lowercase().as_str() {
        "world_compute_basic" | "compute_basic" | "basic" => Some(PipelineKey::WorldComputeBasic),
        "world_compute_high_contrast" | "compute_high_contrast" | "high_contrast" | "contrast" => {
            Some(PipelineKey::WorldComputeHighContrast)
        }
        "world_compose_fullscreen" | "compose_fullscreen" | "fullscreen" => {
            Some(PipelineKey::WorldComposeFullscreen)
        }
        "ui_composite_sdf" | "ui_sdf" | "sdf" => Some(PipelineKey::UiCompositeSdf),
        _ => None,
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
        GameCommand::SetWorld(scene) => {
            lines.push(format!("queued world switch: {}", scene.label()));
        }
        GameCommand::PushOverlay(scene) => {
            lines.push(format!("queued overlay push: {}", scene.label()));
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

pub fn game_command_apply_system(data: &mut EngineData) -> anyhow::Result<()> {
    let lines = drain_submit_lines(data);
    if lines.is_empty() {
        return Ok(());
    }

    data.scene
        .channels
        .overlay_command_inputs
        .extend(lines.into_iter().map(|submit| match submit {
            OverlaySubmitMessage::Line(line) => OverlayCommandInput::Line(line),
        }));

    Ok(())
}

pub fn game_command_execute_system(data: &mut EngineData) -> anyhow::Result<()> {
    let inputs = drain_command_inputs(data);
    if inputs.is_empty() {
        return Ok(());
    }

    for input in inputs {
        let OverlayCommandInput::Line(line) = input;
        if let Some(command) = parse_command_line(&line) {
            let mut command_to_apply = command.clone();
            match &command {
                GameCommand::SetWorld(scene) => {
                    data.scene.queue(SceneCommand::ReplaceWorld(*scene));
                }
                GameCommand::PushOverlay(scene) => {
                    data.scene.queue(SceneCommand::PushOverlay(*scene));
                    data.scene.queue(SceneCommand::PauseWorld(true));
                }
                GameCommand::PopOverlay => {
                    data.scene.queue(SceneCommand::PopOverlay);
                    data.scene.queue(SceneCommand::PauseWorld(false));
                }
                GameCommand::PauseLogs => {
                    data.scene.overlay_runtime.ui.logs_paused = true;
                }
                GameCommand::ResumeLogs => {
                    data.scene.overlay_runtime.ui.logs_paused = false;
                    flush_paused_logs(&mut data.scene.overlay_runtime.ui);
                }
                GameCommand::ToggleLogs => {
                    data.scene.overlay_runtime.ui.logs_paused =
                        !data.scene.overlay_runtime.ui.logs_paused;
                    if !data.scene.overlay_runtime.ui.logs_paused {
                        flush_paused_logs(&mut data.scene.overlay_runtime.ui);
                    }
                }
                GameCommand::FreezeTime => {
                    data.scene.queue(SceneCommand::PauseWorld(true));
                }
                GameCommand::ResumeTime => {
                    data.scene.queue(SceneCommand::PauseWorld(false));
                }
                GameCommand::ToggleTime => {
                    data.scene
                        .queue(SceneCommand::PauseWorld(!data.scene.world.paused));
                }
                GameCommand::SetPipeline { slot, key } => {
                    if let Err(err) = data.gfx.set_pipeline_for_slot(*slot, *key) {
                        command_to_apply =
                            GameCommand::Invalid(format!("set_pipeline failed: {err}"));
                    } else {
                        data.scene.overlay_runtime.ui.editor.status =
                            format!("editor: pipeline {} -> {}", slot.label(), key.label());
                    }
                }
                GameCommand::ReloadShaders => {
                    for msg in data.gfx.force_shader_reload() {
                        data.scene
                            .overlay_runtime
                            .ui
                            .lines
                            .push(format!("[world] {msg}"));
                    }
                }
                GameCommand::ShaderWatch(enabled) => {
                    data.gfx.set_shader_watch_enabled(*enabled);
                }
                GameCommand::ShaderStatus => {
                    for line in data.gfx.shader_status_lines() {
                        data.scene.overlay_runtime.ui.lines.push(line);
                    }
                }
                GameCommand::Models | GameCommand::ModelStatus => {
                    for line in data.gfx.model_status_lines() {
                        data.scene.overlay_runtime.ui.lines.push(line);
                    }
                }
                GameCommand::ReloadModels => {
                    for msg in data.gfx.force_model_reload() {
                        data.scene
                            .overlay_runtime
                            .ui
                            .lines
                            .push(format!("[world] {msg}"));
                    }
                }
                GameCommand::ModelWatch(enabled) => {
                    data.gfx.set_model_watch_enabled(*enabled);
                }
                _ => {}
            }
            apply_game_command(&mut data.scene.overlay_runtime.ui.lines, command_to_apply);
            clamp_scrollback_lines(
                &mut data.scene.overlay_runtime.ui.lines,
                data.scene.overlay_runtime.ui.max_lines,
            );
        }
    }
    data.scene.overlay_runtime.ui.scroll_lines_from_bottom = 0;
    let scroll_entity = data.scene.overlay_runtime.ui.scrollback;
    if let Some(dirty) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiDirty>(scroll_entity)
    {
        dirty.text = true;
    }

    Ok(())
}
