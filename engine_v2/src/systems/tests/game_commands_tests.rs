use crate::systems::game_commands::{
    apply_game_command, clamp_scrollback_lines, parse_command_line, GameCommand,
};

#[test]
fn parse_command_line_recognizes_builtins() {
    assert_eq!(parse_command_line("grotto> help"), Some(GameCommand::Help));
    assert_eq!(parse_command_line("clear"), Some(GameCommand::Clear));
    assert_eq!(
        parse_command_line("grotto> echo hello world"),
        Some(GameCommand::Echo("hello world".to_string()))
    );
}

#[test]
fn parse_command_line_maps_unknown_command() {
    assert_eq!(
        parse_command_line("grotto> hello"),
        Some(GameCommand::Unknown("hello".to_string()))
    );
    assert_eq!(parse_command_line("grotto>   "), None);
}

#[test]
fn apply_game_command_updates_lines() {
    let mut lines = vec!["grotto> boot complete".to_string()];
    apply_game_command(&mut lines, GameCommand::Echo("test".to_string()));
    assert_eq!(lines.last(), Some(&"grotto> test".to_string()));

    apply_game_command(&mut lines, GameCommand::Help);
    assert_eq!(
        lines.last(),
        Some(&"commands: help, clear, echo <text>".to_string())
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
