use crate::editor_host::HostViewportSession;
use crate::editor_panels::{ViewportPreviewState, ViewportToolState};
use crate::editor_tools_state::TranslateAxis;
use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq)]
pub struct HostViewportFrameState {
	pub selected_entity: Option<EntityId>,
	pub drag_in_progress: bool,
	pub active_entity: Option<EntityId>,
	pub active_axis: Option<TranslateAxis>,
	pub hovered_entity: Option<EntityId>,
	pub active_preview: Option<ViewportPreviewState>,
	pub active_translate_axis: Option<TranslateAxis>,
}

impl HostViewportFrameState {
	pub fn from_session(session: &HostViewportSession<'_>) -> Self {
		let tool_state = session.tool_state();

		Self::from_parts(
			session.selected_entity(),
			session.drag_in_progress(),
			session.active_entity(),
			session.active_axis(),
			tool_state,
		)
	}

	pub fn from_parts(
		selected_entity: Option<EntityId>,
		drag_in_progress: bool,
		active_entity: Option<EntityId>,
		active_axis: Option<TranslateAxis>,
		tool_state: ViewportToolState,
	) -> Self {
		Self {
			selected_entity,
			drag_in_progress,
			active_entity,
			active_axis,
			hovered_entity: tool_state.hovered_entity,
			active_preview: tool_state.active_preview,
			active_translate_axis: tool_state.active_translate_axis,
		}
	}
}