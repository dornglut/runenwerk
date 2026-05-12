//! Drawing app product state.

pub mod document_factory;
pub mod input;
pub mod presentation;
pub mod state;
pub mod workspace;

pub use document_factory::minimal_drawing_document;
pub use input::{DrawingPreviewStroke, DrawingToolInputEvent, DrawingToolRouteKind};
pub use presentation::{
    DRAWING_CANVAS_LAYER_ID, DRAWING_UI_SURFACE_ID, build_workspace_frame, default_surface_size,
};
pub use state::RunenwerkDrawApp;
pub use workspace::{DrawingCanvasView, DrawingWorkspaceProjection};
