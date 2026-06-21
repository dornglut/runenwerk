//! Drawing app product state.

pub mod composition;
pub mod document_factory;
pub mod ink;
pub mod input;
pub mod presentation;
pub mod state;
pub mod tool_session;

pub use composition::{
    DrawingCanvasView, DrawingCompositionContentState, DrawingCompositionDiagnosticCode,
    DrawingCompositionDiagnosticRecord, DrawingCompositionDiagnosticSeverity,
    DrawingCompositionDiagnosticStage, DrawingCompositionDiagnosticSubject,
    DrawingCompositionExtensionV1, DrawingCompositionProjection, DrawingCompositionRejection,
    DrawingCompositionRuntime, DrawingContentRole, DrawingMountedContentProjection,
    DrawingTabletPanelProjection,
};

pub use document_factory::minimal_drawing_document;
pub use ink::{
    DrawingInkGpuValidationEntry, DrawingInkGpuValidationMetrics, DrawingInkGpuValidationStatus,
    DrawingInkJournalEntry, DrawingInkJournalStage, DrawingInkRuntimeState,
};
pub use input::{DrawingPreviewStroke, DrawingToolInputEvent, DrawingToolRouteKind};
pub(crate) use presentation::build_composition_frame_with_ink_surface_refs_and_strokes;
pub use presentation::{
    DRAWING_CANVAS_LAYER_ID, DRAWING_INK_TEXTURE_NAMESPACE, DRAWING_UI_SURFACE_ID,
    DrawingImmediateStrokeProjection, DrawingInkSurfaceKind, DrawingInkSurfaceProjection,
    build_composition_frame, build_composition_frame_with_ink,
    build_composition_frame_with_ink_and_stroke, default_surface_size,
    drawing_ink_texture_target_id,
};
pub(crate) use state::DrawingPreviewTileJobSnapshot;
pub use state::{DrawingPreviewTileJobTracker, RunenwerkDrawApp};
pub use tool_session::{
    DrawingToolControlInputEvent, DrawingToolControlInputSource, DrawingToolControlRequest,
    DrawingToolIntent, DrawingToolSession, DrawingToolSessionAnchor, DrawingToolSessionId,
    DrawingToolSessionKind, DrawingToolSessionOutcome, DrawingToolSessionPhase,
};
