//! File: domain/ui/ui_runtime/src/state/runtime_state.rs
//! Purpose: Persistent runtime state across UI frames.

use crate::WidgetId;
use ui_input::FocusTargetId;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiRuntimeState {
	pub hovered_widget: Option<WidgetId>,
	pub pressed_widget: Option<WidgetId>,
	pub captured_widget: Option<WidgetId>,
	pub focused_target: Option<FocusTargetId>,
}

impl UiRuntimeState {
	pub fn clear_pointer_state(&mut self) {
		self.hovered_widget = None;
		self.pressed_widget = None;
		self.captured_widget = None;
	}
}