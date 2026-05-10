//! File: domain/drawing/src/ratification/ratifier.rs
//! Purpose: Drawing document semantic ratification.

use std::collections::{BTreeMap, BTreeSet};

use graph::{GraphValidationError, NodeId, validate_graph};
use ratification::{RatificationIssue, RatificationReport};

use crate::{
    AdjustmentDescriptor, CanvasTileId, CompositeOutputId, DrawingCompositeNode, DrawingDocument,
    DrawingTileProductId, EffectMaturityTier, LayerStackEntry, LayerStackEntryId, PaintSourceId,
    PaintTarget, PaperHeightSource, StrokeId, StrokeSample,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawingIssueCode {
    EmptyDocumentName,
    InvalidDocumentSchemaVersion,
    InvalidDocumentRevision,
    InvalidCanvasBounds,
    DuplicateStrokeId,
    EmptyCommittedStroke,
    InvalidStrokeSample,
    NonMonotonicStrokeSamples,
    InvalidStrokeColor,
    MissingBrushReference,
    AmbiguousPaintTarget,
    DuplicateBrushId,
    InvalidBrushRange,
    InvalidBrushDynamics,
    DuplicatePaperId,
    InvalidPaperRange,
    MissingPaperHeightReference,
    GraphStructural,
    CompositionGraphCycle,
    IncompatibleCompositePorts,
    MissingGraphNodeSemantics,
    OrphanGraphNodeSemantics,
    MissingRootLayerStack,
    DuplicateLayerStackEntryId,
    InvalidLayerOrdering,
    InvalidLayerOpacity,
    MissingRequiredNodeInput,
    OutputWithoutSemantics,
    DeclaredEffectSaved,
    PassThroughGroupUnsupported,
    InvalidTransform,
    InvalidAdjustment,
    CommandTargetMissing,
    DuplicateTileProductId,
    InvalidTileLineage,
    MissingTileSourceRevision,
    MissingFormationVersion,
    InvalidInvalidationBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawingIssueSubject {
    Document,
    Stroke(StrokeId),
    Brush(crate::BrushId),
    Paper(crate::PaperId),
    CompositionGraph,
    Node(NodeId),
    LayerStackEntry(LayerStackEntryId),
    PaintSource(PaintSourceId),
    Output(CompositeOutputId),
    TileProduct(DrawingTileProductId),
    Tile(CanvasTileId),
    Lineage,
}

pub type DrawingRatificationReport = RatificationReport<DrawingIssueCode, DrawingIssueSubject>;

pub fn ratify_drawing_document(document: &DrawingDocument) -> DrawingRatificationReport {
    let mut report = DrawingRatificationReport::new();

    ratify_document_header(document, &mut report);
    ratify_brushes(document, &mut report);
    ratify_papers(document, &mut report);
    ratify_composition(document, &mut report);
    ratify_strokes(document, &mut report);
    ratify_tile_products(document, &mut report);

    report
}

fn ratify_document_header(document: &DrawingDocument, report: &mut DrawingRatificationReport) {
    if document.display_name.trim().is_empty() {
        push(
            report,
            DrawingIssueCode::EmptyDocumentName,
            DrawingIssueSubject::Document,
            "drawing document display name must not be empty",
        );
    }
    if document.schema_version == 0 {
        push(
            report,
            DrawingIssueCode::InvalidDocumentSchemaVersion,
            DrawingIssueSubject::Document,
            "drawing document schema version must be non-zero",
        );
    }
    if document.revision.raw() == 0 {
        push(
            report,
            DrawingIssueCode::InvalidDocumentRevision,
            DrawingIssueSubject::Document,
            "drawing document revision must be non-zero",
        );
    }
    if !document.canvas_bounds.is_valid() {
        push(
            report,
            DrawingIssueCode::InvalidCanvasBounds,
            DrawingIssueSubject::Document,
            "drawing document canvas bounds must be finite and ordered",
        );
    }
}

fn ratify_brushes(document: &DrawingDocument, report: &mut DrawingRatificationReport) {
    let mut ids = BTreeSet::new();
    for brush in &document.brushes {
        if !ids.insert(brush.brush_id) {
            push(
                report,
                DrawingIssueCode::DuplicateBrushId,
                DrawingIssueSubject::Brush(brush.brush_id),
                "brush ids must be unique",
            );
        }
        if brush.schema_version == 0 || brush.revision == 0 || brush.name.trim().is_empty() {
            push(
                report,
                DrawingIssueCode::InvalidBrushRange,
                DrawingIssueSubject::Brush(brush.brush_id),
                "brush descriptor must have schema version, revision, and name",
            );
        }
        if !brush.ink.size.is_valid_positive()
            || !brush.ink.opacity.is_valid_unit()
            || !brush.ink.flow.is_valid_unit()
            || !unit_value(brush.ink.edge_softness)
            || !unit_value(brush.ink.viscosity)
            || !unit_value(brush.ink.absorption_response)
        {
            push(
                report,
                DrawingIssueCode::InvalidBrushRange,
                DrawingIssueSubject::Brush(brush.brush_id),
                "ink brush ranges must be finite and within their allowed bounds",
            );
        }
        if !brush.ink.dynamics.pressure_to_size.is_valid()
            || !brush.ink.dynamics.pressure_to_opacity.is_valid()
        {
            push(
                report,
                DrawingIssueCode::InvalidBrushDynamics,
                DrawingIssueSubject::Brush(brush.brush_id),
                "brush dynamics curves must be finite and bounded",
            );
        }
    }
}

fn ratify_papers(document: &DrawingDocument, report: &mut DrawingRatificationReport) {
    let mut ids = BTreeSet::new();
    for paper in &document.papers {
        if !ids.insert(paper.paper_id) {
            push(
                report,
                DrawingIssueCode::DuplicatePaperId,
                DrawingIssueSubject::Paper(paper.paper_id),
                "paper ids must be unique",
            );
        }
        if paper.schema_version == 0
            || paper.revision == 0
            || paper.name.trim().is_empty()
            || !unit_value(paper.roughness)
            || !unit_value(paper.absorbency)
        {
            push(
                report,
                DrawingIssueCode::InvalidPaperRange,
                DrawingIssueSubject::Paper(paper.paper_id),
                "paper descriptor ranges must be finite and bounded",
            );
        }
        match &paper.height_source {
            PaperHeightSource::None => {}
            PaperHeightSource::ProceduralNoise {
                scale, amplitude, ..
            } => {
                if !scale.is_finite() || *scale <= 0.0 || !amplitude.is_finite() || *amplitude < 0.0
                {
                    push(
                        report,
                        DrawingIssueCode::InvalidPaperRange,
                        DrawingIssueSubject::Paper(paper.paper_id),
                        "procedural paper height source scale and amplitude must be valid",
                    );
                }
            }
            PaperHeightSource::FormedProductReference { product_ref }
            | PaperHeightSource::ImportedHeightField {
                asset_ref: product_ref,
            }
            | PaperHeightSource::SdfDerived {
                source_ref: product_ref,
            } => {
                if product_ref.trim().is_empty() {
                    push(
                        report,
                        DrawingIssueCode::MissingPaperHeightReference,
                        DrawingIssueSubject::Paper(paper.paper_id),
                        "paper height source reference must not be empty",
                    );
                }
            }
        }
    }
}

fn ratify_composition(document: &DrawingDocument, report: &mut DrawingRatificationReport) {
    let composition = &document.composition;

    if composition.graph.cycle_policy != graph::CyclePolicy::RejectDirectedCycles {
        push(
            report,
            DrawingIssueCode::CompositionGraphCycle,
            DrawingIssueSubject::CompositionGraph,
            "drawing composition graphs must reject directed cycles",
        );
    }

    if let Err(error) = validate_graph(&composition.graph) {
        let code = graph_issue_code(&error);
        push(
            report,
            code,
            DrawingIssueSubject::CompositionGraph,
            graph_error_message(&error),
        );
    }

    let graph_node_ids = composition
        .graph
        .nodes
        .iter()
        .map(|node| node.id)
        .collect::<BTreeSet<_>>();

    for node in &composition.graph.nodes {
        if !composition.nodes.contains_key(&node.id) {
            push(
                report,
                DrawingIssueCode::MissingGraphNodeSemantics,
                DrawingIssueSubject::Node(node.id),
                "drawing graph node is missing drawing-owned semantics",
            );
        }
    }
    for node_id in composition.nodes.keys().copied() {
        if !graph_node_ids.contains(&node_id) {
            push(
                report,
                DrawingIssueCode::OrphanGraphNodeSemantics,
                DrawingIssueSubject::Node(node_id),
                "drawing node semantics are not backed by graph structure",
            );
        }
    }

    match composition.nodes.get(&composition.root_stack_node) {
        Some(DrawingCompositeNode::LayerStack(stack)) => {
            ratify_stack_entries(&stack.entries, &composition.nodes, report);
        }
        _ => push(
            report,
            DrawingIssueCode::MissingRootLayerStack,
            DrawingIssueSubject::CompositionGraph,
            "drawing composition graph must have a root LayerStackNode",
        ),
    }

    for (node_id, node) in &composition.nodes {
        ratify_composite_node(*node_id, node, &composition.nodes, report);
    }

    if let Some(active_output) = composition.active_output {
        let found = composition.nodes.values().any(|node| {
            matches!(
                node,
                DrawingCompositeNode::CompositeOutput(output) if output.output_id == active_output
            )
        });
        if !found {
            push(
                report,
                DrawingIssueCode::MissingRequiredNodeInput,
                DrawingIssueSubject::Output(active_output),
                "active composite output must reference an output node",
            );
        }
    }
}

fn ratify_composite_node(
    node_id: NodeId,
    node: &DrawingCompositeNode,
    nodes: &BTreeMap<NodeId, DrawingCompositeNode>,
    report: &mut DrawingRatificationReport,
) {
    match node {
        DrawingCompositeNode::LayerStack(stack) => {
            ratify_stack_entries(&stack.entries, nodes, report);
        }
        DrawingCompositeNode::Group(group) => {
            if group.isolation_policy == crate::GroupIsolationPolicy::PassThrough {
                push(
                    report,
                    DrawingIssueCode::PassThroughGroupUnsupported,
                    DrawingIssueSubject::Node(node_id),
                    "pass-through group blending is deferred",
                );
            }
            if let Some(mask_node) = group.mask_node {
                require_node(mask_node, nodes, report);
            }
            ratify_stack_entries(&group.child_stack.entries, nodes, report);
        }
        DrawingCompositeNode::Mask(mask) => {
            if let Some(source) = mask.source_node {
                require_node(source, nodes, report);
            } else {
                missing_input(node_id, report);
            }
        }
        DrawingCompositeNode::Clip(clip) => match (clip.source_node, clip.clip_node) {
            (Some(source), Some(clip_node)) => {
                require_node(source, nodes, report);
                require_node(clip_node, nodes, report);
            }
            _ => missing_input(node_id, report),
        },
        DrawingCompositeNode::Transform(transform) => {
            if !transform.is_valid() {
                push(
                    report,
                    DrawingIssueCode::InvalidTransform,
                    DrawingIssueSubject::Node(node_id),
                    "transform node values must be finite and non-zero scale",
                );
            }
            if let Some(source) = transform.source_node {
                require_node(source, nodes, report);
            } else {
                missing_input(node_id, report);
            }
        }
        DrawingCompositeNode::Adjustment(adjustment) => {
            if let Some(source) = adjustment.source_node {
                require_node(source, nodes, report);
            } else {
                missing_input(node_id, report);
            }
            if !adjustment_descriptor_is_valid(&adjustment.descriptor) {
                push(
                    report,
                    DrawingIssueCode::InvalidAdjustment,
                    DrawingIssueSubject::Node(node_id),
                    "adjustment descriptor values must be finite and bounded",
                );
            }
        }
        DrawingCompositeNode::CompositeOutput(output) => {
            if output.semantics.is_none() {
                push(
                    report,
                    DrawingIssueCode::OutputWithoutSemantics,
                    DrawingIssueSubject::Output(output.output_id),
                    "composite outputs must declare explicit semantics",
                );
            }
            if let Some(source) = output.source_node {
                require_node(source, nodes, report);
            } else {
                missing_input(node_id, report);
            }
        }
        DrawingCompositeNode::Effect(effect) => {
            if effect.maturity == EffectMaturityTier::Declared {
                push(
                    report,
                    DrawingIssueCode::DeclaredEffectSaved,
                    DrawingIssueSubject::Node(node_id),
                    "declared effects are catalog-only and cannot be saved as authored state",
                );
            }
        }
        DrawingCompositeNode::PaintLayerSource(_)
        | DrawingCompositeNode::PaperSource(_)
        | DrawingCompositeNode::ReferenceImageSource(_) => {}
    }
}

fn ratify_stack_entries(
    entries: &[LayerStackEntry],
    nodes: &BTreeMap<NodeId, DrawingCompositeNode>,
    report: &mut DrawingRatificationReport,
) {
    let mut ids = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if !ids.insert(entry.entry_id) {
            push(
                report,
                DrawingIssueCode::DuplicateLayerStackEntryId,
                DrawingIssueSubject::LayerStackEntry(entry.entry_id),
                "layer stack entry ids must be unique within a stack",
            );
        }
        if entry.name.trim().is_empty() {
            push(
                report,
                DrawingIssueCode::InvalidLayerOrdering,
                DrawingIssueSubject::LayerStackEntry(entry.entry_id),
                "layer stack entry names must not be empty",
            );
        }
        if !unit_value(entry.opacity) {
            push(
                report,
                DrawingIssueCode::InvalidLayerOpacity,
                DrawingIssueSubject::LayerStackEntry(entry.entry_id),
                "layer stack opacity must be finite and within 0..=1",
            );
        }
        if entry.clip_to_below && index == 0 {
            push(
                report,
                DrawingIssueCode::InvalidLayerOrdering,
                DrawingIssueSubject::LayerStackEntry(entry.entry_id),
                "first layer stack entry cannot clip to a missing layer below it",
            );
        }
        require_node(entry.content.source_node(), nodes, report);
        if let Some(mask_node) = entry.mask_node {
            require_node(mask_node, nodes, report);
        }
    }
}

fn ratify_strokes(document: &DrawingDocument, report: &mut DrawingRatificationReport) {
    let mut committed_ids = BTreeSet::new();
    let brush_ids = document
        .brushes
        .iter()
        .map(|brush| brush.brush_id)
        .collect::<BTreeSet<_>>();
    let paint_source_counts = paint_source_counts(document);
    let stack_entry_counts = stack_entry_counts(document);

    for stroke in &document.strokes {
        if !committed_ids.insert(stroke.stroke_id) {
            push(
                report,
                DrawingIssueCode::DuplicateStrokeId,
                DrawingIssueSubject::Stroke(stroke.stroke_id),
                "committed stroke ids must be unique",
            );
        }
        ratify_stroke_samples(stroke.stroke_id, &stroke.samples, true, report);
        if !stroke.color.is_valid() {
            push(
                report,
                DrawingIssueCode::InvalidStrokeColor,
                DrawingIssueSubject::Stroke(stroke.stroke_id),
                "stroke color channels must be finite and within 0..=1",
            );
        }
        if !brush_ids.contains(&stroke.brush_id) {
            push(
                report,
                DrawingIssueCode::MissingBrushReference,
                DrawingIssueSubject::Stroke(stroke.stroke_id),
                "stroke brush reference must resolve to a brush descriptor",
            );
        }
        ratify_paint_target(
            stroke.stroke_id,
            stroke.target,
            &paint_source_counts,
            &stack_entry_counts,
            report,
        );
        if !stroke.bounds.is_valid() {
            push(
                report,
                DrawingIssueCode::InvalidStrokeSample,
                DrawingIssueSubject::Stroke(stroke.stroke_id),
                "stroke bounds must be finite and ordered",
            );
        }
    }

    for pending in &document.pending_strokes {
        if committed_ids.contains(&pending.stroke_id) {
            push(
                report,
                DrawingIssueCode::DuplicateStrokeId,
                DrawingIssueSubject::Stroke(pending.stroke_id),
                "pending stroke id must not duplicate a committed stroke",
            );
        }
        ratify_stroke_samples(pending.stroke_id, &pending.samples, false, report);
        if !pending.color.is_valid() {
            push(
                report,
                DrawingIssueCode::InvalidStrokeColor,
                DrawingIssueSubject::Stroke(pending.stroke_id),
                "pending stroke color channels must be finite and within 0..=1",
            );
        }
        if !brush_ids.contains(&pending.brush_id) {
            push(
                report,
                DrawingIssueCode::MissingBrushReference,
                DrawingIssueSubject::Stroke(pending.stroke_id),
                "pending stroke brush reference must resolve to a brush descriptor",
            );
        }
        ratify_paint_target(
            pending.stroke_id,
            pending.target,
            &paint_source_counts,
            &stack_entry_counts,
            report,
        );
    }
}

fn ratify_stroke_samples(
    stroke_id: StrokeId,
    samples: &[StrokeSample],
    require_non_empty: bool,
    report: &mut DrawingRatificationReport,
) {
    if require_non_empty && samples.is_empty() {
        push(
            report,
            DrawingIssueCode::EmptyCommittedStroke,
            DrawingIssueSubject::Stroke(stroke_id),
            "committed stroke must contain at least one sample",
        );
    }
    let mut last_sequence = None;
    let mut last_timestamp = None;
    for sample in samples {
        if !sample.position.is_finite()
            || sample
                .pressure
                .is_some_and(|pressure| !unit_value(pressure))
            || sample.tilt.is_some_and(|tilt| !tilt.is_valid())
            || sample
                .twist_degrees
                .is_some_and(|twist| !twist.is_finite() || !(0.0..=360.0).contains(&twist))
        {
            push(
                report,
                DrawingIssueCode::InvalidStrokeSample,
                DrawingIssueSubject::Stroke(stroke_id),
                "stroke samples must have finite position and bounded stylus values",
            );
        }
        if let Some(previous) = last_sequence
            && sample.sequence <= previous
        {
            push(
                report,
                DrawingIssueCode::NonMonotonicStrokeSamples,
                DrawingIssueSubject::Stroke(stroke_id),
                "stroke sample sequence values must be strictly increasing",
            );
        }
        if let (Some(previous), Some(current)) = (last_timestamp, sample.timestamp_micros)
            && current < previous
        {
            push(
                report,
                DrawingIssueCode::NonMonotonicStrokeSamples,
                DrawingIssueSubject::Stroke(stroke_id),
                "stroke sample timestamps must be monotonic",
            );
        }
        last_sequence = Some(sample.sequence);
        if let Some(timestamp) = sample.timestamp_micros {
            last_timestamp = Some(timestamp);
        }
    }
}

fn ratify_paint_target(
    stroke_id: StrokeId,
    target: PaintTarget,
    paint_source_counts: &BTreeMap<PaintSourceId, usize>,
    stack_entry_counts: &BTreeMap<LayerStackEntryId, usize>,
    report: &mut DrawingRatificationReport,
) {
    let count = match target {
        PaintTarget::PaintSource(id) => paint_source_counts.get(&id).copied().unwrap_or(0),
        PaintTarget::StackEntry(id) => stack_entry_counts.get(&id).copied().unwrap_or(0),
    };
    if count != 1 {
        push(
            report,
            DrawingIssueCode::AmbiguousPaintTarget,
            DrawingIssueSubject::Stroke(stroke_id),
            "stroke paint target must resolve to exactly one paint source or stack entry",
        );
    }
}

fn ratify_tile_products(document: &DrawingDocument, report: &mut DrawingRatificationReport) {
    let mut ids = BTreeSet::new();
    let output_ids = output_ids(document);
    for product in &document.tile_products {
        if !ids.insert(product.product_id) {
            push(
                report,
                DrawingIssueCode::DuplicateTileProductId,
                DrawingIssueSubject::TileProduct(product.product_id),
                "tile product ids must be unique",
            );
        }
        if product.source_document_revision.raw() == 0 {
            push(
                report,
                DrawingIssueCode::MissingTileSourceRevision,
                DrawingIssueSubject::TileProduct(product.product_id),
                "tile products must record a non-zero source document revision",
            );
        }
        if product.formation_version.raw() == 0 {
            push(
                report,
                DrawingIssueCode::MissingFormationVersion,
                DrawingIssueSubject::TileProduct(product.product_id),
                "tile products must record a non-zero formation version",
            );
        }
        if product.lineage.document_revision != product.source_document_revision {
            push(
                report,
                DrawingIssueCode::InvalidTileLineage,
                DrawingIssueSubject::TileProduct(product.product_id),
                "tile product lineage revision must match source revision",
            );
        }
        if !output_ids.contains(&product.source_output) {
            push(
                report,
                DrawingIssueCode::InvalidTileLineage,
                DrawingIssueSubject::TileProduct(product.product_id),
                "tile product source output must resolve to a composite output",
            );
        }
        if !product.invalidation_bounds.is_valid() {
            push(
                report,
                DrawingIssueCode::InvalidInvalidationBounds,
                DrawingIssueSubject::Tile(product.tile_id),
                "tile invalidation bounds must be finite and ordered",
            );
        }
        if product.last_good_product == Some(product.product_id) {
            push(
                report,
                DrawingIssueCode::InvalidTileLineage,
                DrawingIssueSubject::TileProduct(product.product_id),
                "tile product cannot name itself as its last-good fallback",
            );
        }
    }
}

fn require_node(
    node_id: NodeId,
    nodes: &BTreeMap<NodeId, DrawingCompositeNode>,
    report: &mut DrawingRatificationReport,
) {
    if !nodes.contains_key(&node_id) {
        push(
            report,
            DrawingIssueCode::MissingRequiredNodeInput,
            DrawingIssueSubject::Node(node_id),
            "node input references a missing drawing node",
        );
    }
}

fn missing_input(node_id: NodeId, report: &mut DrawingRatificationReport) {
    push(
        report,
        DrawingIssueCode::MissingRequiredNodeInput,
        DrawingIssueSubject::Node(node_id),
        "node is missing a required input",
    );
}

fn paint_source_counts(document: &DrawingDocument) -> BTreeMap<PaintSourceId, usize> {
    let mut counts = BTreeMap::new();
    for node in document.composition.nodes.values() {
        if let DrawingCompositeNode::PaintLayerSource(source) = node {
            *counts.entry(source.paint_source_id).or_insert(0) += 1;
        }
    }
    counts
}

fn stack_entry_counts(document: &DrawingDocument) -> BTreeMap<LayerStackEntryId, usize> {
    let mut counts = BTreeMap::new();
    for node in document.composition.nodes.values() {
        match node {
            DrawingCompositeNode::LayerStack(stack) => {
                collect_entry_counts(&stack.entries, &mut counts)
            }
            DrawingCompositeNode::Group(group) => {
                collect_entry_counts(&group.child_stack.entries, &mut counts)
            }
            _ => {}
        }
    }
    counts
}

fn collect_entry_counts(
    entries: &[LayerStackEntry],
    counts: &mut BTreeMap<LayerStackEntryId, usize>,
) {
    for entry in entries {
        *counts.entry(entry.entry_id).or_insert(0) += 1;
    }
}

fn output_ids(document: &DrawingDocument) -> BTreeSet<CompositeOutputId> {
    document
        .composition
        .nodes
        .values()
        .filter_map(|node| match node {
            DrawingCompositeNode::CompositeOutput(output) => Some(output.output_id),
            _ => None,
        })
        .collect()
}

fn adjustment_descriptor_is_valid(descriptor: &AdjustmentDescriptor) -> bool {
    match descriptor {
        AdjustmentDescriptor::Opacity(opacity) => unit_value(*opacity),
        AdjustmentDescriptor::BrightnessContrast {
            brightness,
            contrast,
        } => brightness.is_finite() && contrast.is_finite(),
        AdjustmentDescriptor::Hsv {
            hue_degrees,
            saturation,
            value,
        } => hue_degrees.is_finite() && saturation.is_finite() && value.is_finite(),
        AdjustmentDescriptor::Threshold { threshold } => unit_value(*threshold),
        AdjustmentDescriptor::ChannelRemap {
            red_from,
            green_from,
            blue_from,
            alpha_from,
        } => [*red_from, *green_from, *blue_from, *alpha_from]
            .into_iter()
            .all(|channel| channel < 4),
        AdjustmentDescriptor::SimpleGradientMap { dark, light } => {
            dark.is_valid() && light.is_valid()
        }
    }
}

fn graph_issue_code(error: &GraphValidationError) -> DrawingIssueCode {
    match error {
        GraphValidationError::PortTypeMismatch { .. } => {
            DrawingIssueCode::IncompatibleCompositePorts
        }
        GraphValidationError::DirectedCycleDetected => DrawingIssueCode::CompositionGraphCycle,
        _ => DrawingIssueCode::GraphStructural,
    }
}

fn graph_error_message(error: &GraphValidationError) -> String {
    match error {
        GraphValidationError::DuplicateNodeId(id) => format!("duplicate node id {}", id.raw()),
        GraphValidationError::DuplicatePortId(id) => format!("duplicate port id {}", id.raw()),
        GraphValidationError::DuplicateEdgeId(id) => format!("duplicate edge id {}", id.raw()),
        GraphValidationError::MissingNode(id) => format!("missing node {}", id.raw()),
        GraphValidationError::MissingPort { edge_id, port_id } => {
            format!(
                "edge {} references missing port {}",
                edge_id.raw(),
                port_id.raw()
            )
        }
        GraphValidationError::EdgeDirectionInvalid { edge_id, .. } => {
            format!("edge {} has invalid port directions", edge_id.raw())
        }
        GraphValidationError::PortTypeMismatch { edge_id, .. } => {
            format!(
                "edge {} has incompatible composite port types",
                edge_id.raw()
            )
        }
        GraphValidationError::DirectedCycleDetected => "directed cycle detected".to_string(),
    }
}

fn unit_value(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn push(
    report: &mut DrawingRatificationReport,
    code: DrawingIssueCode,
    subject: DrawingIssueSubject,
    message: impl Into<String>,
) {
    report.push(RatificationIssue::error(code, subject, message));
}
