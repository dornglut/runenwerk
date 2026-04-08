use ui_input::{PointerEventKind, UiInputEvent};

use crate::editor_host::{
	HostEditorShellSession, HostEditorSurfaceLayout, HostViewportInput, HostViewportSession,
};

#[derive(Debug, Clone, PartialEq)]
pub enum HostEditorSurfaceInput {
	Shell(UiInputEvent),
	Viewport(HostViewportInput),
}

pub struct HostEditorSurfaceInputRouter;

impl HostEditorSurfaceInputRouter {
	pub fn route_pointer_event(
		layout: &HostEditorSurfaceLayout,
		event: &UiInputEvent,
	) -> Option<HostEditorSurfaceInput> {
		let UiInputEvent::Pointer(pointer) = event else {
			return Some(HostEditorSurfaceInput::Shell(event.clone()));
		};

		let position = pointer.position;

		if layout.contains_viewport_point(position.x, position.y) {
			match pointer.kind {
				PointerEventKind::Up => {
					return Some(HostEditorSurfaceInput::Viewport(HostViewportInput::PointerUp));
				}
				PointerEventKind::Leave => {
					return Some(HostEditorSurfaceInput::Viewport(
						HostViewportInput::CancelInteraction,
					));
				}
				_ => {}
			}
		}

		if layout.contains_shell_point(position.x, position.y) {
			return Some(HostEditorSurfaceInput::Shell(event.clone()));
		}

		None
	}

	pub fn dispatch_shell(
		shell_session: &mut HostEditorShellSession<'_>,
		layout: &HostEditorSurfaceLayout,
		theme: &ui_theme::ThemeTokens,
		event: &UiInputEvent,
	) -> Result<ui_runtime::UiInputOutcome, &'static str> {
		shell_session.dispatch_input(layout.shell_bounds(), theme, event)
	}

	pub fn dispatch_viewport(
		viewport_session: &mut HostViewportSession<'_>,
		input: HostViewportInput,
	) -> Result<(), &'static str> {
		match input {
			HostViewportInput::PointerDown { hit } => viewport_session.pointer_down(hit),
			HostViewportInput::PointerDragAxis { amount } => {
				viewport_session.pointer_drag_axis(amount)
			}
			HostViewportInput::PointerUp => viewport_session.pointer_up(),
			HostViewportInput::CancelInteraction => viewport_session.cancel(),
		}
	}
}