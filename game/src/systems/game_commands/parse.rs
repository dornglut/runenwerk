use super::spec::{
    CONSOLE_PROMPT, DEFAULT_HISTORY_COUNT, GameCommand, MAX_HISTORY_COUNT, find_command_spec,
};
use engine::plugins::render::domain::{PassSlot, PipelineKey};

pub(crate) fn parse_command_line(line: &str) -> Option<GameCommand> {
    let without_prompt = line.strip_prefix(CONSOLE_PROMPT).unwrap_or(line);
    let trimmed = without_prompt.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let raw_cmd = parts.next().unwrap_or_default().to_ascii_lowercase();
    let cmd = normalize_command_name(&raw_cmd);
    let args: Vec<String> = parts.map(ToString::to_string).collect();
    let rest = args.join(" ");

    let Some(spec) = find_command_spec(&cmd) else {
        return Some(GameCommand::Unknown(raw_cmd));
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
                "gameplay" => Some(GameCommand::SetWorld("gameplay_stub".to_string())),
                "hub" => Some(GameCommand::SetWorld("hub_stub".to_string())),
                _ => Some(GameCommand::Invalid(
                    "usage: set_world <gameplay|hub>".to_string(),
                )),
            }
        }
        "set_scene" => {
            let Some(target) = args.first() else {
                return Some(GameCommand::Invalid(
                    "usage: set_scene <scene_id>".to_string(),
                ));
            };
            Some(GameCommand::SetScene(normalize_scene_label(target)))
        }
        "push_overlay" => {
            let target = args
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|| "hud".to_string());
            let overlay = match target.as_str() {
                "console" => "console_ui",
                "inventory" | "inv" => "inventory_ui",
                "hud" | "pause" => "hud_ui",
                _ => {
                    return Some(GameCommand::Invalid(
                        "usage: push_overlay [console|hud|inventory|pause]".to_string(),
                    ));
                }
            };
            Some(GameCommand::PushOverlay(overlay.to_string()))
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
            match parse_toggle_value(mode) {
                Some(enabled) => Some(GameCommand::ShaderWatch(enabled)),
                None => Some(GameCommand::Invalid(
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
            match parse_toggle_value(mode) {
                Some(enabled) => Some(GameCommand::ModelWatch(enabled)),
                None => Some(GameCommand::Invalid(
                    "usage: model_watch <on|off>".to_string(),
                )),
            }
        }
        "model_status" => Some(GameCommand::ModelStatus),
        _ => Some(GameCommand::Unknown(raw_cmd)),
    }
}

fn normalize_command_name(raw: &str) -> String {
    raw.to_ascii_lowercase().replace('-', "_")
}

fn normalize_scene_label(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace('-', "_")
}

fn parse_toggle_value(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "on" | "true" | "1" | "yes" => Some(true),
        "off" | "false" | "0" | "no" => Some(false),
        _ => None,
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
