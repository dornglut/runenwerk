use ui_input::UiInputEvent;
use ui_math::UiRect;
use ui_runtime::UiInputOutcome;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_host::{HostEditorShellFrame, HostEditorShellInputBridge};
use crate::shell::RunenwerkEditorShellState;

pub struct HostEditorShellSession<'a> {
	app: &'a mut RunenwerkEditorApp,
	shell_state: &'a mut RunenwerkEditorShellState,
}

impl<'a> HostEditorShellSession<'a> {
	pub fn new(
		app: &'a mut RunenwerkEditorApp,
		shell_state: &'a mut RunenwerkEditorShellState,
	) -> Self {
		Self { app, shell_state }
	}

	pub fn build_frame(
		&mut self,
		bounds: UiRect,
		theme: &ThemeTokens,
	) -> HostEditorShellFrame {
		let frame = self.app.build_shell_frame(self.shell_state, bounds, theme);
		HostEditorShellFrame::new(bounds, frame)
	}

	pub fn dispatch_input(
		&mut self,
		bounds: UiRect,
		theme: &ThemeTokens,
		event: &UiInputEvent,
	) -> Result<UiInputOutcome, &'static str> {
		HostEditorShellInputBridge::dispatch(self.app, self.shell_state, bounds, theme, event)
	}

	pub fn app(&self) -> &RunenwerkEditorApp {
		self.app
	}

	pub fn app_mut(&mut self) -> &mut RunenwerkEditorApp {
		self.app
	}

	pub fn shell_state(&self) -> &RunenwerkEditorShellState {
		self.shell_state
	}

	pub fn shell_state_mut(&mut self) -> &mut RunenwerkEditorShellState {
		self.shell_state
	}
}