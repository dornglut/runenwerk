//! Drawing app product state.

pub mod document_factory;
pub mod ink;
pub mod input;
pub mod presentation;
pub mod state;
pub mod workspace;

pub use document_factory::minimal_drawing_document;
pub use ink::{DrawingInkJournalEntry, DrawingInkJournalStage, DrawingInkRuntimeState};
pub use input::{DrawingPreviewStroke, DrawingToolInputEvent, DrawingToolRouteKind};
pub use presentation::{
    DRAWING_CANVAS_LAYER_ID, DRAWING_INK_TEXTURE_NAMESPACE, DRAWING_UI_SURFACE_ID,
    DrawingInkSurfaceKind, build_workspace_frame, build_workspace_frame_with_ink,
    default_surface_size, drawing_ink_texture_target_id,
};
pub use state::RunenwerkDrawApp;
pub use workspace::{DrawingCanvasView, DrawingWorkspaceProjection};
