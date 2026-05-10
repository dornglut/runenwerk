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
    CanvasCoordinate, CanvasRect, CanvasTileId, DrawingTileProduct, DrawingTileProductSource,
    ProductQualityClass, TilePyramidLevel,
};
