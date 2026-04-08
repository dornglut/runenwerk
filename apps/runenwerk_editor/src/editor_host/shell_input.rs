use ui_input::UiInputEvent;
use ui_math::UiRect;
use ui_runtime::UiInputOutcome;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::RunenwerkEditorShellState;

pub struct HostEditorShellInputBridge;

impl HostEditorShellInputBridge {
	pub fn dispatch(
		app: &mut RunenwerkEditorApp,
		shell_state: &mut RunenwerkEditorShellState,
		bounds: UiRect,
		theme: &ThemeTokens,
		event: &UiInputEvent,
	) -> Result<UiInputOutcome, &'static str> {
		app.dispatch_shell_input(shell_state, bounds, theme, event)
	}
}