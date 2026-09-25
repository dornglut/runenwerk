use editor_shell::{ShellCommand, ToolbarCommandKind};
use runenwerk_editor::editor_app::RunenwerkEditorApp;
use runenwerk_editor::shell::{RunenwerkEditorShellState, dispatch_shell_command};

#[test]
fn window_new_toolbar_command_queues_composition_owned_fresh_target_without_early_window_mutation()
{
    let mut app = RunenwerkEditorApp::new();
    let mut shell_state = RunenwerkEditorShellState::new();
    let primary = shell_state.editor_windows().primary_window_id();

    dispatch_shell_command(
        &mut app,
        Some(&mut shell_state),
        ShellCommand::RunToolbarCommand {
            command: ToolbarCommandKind::NewWindow,
        },
        None,
        None,
        None,
        None,
    )
    .expect("new window toolbar command should be accepted");

    assert_eq!(
        shell_state.editor_windows().len(),
        1,
        "shell dispatch must not create a logical/native presentation before transition coordination"
    );
    assert_eq!(shell_state.editor_windows().active_window_id(), primary);
    assert!(shell_state.has_pending_fresh_target_request());
    assert!(shell_state.composition_coordination_pending());
}
