use crate::editor_panels::ViewportToolState;
use editor_core::EntityId;
use editor_shell::{
    ObservationConsumerKind, ObservationFrameMetadata, ObservationSourceReality,
    ViewportObservationFrame, ViewportViewModel,
};

pub fn build_viewport_observation_frame(
    selected_entity: Option<EntityId>,
    drag_in_progress: bool,
    tool_state: ViewportToolState,
    source_version: editor_core::RealityVersion,
) -> ViewportObservationFrame {
    ViewportObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Viewport,
            source_version,
        ),
        selected_entity,
        hovered_entity: tool_state.hovered_entity,
        drag_in_progress,
        preview_active: tool_state.active_preview.is_some(),
    }
}

pub fn build_viewport_view_model(frame: &ViewportObservationFrame) -> ViewportViewModel {
    ViewportViewModel {
        selected_entity: frame.selected_entity,
        hovered_entity: frame.hovered_entity,
        drag_in_progress: frame.drag_in_progress,
        preview_active: frame.preview_active,
    }
}
