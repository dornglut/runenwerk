use editor_shell::{ShellCommand, ToolbarCommandKind};
use engine::plugins::render::backend::RenderSurfaceId;
use engine::runtime::NativeWindowId;
use runenwerk_editor::editor_app::RunenwerkEditorApp;
use runenwerk_editor::shell::{RunenwerkEditorShellState, dispatch_shell_command};

#[test]
fn window_new_toolbar_command_creates_logical_editor_window_without_native_handles() {
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

    assert_eq!(shell_state.editor_windows().len(), 2);
    assert_ne!(shell_state.editor_windows().active_window_id(), primary);
    assert_eq!(
        shell_state
            .editor_window_binding(primary)
            .map(|binding| { (binding.native_window_id, binding.render_surface_id) }),
        Some((NativeWindowId::primary(), RenderSurfaceId::primary()))
    );
    assert!(
        shell_state
            .editor_window_binding(shell_state.editor_windows().active_window_id())
            .is_none(),
        "native window and render surface handles are bound by the app/runtime after native creation"
    );
}
