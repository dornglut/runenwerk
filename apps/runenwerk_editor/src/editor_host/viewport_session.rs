use crate::editor_app::RunenwerkEditorApp;
use crate::editor_host::{HostViewportFrameState, HostViewportInputBridge};
use crate::editor_panels::ViewportToolState;
use crate::editor_tools_state::TranslateAxis;
use editor_core::EntityId;
use editor_viewport::ViewportHitResult;

pub struct HostViewportSession<'a> {
	app: &'a mut RunenwerkEditorApp,
}

impl<'a> HostViewportSession<'a> {
	pub fn new(app: &'a mut RunenwerkEditorApp) -> Self {
		Self { app }
	}

	pub fn pointer_down(
		&mut self,
		hit: ViewportHitResult,
	) -> Result<(), &'static str> {
		HostViewportInputBridge::pointer_down(self.app, hit)
	}

	pub fn pointer_drag_axis(
		&mut self,
		amount: f32,
	) -> Result<(), &'static str> {
		HostViewportInputBridge::pointer_drag_axis(self.app, amount)
	}

	pub fn pointer_up(&mut self) -> Result<(), &'static str> {
		HostViewportInputBridge::pointer_up(self.app)
	}

	pub fn cancel(&mut self) -> Result<(), &'static str> {
		HostViewportInputBridge::cancel(self.app)
	}

	pub fn selected_entity(&self) -> Option<EntityId> {
		self.app.runtime().selected_entity()
	}

	pub fn drag_in_progress(&self) -> bool {
		self.app.viewport_interaction_state().drag_in_progress()
	}

	pub fn active_entity(&self) -> Option<EntityId> {
		self.app.viewport_interaction_state().active_entity()
	}

	pub fn active_axis(&self) -> Option<TranslateAxis> {
		self.app.viewport_interaction_state().active_axis()
	}

	pub fn tool_state(&self) -> ViewportToolState {
		self.app.viewport_tool_state()
	}

	pub fn frame_state(&self) -> HostViewportFrameState {
		HostViewportFrameState::from_session(self)
	}

	pub fn app(&self) -> &RunenwerkEditorApp {
		self.app
	}

	pub fn app_mut(&mut self) -> &mut RunenwerkEditorApp {
		self.app
	}
}