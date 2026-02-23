use engine::plugins::render::domain::{PassSlot, PipelineKey};

pub(super) const CONSOLE_PROMPT: &str = "grotto> ";
pub(super) const DEFAULT_HISTORY_COUNT: usize = 10;
pub(super) const MAX_HISTORY_COUNT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub usage: &'static str,
    pub summary: &'static str,
    pub aliases: &'static [&'static str],
}

const COMMAND_REGISTRY: [CommandSpec; 24] = [
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
        name: "set_scene",
        usage: "set_scene <scene_id>",
        summary: "switch to a registered scene id (template flow) or a known built-in scene id",
        aliases: &["scene"],
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

pub(crate) fn command_registry() -> &'static [CommandSpec] {
    &COMMAND_REGISTRY
}

fn canonical_name(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace('-', "_")
}

pub(super) fn find_command_spec(name: &str) -> Option<&'static CommandSpec> {
    let key = canonical_name(name);
    command_registry().iter().find(|spec| {
        canonical_name(spec.name) == key
            || spec
                .aliases
                .iter()
                .any(|alias| canonical_name(alias) == key)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    Help(Option<String>),
    Clear,
    Echo(String),
    History(usize),
    Count,
    SetWorld(String),
    SetScene(String),
    PushOverlay(String),
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
