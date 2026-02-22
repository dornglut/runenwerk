use crate::systems::game_commands::{
    GameCommand, apply_game_command, clamp_scrollback_lines, command_registry, parse_command_line,
};

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
    assert_eq!(parse_command_line("grotto>   "), None);
}

#[test]
fn apply_game_command_updates_lines() {
    let mut lines = vec!["grotto> boot complete".to_string()];
    apply_game_command(&mut lines, GameCommand::Echo("test".to_string()));
    assert_eq!(lines.last(), Some(&"grotto> test".to_string()));

    apply_game_command(&mut lines, GameCommand::Help(None));
    assert_eq!(lines[2], "commands: help, clear, echo, history, count");
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
