use editor_core::{EntityId, SelectionTarget};
use editor_shell::ShellCommand;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{
	build_editor_shell_view_model, dispatch_shell_command, SELECT_TOOL_ID, TRANSLATE_TOOL_ID,
};

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct TestMarker;

#[test]
fn build_editor_shell_view_model_reflects_current_outliner_selection() {
	let mut app = RunenwerkEditorApp::new();

	let ecs_entity = app.runtime_mut().world_mut().spawn(TestMarker);

	app.runtime_mut()
		.ids_mut()
		.register_entity(EntityId(1), ecs_entity, "Player", None);

	app.runtime_mut()
		.session_mut()
		.select_single(SelectionTarget::Entity(EntityId(1)));

	let shell = build_editor_shell_view_model(&app);

	assert_eq!(shell.outliner.rows.len(), 1);
	assert_eq!(shell.outliner.rows[0].entity, EntityId(1));
	assert!(shell.outliner.rows[0].is_selected);
}

#[test]
fn build_editor_shell_view_model_reflects_active_tool_and_viewport_state() {
	let mut app = RunenwerkEditorApp::new();

	app.runtime_mut()
		.session_mut()
		.set_active_tool(Some(TRANSLATE_TOOL_ID));

	app.tool_runtime_state_mut()
		.set_hovered_entity(Some(EntityId(7)));

	let shell = build_editor_shell_view_model(&app);

	assert_eq!(shell.toolbar.buttons.len(), 2);
	assert!(shell.toolbar.buttons.iter().any(|button| {
		button.id == TRANSLATE_TOOL_ID && button.is_active
	}));
	assert_eq!(shell.viewport.hovered_entity, Some(EntityId(7)));
	assert!(!shell.viewport.preview_active);
}

#[test]
fn dispatch_shell_command_updates_active_tool() {
	let mut app = RunenwerkEditorApp::new();

	dispatch_shell_command(&mut app, ShellCommand::ActivateSelectTool)
		.expect("select tool command should succeed");
	assert_eq!(app.runtime().session().active_tool(), Some(SELECT_TOOL_ID));

	dispatch_shell_command(&mut app, ShellCommand::ActivateTranslateTool)
		.expect("translate tool command should succeed");
	assert_eq!(app.runtime().session().active_tool(), Some(TRANSLATE_TOOL_ID));
}