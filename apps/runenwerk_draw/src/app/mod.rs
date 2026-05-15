//! Drawing app product state.

pub mod document_factory;
pub mod ink;
pub mod input;
pub mod presentation;
pub mod state;
pub mod workspace;

pub use document_factory::minimal_drawing_document;
pub use ink::{
    DrawingInkGpuValidationEntry, DrawingInkGpuValidationMetrics, DrawingInkGpuValidationStatus,
    DrawingInkJournalEntry, DrawingInkJournalStage, DrawingInkRuntimeState,
};
pub use input::{DrawingPreviewStroke, DrawingToolInputEvent, DrawingToolRouteKind};
pub(crate) use presentation::build_workspace_frame_with_ink_surface_refs_and_stroke;
pub use presentation::{
    DRAWING_CANVAS_LAYER_ID, DRAWING_INK_TEXTURE_NAMESPACE, DRAWING_UI_SURFACE_ID,
    DrawingImmediateStrokeProjection, DrawingInkSurfaceKind, DrawingInkSurfaceProjection,
    build_workspace_frame, build_workspace_frame_with_ink,
    build_workspace_frame_with_ink_and_stroke, default_surface_size, drawing_ink_texture_target_id,
};
pub(crate) use state::DrawingPreviewTileJobSnapshot;
pub use state::{DrawingPreviewTileJobTracker, RunenwerkDrawApp};
pub use workspace::{DrawingCanvasView, DrawingTabletPanelProjection, DrawingWorkspaceProjection};
