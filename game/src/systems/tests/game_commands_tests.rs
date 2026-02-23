use crate::systems::game_commands::{
    GameCommand, apply_game_command, clamp_scrollback_lines, command_registry, flush_paused_logs,
    parse_command_line,
};
use ecs::World;
use engine::plugins::render::domain::{PassSlot, PipelineKey};
use engine::plugins::scene::domain::SceneId;
use engine::plugins::ui::domain::initialize_console_ui;

#[test]
fn parse_command_line_recognizes_builtins() {
    assert_eq!(
        parse_command_line("grotto> help"),
        Some(GameCommand::Help(None))
    );
    assert_eq!(
        parse_command_line("grotto> help echo"),
        Some(GameCommand::Help(Some("echo".to_string())))
    );
    assert_eq!(parse_command_line("clear"), Some(GameCommand::Clear));
    assert_eq!(parse_command_line("cls"), Some(GameCommand::Clear));
    assert_eq!(
        parse_command_line("grotto> echo hello world"),
        Some(GameCommand::Echo("hello world".to_string()))
    );
    assert_eq!(
        parse_command_line("grotto> history"),
        Some(GameCommand::History(10))
    );
    assert_eq!(
        parse_command_line("grotto> hist 5"),
        Some(GameCommand::History(5))
    );
    assert_eq!(
        parse_command_line("grotto> lines"),
        Some(GameCommand::Count)
    );
    assert_eq!(
        parse_command_line("grotto> set_world gameplay"),
        Some(GameCommand::SetWorld(SceneId::GameplayStub))
    );
    assert_eq!(
        parse_command_line("grotto> set_world hub"),
        Some(GameCommand::SetWorld(SceneId::HubStub))
    );
    assert_eq!(
        parse_command_line("grotto> push_overlay pause"),
        Some(GameCommand::PushOverlay(SceneId::HudUi))
    );
    assert_eq!(
        parse_command_line("grotto> push_overlay inventory"),
        Some(GameCommand::PushOverlay(SceneId::InventoryUi))
    );
    assert_eq!(
        parse_command_line("grotto> pop_overlay"),
        Some(GameCommand::PopOverlay)
    );
    assert_eq!(
        parse_command_line("grotto> pause_logs"),
        Some(GameCommand::PauseLogs)
    );
    assert_eq!(
        parse_command_line("grotto> resume_logs"),
        Some(GameCommand::ResumeLogs)
    );
    assert_eq!(
        parse_command_line("grotto> toggle_logs"),
        Some(GameCommand::ToggleLogs)
    );
    assert_eq!(
        parse_command_line("grotto> freeze_time"),
        Some(GameCommand::FreezeTime)
    );
    assert_eq!(
        parse_command_line("grotto> resume_time"),
        Some(GameCommand::ResumeTime)
    );
    assert_eq!(
        parse_command_line("grotto> toggle_time"),
        Some(GameCommand::ToggleTime)
    );
    assert_eq!(
        parse_command_line("grotto> pipelines"),
        Some(GameCommand::Pipelines)
    );
    assert_eq!(
        parse_command_line("grotto> set_pipeline world_compute world_compute_high_contrast"),
        Some(GameCommand::SetPipeline {
            slot: PassSlot::WorldCompute,
            key: PipelineKey::WorldComputeHighContrast,
        })
    );
    assert_eq!(
        parse_command_line("grotto> reload_shaders"),
        Some(GameCommand::ReloadShaders)
    );
    assert_eq!(
        parse_command_line("grotto> shader_watch on"),
        Some(GameCommand::ShaderWatch(true))
    );
    assert_eq!(
        parse_command_line("grotto> shader_status"),
        Some(GameCommand::ShaderStatus)
    );
    assert_eq!(
        parse_command_line("grotto> models"),
        Some(GameCommand::Models)
    );
    assert_eq!(
        parse_command_line("grotto> reload_models"),
        Some(GameCommand::ReloadModels)
    );
    assert_eq!(
        parse_command_line("grotto> model_watch off"),
        Some(GameCommand::ModelWatch(false))
    );
    assert_eq!(
        parse_command_line("grotto> model_status"),
        Some(GameCommand::ModelStatus)
    );
}

#[test]
fn parse_command_line_maps_unknown_command() {
    assert_eq!(
        parse_command_line("grotto> hello"),
        Some(GameCommand::Unknown("hello".to_string()))
    );
    assert_eq!(
        parse_command_line("grotto> history nope"),
        Some(GameCommand::Invalid("usage: history [count]".to_string()))
    );
    assert_eq!(
        parse_command_line("grotto> set_world nope"),
        Some(GameCommand::Invalid(
            "usage: set_world <gameplay|hub>".to_string()
        ))
    );
    assert_eq!(
        parse_command_line("grotto> shader_watch maybe"),
        Some(GameCommand::Invalid(
            "usage: shader_watch <on|off>".to_string()
        ))
    );
    assert_eq!(
        parse_command_line("grotto> model_watch maybe"),
        Some(GameCommand::Invalid(
            "usage: model_watch <on|off>".to_string()
        ))
    );
    assert_eq!(parse_command_line("grotto>   "), None);
}

#[test]
fn apply_game_command_updates_lines() {
    let mut lines = vec!["grotto> boot complete".to_string()];
    apply_game_command(&mut lines, GameCommand::Echo("test".to_string()));
    assert_eq!(lines.last(), Some(&"grotto> test".to_string()));

    apply_game_command(&mut lines, GameCommand::Help(None));
    assert_eq!(
        lines[2],
        "commands: help, clear, echo, history, count, set_world, push_overlay, pop_overlay, pause_logs, resume_logs, toggle_logs, freeze_time, resume_time, toggle_time, pipelines, set_pipeline, reload_shaders, shader_watch, shader_status, models, reload_models, model_watch, model_status"
    );
    assert_eq!(lines[3], "type 'help <command>' for details");

    apply_game_command(&mut lines, GameCommand::Help(Some("echo".to_string())));
    assert_eq!(
        lines.last(),
        Some(&"echo <text> - print text back to the console".to_string())
    );

    apply_game_command(&mut lines, GameCommand::Clear);
    assert_eq!(lines, vec!["grotto> screen cleared".to_string()]);
}

#[test]
fn apply_game_command_unknown_adds_feedback() {
    let mut lines = vec!["grotto> boot complete".to_string()];
    apply_game_command(&mut lines, GameCommand::Unknown("oops".to_string()));
    assert_eq!(lines[1], "unknown command: oops");
    assert_eq!(lines[2], "type 'help' for available commands");
}

#[test]
fn clamp_scrollback_lines_keeps_recent_tail() {
    let mut lines = vec![
        "l1".to_string(),
        "l2".to_string(),
        "l3".to_string(),
        "l4".to_string(),
    ];
    clamp_scrollback_lines(&mut lines, 2);
    assert_eq!(lines, vec!["l3".to_string(), "l4".to_string()]);
}

#[test]
fn command_registry_exposes_expected_core_commands() {
    let names = command_registry()
        .iter()
        .map(|spec| spec.name)
        .collect::<Vec<_>>();
    assert!(names.contains(&"help"));
    assert!(names.contains(&"clear"));
    assert!(names.contains(&"echo"));
    assert!(names.contains(&"history"));
    assert!(names.contains(&"count"));
    assert!(names.contains(&"set_world"));
    assert!(names.contains(&"push_overlay"));
    assert!(names.contains(&"pop_overlay"));
    assert!(names.contains(&"pause_logs"));
    assert!(names.contains(&"resume_logs"));
    assert!(names.contains(&"toggle_logs"));
    assert!(names.contains(&"freeze_time"));
    assert!(names.contains(&"resume_time"));
    assert!(names.contains(&"toggle_time"));
    assert!(names.contains(&"pipelines"));
    assert!(names.contains(&"set_pipeline"));
    assert!(names.contains(&"reload_shaders"));
    assert!(names.contains(&"shader_watch"));
    assert!(names.contains(&"shader_status"));
    assert!(names.contains(&"models"));
    assert!(names.contains(&"reload_models"));
    assert!(names.contains(&"model_watch"));
    assert!(names.contains(&"model_status"));
}

#[test]
fn history_and_count_commands_generate_output() {
    let mut lines = vec![
        "grotto> boot complete".to_string(),
        "grotto> map loaded".to_string(),
        "grotto> ready".to_string(),
    ];

    apply_game_command(&mut lines, GameCommand::Count);
    assert_eq!(lines.last(), Some(&"scrollback lines: 3".to_string()));

    apply_game_command(&mut lines, GameCommand::History(2));
    assert!(lines.iter().any(|line| line == "history (last 2 lines):"));
}

#[test]
fn flush_paused_logs_moves_buffer_into_log_lines() {
    let mut world = World::new();
    let mut ui = initialize_console_ui(&mut world);
    ui.log_lines = vec!["[world] boot".to_string()];
    ui.log_paused_lines = vec!["[combat] hit".to_string(), "[loot] +1".to_string()];
    ui.logs_paused = false;

    flush_paused_logs(&mut ui);

    assert_eq!(ui.log_paused_lines.len(), 0);
    assert_eq!(ui.log_lines.len(), 3);
    assert_eq!(ui.log_lines[1], "[combat] hit");
    assert_eq!(ui.log_lines[2], "[loot] +1");
    assert_eq!(ui.log_scroll_lines_from_bottom, 0);
}
