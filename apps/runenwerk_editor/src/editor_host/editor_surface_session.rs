use editor_viewport::ViewportHitResult;
use ui_input::UiInputEvent;
use ui_math::UiRect;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_host::{
	HostEditorShellSession, HostEditorSurfaceFrame, HostEditorSurfaceInput,
	HostEditorSurfaceInputRouter, HostEditorSurfaceLayout, HostViewportInput,
	HostViewportSession,
};
use crate::shell::RunenwerkEditorShellState;

#[derive(Debug, Clone, PartialEq)]
pub enum HostEditorSurfaceDispatchOutcome {
	Shell(ui_runtime::UiInputOutcome),
	Viewport,
	Ignored,
}

pub struct HostEditorSurfaceSession<'a> {
	app: &'a mut RunenwerkEditorApp,
	shell_state: &'a mut RunenwerkEditorShellState,
}

impl<'a> HostEditorSurfaceSession<'a> {
	pub fn new(
		app: &'a mut RunenwerkEditorApp,
		shell_state: &'a mut RunenwerkEditorShellState,
	) -> Self {
		Self { app, shell_state }
	}

	pub fn layout(
		&self,
		bounds: UiRect,
	) -> HostEditorSurfaceLayout {
		HostEditorSurfaceLayout::new(bounds)
	}

	pub fn build_frame(
		&mut self,
		bounds: UiRect,
		theme: &ThemeTokens,
	) -> HostEditorSurfaceFrame {
		let layout = self.layout(bounds);

		let shell = {
			let mut shell_session = HostEditorShellSession::new(self.app, self.shell_state);
			shell_session.build_frame(layout.shell_bounds(), theme)
		};

		let viewport = {
			let viewport_session = HostViewportSession::new(self.app);
			viewport_session.frame_state()
		};

		HostEditorSurfaceFrame::new(layout, shell, viewport)
	}

	pub fn dispatch_shell_input(
		&mut self,
		bounds: UiRect,
		theme: &ThemeTokens,
		event: &UiInputEvent,
	) -> Result<ui_runtime::UiInputOutcome, &'static str> {
		let layout = self.layout(bounds);
		let mut shell_session = HostEditorShellSession::new(self.app, self.shell_state);
		HostEditorSurfaceInputRouter::dispatch_shell(&mut shell_session, &layout, theme, event)
	}

	pub fn dispatch_viewport_hit(
		&mut self,
		hit: ViewportHitResult,
	) -> Result<(), &'static str> {
		let mut viewport_session = HostViewportSession::new(self.app);
		HostEditorSurfaceInputRouter::dispatch_viewport(
			&mut viewport_session,
			HostViewportInput::PointerDown { hit },
		)
	}

	pub fn dispatch_pointer_event(
		&mut self,
		bounds: UiRect,
		theme: &ThemeTokens,
		event: &UiInputEvent,
	) -> Result<HostEditorSurfaceDispatchOutcome, &'static str> {
		let layout = self.layout(bounds);

		let Some(routed) = HostEditorSurfaceInputRouter::route_pointer_event(&layout, event) else {
			return Ok(HostEditorSurfaceDispatchOutcome::Ignored);
		};

		match routed {
			HostEditorSurfaceInput::Shell(shell_event) => {
				let mut shell_session = HostEditorShellSession::new(self.app, self.shell_state);
				let outcome = HostEditorSurfaceInputRouter::dispatch_shell(
					&mut shell_session,
					&layout,
					theme,
					&shell_event,
				)?;
				Ok(HostEditorSurfaceDispatchOutcome::Shell(outcome))
			}
			HostEditorSurfaceInput::Viewport(viewport_input) => {
				let mut viewport_session = HostViewportSession::new(self.app);
				HostEditorSurfaceInputRouter::dispatch_viewport(
					&mut viewport_session,
					viewport_input,
				)?;
				Ok(HostEditorSurfaceDispatchOutcome::Viewport)
			}
		}
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