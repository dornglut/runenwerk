use crate::editor_panels::ViewportToolState;
use editor_core::EntityId;
use editor_shell::ViewportViewModel;

pub fn build_viewport_view_model(
    selected_entity: Option<EntityId>,
    drag_in_progress: bool,
    tool_state: ViewportToolState,
) -> ViewportViewModel {
    ViewportViewModel {
        selected_entity,
        hovered_entity: tool_state.hovered_entity,
        drag_in_progress,
        preview_active: tool_state.active_preview.is_some(),
    }
}
