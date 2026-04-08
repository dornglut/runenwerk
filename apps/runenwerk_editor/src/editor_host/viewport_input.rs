use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::viewport::ViewportInteractionCommand;
use editor_viewport::ViewportHitResult;

#[derive(Debug, Clone, PartialEq)]
pub enum HostViewportInput {
	PointerDown { hit: ViewportHitResult },
	PointerDragAxis { amount: f32 },
	PointerUp,
	CancelInteraction,
}

pub struct HostViewportInputBridge;

impl HostViewportInputBridge {
	pub fn dispatch(
		app: &mut RunenwerkEditorApp,
		input: HostViewportInput,
	) -> Result<(), &'static str> {
		match input {
			HostViewportInput::PointerDown { hit } => {
				app.dispatch_viewport_interaction_command(
					ViewportInteractionCommand::PointerDown { hit },
				)
			}
			HostViewportInput::PointerDragAxis { amount } => {
				app.dispatch_viewport_interaction_command(
					ViewportInteractionCommand::PointerDragAxis { amount },
				)
			}
			HostViewportInput::PointerUp => {
				app.dispatch_viewport_interaction_command(
					ViewportInteractionCommand::PointerUp,
				)
			}
			HostViewportInput::CancelInteraction => {
				app.cancel_viewport_interaction()
			}
		}
	}

	pub fn pointer_down(
		app: &mut RunenwerkEditorApp,
		hit: ViewportHitResult,
	) -> Result<(), &'static str> {
		Self::dispatch(app, HostViewportInput::PointerDown { hit })
	}

	pub fn pointer_drag_axis(
		app: &mut RunenwerkEditorApp,
		amount: f32,
	) -> Result<(), &'static str> {
		Self::dispatch(app, HostViewportInput::PointerDragAxis { amount })
	}

	pub fn pointer_up(app: &mut RunenwerkEditorApp) -> Result<(), &'static str> {
		Self::dispatch(app, HostViewportInput::PointerUp)
	}

	pub fn cancel(app: &mut RunenwerkEditorApp) -> Result<(), &'static str> {
		Self::dispatch(app, HostViewportInput::CancelInteraction)
	}
}