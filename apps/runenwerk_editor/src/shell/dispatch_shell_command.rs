use editor_shell::ShellCommand;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{SELECT_TOOL_ID, TRANSLATE_TOOL_ID};

pub fn dispatch_shell_command(
	app: &mut RunenwerkEditorApp,
	command: ShellCommand,
) -> Result<(), &'static str> {
	match command {
		ShellCommand::ActivateSelectTool => {
			app.runtime_mut()
				.session_mut()
				.set_active_tool(Some(SELECT_TOOL_ID));
		}
		ShellCommand::ActivateTranslateTool => {
			app.runtime_mut()
				.session_mut()
				.set_active_tool(Some(TRANSLATE_TOOL_ID));
		}
		ShellCommand::NoOp => {}
	}

	Ok(())
}