//! Crate: drawing
//! Purpose: Pure domain contracts for drawing documents, strokes, brushes, paper, composition, tile lineage, commands, and ratification.

pub mod brush;
pub mod composition;
pub mod diagnostics;
pub mod document;
pub mod history;
pub mod ids;
pub mod paper;
pub mod product_lineage;
pub mod ratification;
pub mod simulation;
pub mod stroke;
pub mod tile;

pub use brush::{BrushDescriptor, BrushDynamics, BrushRange, DynamicsCurve, InkBrushDescriptor};
pub use composition::{
    AdjustmentDescriptor, AdjustmentNode, BlendMode, ClipMode, ClipNode, CompositeOutput,
    CompositeOutputId, CompositeOutputSemantics, CompositePort, CompositePortSemantic,
    DrawingCompositeGraph, DrawingCompositeNode, EffectFamily, EffectMaturityTier, EffectNode,
    GroupClipPolicy, GroupIsolationPolicy, GroupNode, LayerStackEntry, LayerStackEntryContent,
    LayerStackNode, MaskMode, MaskNode, PaintLayerSource, PaperSource, ReferenceImageSource,
    TransformNode,
};
pub use diagnostics::{DrawingDiagnosticCode, drawing_diagnostic_code};
pub use document::{DRAWING_DOCUMENT_SCHEMA_VERSION, DrawingDocument, DrawingDocumentRevision};
pub use history::{
    DrawingCommand, DrawingCommandOutcome, DrawingOperation, DrawingRecoveryState,
    DrawingTransaction, PendingStrokeRecord,
};
pub use ids::{
    BrushId, DrawingDocumentId, DrawingOperationId, DrawingTileProductId, LayerStackEntryId,
    PaintSourceId, PaperId, ReferenceImageId, StrokeId,
};
pub use paper::{PaperDescriptor, PaperHeightSource};
pub use product_lineage::{
    BrushLineageRef, DrawingProductLineage, PaperLineageRef, SourceRange, StrokeLineageRange,
};
pub use ratification::{
    DrawingIssueCode, DrawingIssueSubject, DrawingRatificationReport, ratify_drawing_document,
};
pub use simulation::FormationVersion;
pub use stroke::{ColorRgba, PaintTarget, StrokeRecord, StrokeSample, StrokeToolKind, StylusTilt};
pub use tile::{
    CanvasCoordinate, CanvasRect, CanvasTileId, DEFAULT_FINAL_INK_TILE_PIXEL_HEIGHT,
    DEFAULT_FINAL_INK_TILE_PIXEL_WIDTH, DEFAULT_INK_TILE_PIXEL_HEIGHT,
    DEFAULT_INK_TILE_PIXEL_WIDTH, DEFAULT_INK_TILE_SIZE_CANVAS_UNITS,
    DEFAULT_MAX_AFFECTED_INK_TILES, DrawingInkPreviewStroke, DrawingInkTileFormation,
    DrawingInkTileInvalidation, DrawingInkTilePayload, DrawingInkTileProduct,
    DrawingInkTileProductContracts, DrawingTileFormationDiagnostic,
    DrawingTileFormationDiagnosticCode, DrawingTileFormationPolicy, DrawingTileProduct,
    DrawingTileProductSource, ProductQualityClass, TilePyramidLevel,
    build_drawing_ink_tile_product_contracts, build_drawing_ink_tile_publication_outcome,
    drawing_committed_ink_tile_source_cache_key, drawing_ink_tile_diagnostic_to_field_product,
    drawing_ink_tile_invalidation_for_preview_stroke, drawing_ink_tile_invalidation_for_strokes,
    drawing_ink_tile_product_cache_identity, drawing_ink_tile_product_descriptor,
    drawing_ink_tile_query_snapshot_for_descriptor, drawing_quality_scale_band,
    drawing_tile_determinism_key, form_drawing_ink_preview_tiles,
    form_drawing_ink_preview_tiles_for_ids, form_drawing_ink_tiles, form_drawing_ink_tiles_for_ids,
};
