use editor_shell::ViewportViewModel;

use crate::editor_host::HostViewportFrameState;

pub fn build_viewport_view_model(
	frame: &HostViewportFrameState,
) -> ViewportViewModel {
	ViewportViewModel {
		selected_entity: frame.selected_entity,
		hovered_entity: frame.hovered_entity,
		drag_in_progress: frame.drag_in_progress,
		preview_active: frame.active_preview.is_some(),
	}
}