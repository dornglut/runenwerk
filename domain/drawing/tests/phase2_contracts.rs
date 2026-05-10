use drawing::*;
use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId,
};

const ROOT_STACK: NodeId = NodeId(1);
const PAINT_SOURCE: NodeId = NodeId(2);
const OUTPUT: NodeId = NodeId(3);

fn valid_document() -> DrawingDocument {
    let root_out = PortId(10);
    let output_in = PortId(30);
    let graph = GraphDefinition::new(
        GraphId::new(1),
        "drawing",
        CyclePolicy::RejectDirectedCycles,
        [
            NodeDefinition::new(
                ROOT_STACK,
                "layer_stack",
                [PortDefinition::new(
                    root_out,
                    "color",
                    PortDirection::Output,
                    CompositePortSemantic::Color.port_type(),
                )],
            ),
            NodeDefinition::new(
                PAINT_SOURCE,
                "paint_source",
                [PortDefinition::new(
                    PortId(20),
                    "color",
                    PortDirection::Output,
                    CompositePortSemantic::Color.port_type(),
                )],
            ),
            NodeDefinition::new(
                OUTPUT,
                "output",
                [PortDefinition::new(
                    output_in,
                    "color",
                    PortDirection::Input,
                    CompositePortSemantic::Color.port_type(),
                )],
            ),
        ],
        [EdgeDefinition::new(EdgeId(1), root_out, output_in)],
    );

    let entry = LayerStackEntry::new(
        LayerStackEntryId::new(1),
        "Ink",
        LayerStackEntryContent::PaintSource(PAINT_SOURCE),
    );
    let composition = DrawingCompositeGraph::new(
        graph,
        ROOT_STACK,
        [
            (
                ROOT_STACK,
                DrawingCompositeNode::LayerStack(LayerStackNode::new([entry])),
            ),
            (
                PAINT_SOURCE,
                DrawingCompositeNode::PaintLayerSource(PaintLayerSource::new(
                    PaintSourceId::new(1),
                    "Ink Source",
                )),
            ),
            (
                OUTPUT,
                DrawingCompositeNode::CompositeOutput(CompositeOutput::new(
                    CompositeOutputId::new(1),
                    "Final",
                    ROOT_STACK,
                    CompositeOutputSemantics::FinalCanvasColor,
                    ProductQualityClass::Final,
                )),
            ),
        ],
        Some(CompositeOutputId::new(1)),
    );

    let mut document = DrawingDocument::new(
        DrawingDocumentId::new(1),
        "Drawing",
        CanvasRect::new(
            CanvasCoordinate::new(0.0, 0.0),
            CanvasCoordinate::new(1024.0, 1024.0),
        ),
        composition,
    );
    document.brushes.push(BrushDescriptor::new(
        BrushId::new(1),
        "Ink",
        InkBrushDescriptor::new(
            BrushRange::new(1.0, 12.0),
            BrushRange::new(0.1, 1.0),
            BrushRange::new(0.1, 1.0),
        ),
    ));
    document.papers.push(PaperDescriptor::new(
        PaperId::new(1),
        "Smooth",
        0.25,
        0.5,
        PaperHeightSource::None,
    ));
    document
}

fn sample(sequence: u64, x: f64) -> StrokeSample {
    StrokeSample::new(CanvasCoordinate::new(x, 1.0), sequence)
        .with_timestamp_micros(sequence * 100)
        .with_pressure(0.5)
}

fn committed_stroke(stroke_id: StrokeId) -> StrokeRecord {
    StrokeRecord::new(
        stroke_id,
        PaintTarget::StackEntry(LayerStackEntryId::new(1)),
        BrushId::new(1),
        ColorRgba::new(0.1, 0.2, 0.3, 1.0),
        [sample(1, 1.0), sample(2, 2.0)],
        DrawingDocumentRevision::new(1),
    )
    .expect("valid stroke")
}

fn issue_codes(report: &DrawingRatificationReport) -> Vec<DrawingIssueCode> {
    report.issues().iter().map(|issue| *issue.code()).collect()
}

#[test]
fn valid_document_contract_is_accepted() {
    let document = valid_document();

    let report = ratify_drawing_document(&document);

    assert!(report.is_accepted(), "{:?}", report.issues());
}

#[test]
fn invalid_stroke_samples_are_rejected() {
    let mut document = valid_document();
    let mut stroke = committed_stroke(StrokeId::new(1));
    stroke.samples[0].pressure = Some(1.5);
    document.strokes.push(stroke);

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::InvalidStrokeSample));
}

#[test]
fn non_monotonic_stroke_samples_are_rejected() {
    let mut document = valid_document();
    let mut stroke = committed_stroke(StrokeId::new(1));
    stroke.samples[1].sequence = stroke.samples[0].sequence;
    document.strokes.push(stroke);

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::NonMonotonicStrokeSamples));
}

#[test]
fn empty_committed_strokes_are_rejected() {
    let mut document = valid_document();
    document.strokes.push(StrokeRecord {
        stroke_id: StrokeId::new(1),
        target: PaintTarget::StackEntry(LayerStackEntryId::new(1)),
        brush_id: BrushId::new(1),
        color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
        samples: Vec::new(),
        bounds: CanvasRect::new(
            CanvasCoordinate::new(0.0, 0.0),
            CanvasCoordinate::new(0.0, 0.0),
        ),
        source_revision: DrawingDocumentRevision::new(1),
    });

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::EmptyCommittedStroke));
}

#[test]
fn brush_ranges_are_ratified() {
    let mut document = valid_document();
    document.brushes[0].ink.opacity = BrushRange::new(0.5, 1.5);

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::InvalidBrushRange));
}

#[test]
fn paper_ranges_and_references_are_ratified() {
    let mut document = valid_document();
    document.papers[0].height_source = PaperHeightSource::ImportedHeightField {
        asset_ref: " ".to_string(),
    };

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::MissingPaperHeightReference));
}

#[test]
fn stack_entry_identity_is_stable_through_reorder() {
    let mut document = valid_document();
    DrawingCommand::CreateStackEntry {
        entry: LayerStackEntry::new(
            LayerStackEntryId::new(2),
            "Ink 2",
            LayerStackEntryContent::PaintSource(PAINT_SOURCE),
        ),
    }
    .apply_to(&mut document)
    .expect("create stack entry");

    DrawingCommand::ReorderStackEntry {
        entry_id: LayerStackEntryId::new(1),
        new_index: 1,
    }
    .apply_to(&mut document)
    .expect("reorder");

    let stack = document.composition.root_stack().expect("root stack");
    assert_eq!(stack.entries[1].entry_id, LayerStackEntryId::new(1));
}

#[test]
fn pass_through_groups_are_rejected_until_supported() {
    let mut document = valid_document();
    document.composition.nodes.insert(
        NodeId(4),
        DrawingCompositeNode::Group(GroupNode {
            name: "Group".to_string(),
            child_stack: LayerStackNode::default(),
            mask_node: None,
            clip_policy: GroupClipPolicy::None,
            isolation_policy: GroupIsolationPolicy::PassThrough,
        }),
    );
    document.composition.graph.nodes.push(NodeDefinition::new(
        NodeId(4),
        "group",
        [PortDefinition::new(
            PortId(40),
            "color",
            PortDirection::Output,
            CompositePortSemantic::Color.port_type(),
        )],
    ));

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::PassThroughGroupUnsupported));
}

#[test]
fn composition_graph_structural_and_semantic_ratification_are_separate() {
    let mut semantic_document = valid_document();
    semantic_document.composition.nodes.remove(&PAINT_SOURCE);
    let semantic_report = ratify_drawing_document(&semantic_document);
    assert!(issue_codes(&semantic_report).contains(&DrawingIssueCode::MissingGraphNodeSemantics));

    let mut structural_document = valid_document();
    structural_document.composition.graph.nodes[2].ports[0].port_type =
        CompositePortSemantic::Alpha.port_type();
    let structural_report = ratify_drawing_document(&structural_document);
    assert!(
        issue_codes(&structural_report).contains(&DrawingIssueCode::IncompatibleCompositePorts)
    );
}

#[test]
fn composition_graph_cycles_are_rejected() {
    let mut document = valid_document();
    document.composition.graph.nodes[0]
        .ports
        .push(PortDefinition::new(
            PortId(11),
            "in",
            PortDirection::Input,
            CompositePortSemantic::Color.port_type(),
        ));
    document.composition.graph.nodes[2]
        .ports
        .push(PortDefinition::new(
            PortId(31),
            "out",
            PortDirection::Output,
            CompositePortSemantic::Color.port_type(),
        ));
    document
        .composition
        .graph
        .edges
        .push(EdgeDefinition::new(EdgeId(2), PortId(31), PortId(11)));

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::CompositionGraphCycle));
}

#[test]
fn ambiguous_paint_targets_are_rejected() {
    let mut document = valid_document();
    document.composition.nodes.insert(
        NodeId(4),
        DrawingCompositeNode::PaintLayerSource(PaintLayerSource::new(
            PaintSourceId::new(1),
            "Duplicate",
        )),
    );
    document.composition.graph.nodes.push(NodeDefinition::new(
        NodeId(4),
        "paint_source_2",
        [PortDefinition::new(
            PortId(40),
            "color",
            PortDirection::Output,
            CompositePortSemantic::Color.port_type(),
        )],
    ));
    document.strokes.push(
        StrokeRecord::new(
            StrokeId::new(1),
            PaintTarget::PaintSource(PaintSourceId::new(1)),
            BrushId::new(1),
            ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            [sample(1, 1.0)],
            DrawingDocumentRevision::new(1),
        )
        .expect("stroke"),
    );

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::AmbiguousPaintTarget));
}

#[test]
fn outputs_without_semantics_are_rejected() {
    let mut document = valid_document();
    let DrawingCompositeNode::CompositeOutput(output) =
        document.composition.nodes.get_mut(&OUTPUT).expect("output")
    else {
        panic!("expected output");
    };
    output.semantics = None;

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::OutputWithoutSemantics));
}

#[test]
fn declared_effects_cannot_be_saved_as_document_state() {
    let mut document = valid_document();
    document.composition.nodes.insert(
        NodeId(4),
        DrawingCompositeNode::Effect(EffectNode::new(
            EffectFamily::Watercolor,
            "wet_bleed",
            EffectMaturityTier::Declared,
        )),
    );
    document.composition.graph.nodes.push(NodeDefinition::new(
        NodeId(4),
        "effect",
        [PortDefinition::new(
            PortId(40),
            "color",
            PortDirection::Output,
            CompositePortSemantic::Color.port_type(),
        )],
    ));

    let report = ratify_drawing_document(&document);

    assert!(issue_codes(&report).contains(&DrawingIssueCode::DeclaredEffectSaved));
}

#[test]
fn tile_product_descriptors_preserve_lineage_without_payloads() {
    let mut document = valid_document();
    let lineage = DrawingProductLineage::new(document.revision);
    document.tile_products.push(
        DrawingTileProduct::new(
            DrawingTileProductId::new(1),
            CanvasTileId::new(TilePyramidLevel::new(0), 0, 0),
            DrawingTileProductSource::new(
                ProductQualityClass::Preview,
                document.revision,
                CompositeOutputId::new(1),
                lineage,
                FormationVersion::new(1),
                document.canvas_bounds,
            ),
        )
        .with_last_good(DrawingTileProductId::new(2)),
    );

    let report = ratify_drawing_document(&document);

    assert!(report.is_accepted(), "{:?}", report.issues());
    assert_eq!(
        document.tile_products[0].last_good_product,
        Some(DrawingTileProductId::new(2))
    );
}

#[test]
fn invalid_tile_lineage_is_rejected() {
    let mut document = valid_document();
    document.tile_products.push(DrawingTileProduct::new(
        DrawingTileProductId::new(1),
        CanvasTileId::new(TilePyramidLevel::new(0), 0, 0),
        DrawingTileProductSource::new(
            ProductQualityClass::Preview,
            DrawingDocumentRevision::new(0),
            CompositeOutputId::new(99),
            DrawingProductLineage::new(DrawingDocumentRevision::new(1)),
            FormationVersion::new(0),
            document.canvas_bounds,
        ),
    ));

    let report = ratify_drawing_document(&document);
    let codes = issue_codes(&report);

    assert!(codes.contains(&DrawingIssueCode::MissingTileSourceRevision));
    assert!(codes.contains(&DrawingIssueCode::MissingFormationVersion));
    assert!(codes.contains(&DrawingIssueCode::InvalidTileLineage));
}

#[test]
fn commands_and_transactions_produce_accepted_state_or_diagnostics() {
    let mut document = valid_document();
    let transaction = DrawingTransaction::new(
        "draw stroke",
        [
            DrawingCommand::BeginStroke {
                stroke_id: StrokeId::new(1),
                target: PaintTarget::StackEntry(LayerStackEntryId::new(1)),
                brush_id: BrushId::new(1),
                color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            },
            DrawingCommand::AppendStrokeSample {
                stroke_id: StrokeId::new(1),
                sample: sample(1, 1.0),
            },
            DrawingCommand::CommitStroke {
                stroke_id: StrokeId::new(1),
            },
        ],
    );

    let outcomes = transaction.apply_to(&mut document).expect("accepted");

    assert_eq!(outcomes.len(), 3);
    assert_eq!(document.strokes.len(), 1);
    assert!(ratify_drawing_document(&document).is_accepted());

    let rejected = DrawingCommand::SetLayerOpacity {
        entry_id: LayerStackEntryId::new(1),
        opacity: 2.0,
    }
    .apply_to(&mut document)
    .expect_err("invalid opacity should be rejected");
    assert!(issue_codes(&rejected).contains(&DrawingIssueCode::InvalidLayerOpacity));
}
