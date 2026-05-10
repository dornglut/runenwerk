//! File: domain/drawing/src/diagnostics/issue.rs
//! Purpose: Stable drawing diagnostic code mapping.

use crate::DrawingIssueCode;

pub type DrawingDiagnosticCode = &'static str;

pub fn drawing_diagnostic_code(code: DrawingIssueCode) -> DrawingDiagnosticCode {
    match code {
        DrawingIssueCode::EmptyDocumentName => "drawing.document.empty_name",
        DrawingIssueCode::InvalidDocumentSchemaVersion => "drawing.document.invalid_schema_version",
        DrawingIssueCode::InvalidDocumentRevision => "drawing.document.invalid_revision",
        DrawingIssueCode::InvalidCanvasBounds => "drawing.document.invalid_canvas_bounds",
        DrawingIssueCode::DuplicateStrokeId => "drawing.stroke.duplicate_id",
        DrawingIssueCode::EmptyCommittedStroke => "drawing.stroke.empty_committed",
        DrawingIssueCode::InvalidStrokeSample => "drawing.stroke.invalid_sample",
        DrawingIssueCode::NonMonotonicStrokeSamples => "drawing.stroke.non_monotonic_samples",
        DrawingIssueCode::InvalidStrokeColor => "drawing.stroke.invalid_color",
        DrawingIssueCode::MissingBrushReference => "drawing.stroke.missing_brush_reference",
        DrawingIssueCode::AmbiguousPaintTarget => "drawing.composition.ambiguous_paint_target",
        DrawingIssueCode::DuplicateBrushId => "drawing.brush.duplicate_id",
        DrawingIssueCode::InvalidBrushRange => "drawing.brush.invalid_range",
        DrawingIssueCode::InvalidBrushDynamics => "drawing.brush.invalid_dynamics",
        DrawingIssueCode::DuplicatePaperId => "drawing.paper.duplicate_id",
        DrawingIssueCode::InvalidPaperRange => "drawing.paper.invalid_range",
        DrawingIssueCode::MissingPaperHeightReference => "drawing.paper.missing_height_reference",
        DrawingIssueCode::GraphStructural => "drawing.composition.graph_structural",
        DrawingIssueCode::CompositionGraphCycle => "drawing.composition.graph_cycle",
        DrawingIssueCode::IncompatibleCompositePorts => "drawing.composition.incompatible_ports",
        DrawingIssueCode::MissingGraphNodeSemantics => "drawing.composition.missing_node_semantics",
        DrawingIssueCode::OrphanGraphNodeSemantics => "drawing.composition.orphan_node_semantics",
        DrawingIssueCode::MissingRootLayerStack => "drawing.composition.missing_root_stack",
        DrawingIssueCode::DuplicateLayerStackEntryId => {
            "drawing.composition.duplicate_stack_entry_id"
        }
        DrawingIssueCode::InvalidLayerOrdering => "drawing.composition.invalid_layer_ordering",
        DrawingIssueCode::InvalidLayerOpacity => "drawing.composition.invalid_layer_opacity",
        DrawingIssueCode::MissingRequiredNodeInput => "drawing.composition.missing_required_input",
        DrawingIssueCode::OutputWithoutSemantics => "drawing.composition.output_without_semantics",
        DrawingIssueCode::DeclaredEffectSaved => "drawing.composition.declared_effect_saved",
        DrawingIssueCode::PassThroughGroupUnsupported => {
            "drawing.composition.pass_through_group_unsupported"
        }
        DrawingIssueCode::InvalidTransform => "drawing.composition.invalid_transform",
        DrawingIssueCode::InvalidAdjustment => "drawing.composition.invalid_adjustment",
        DrawingIssueCode::CommandTargetMissing => "drawing.command.target_missing",
        DrawingIssueCode::DuplicateTileProductId => "drawing.tile.duplicate_product_id",
        DrawingIssueCode::InvalidTileLineage => "drawing.lineage.invalid_tile_lineage",
        DrawingIssueCode::MissingTileSourceRevision => "drawing.tile.missing_source_revision",
        DrawingIssueCode::MissingFormationVersion => "drawing.tile.missing_formation_version",
        DrawingIssueCode::InvalidInvalidationBounds => "drawing.tile.invalid_invalidation_bounds",
    }
}
