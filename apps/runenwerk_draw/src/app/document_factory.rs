//! Minimal drawing document construction for the app shell.

use drawing::{
    BrushDescriptor, BrushId, BrushRange, CanvasCoordinate, CanvasRect, CompositeOutput,
    CompositeOutputId, CompositeOutputSemantics, CompositePortSemantic, DrawingCompositeGraph,
    DrawingCompositeNode, DrawingDocument, DrawingDocumentId, InkBrushDescriptor, LayerStackEntry,
    LayerStackEntryContent, LayerStackEntryId, LayerStackNode, PaintLayerSource, PaintSourceId,
    PaperDescriptor, PaperHeightSource, PaperId, ProductQualityClass,
};
use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId,
};

const ROOT_STACK_NODE: NodeId = NodeId(1);
const PAINT_SOURCE_NODE: NodeId = NodeId(2);
const OUTPUT_NODE: NodeId = NodeId(3);
const ROOT_STACK_COLOR_PORT: PortId = PortId(10);
const PAINT_SOURCE_COLOR_PORT: PortId = PortId(20);
const OUTPUT_COLOR_PORT: PortId = PortId(30);

pub fn minimal_drawing_document() -> DrawingDocument {
    let graph = GraphDefinition::new(
        GraphId::new(1),
        "runenwerk_draw_minimal_composition",
        CyclePolicy::RejectDirectedCycles,
        [
            NodeDefinition::new(
                ROOT_STACK_NODE,
                "layer_stack",
                [PortDefinition::new(
                    ROOT_STACK_COLOR_PORT,
                    "color",
                    PortDirection::Output,
                    CompositePortSemantic::Color.port_type(),
                )],
            ),
            NodeDefinition::new(
                PAINT_SOURCE_NODE,
                "paint_source",
                [PortDefinition::new(
                    PAINT_SOURCE_COLOR_PORT,
                    "color",
                    PortDirection::Output,
                    CompositePortSemantic::Color.port_type(),
                )],
            ),
            NodeDefinition::new(
                OUTPUT_NODE,
                "output",
                [PortDefinition::new(
                    OUTPUT_COLOR_PORT,
                    "color",
                    PortDirection::Input,
                    CompositePortSemantic::Color.port_type(),
                )],
            ),
        ],
        [EdgeDefinition::new(
            EdgeId(1),
            ROOT_STACK_COLOR_PORT,
            OUTPUT_COLOR_PORT,
        )],
    );
    let layer = LayerStackEntry::new(
        LayerStackEntryId::new(1),
        "Ink",
        LayerStackEntryContent::PaintSource(PAINT_SOURCE_NODE),
    );
    let composition = DrawingCompositeGraph::new(
        graph,
        ROOT_STACK_NODE,
        [
            (
                ROOT_STACK_NODE,
                DrawingCompositeNode::LayerStack(LayerStackNode::new([layer])),
            ),
            (
                PAINT_SOURCE_NODE,
                DrawingCompositeNode::PaintLayerSource(PaintLayerSource::new(
                    PaintSourceId::new(1),
                    "Ink Source",
                )),
            ),
            (
                OUTPUT_NODE,
                DrawingCompositeNode::CompositeOutput(CompositeOutput::new(
                    CompositeOutputId::new(1),
                    "Final Canvas",
                    ROOT_STACK_NODE,
                    CompositeOutputSemantics::FinalCanvasColor,
                    ProductQualityClass::Final,
                )),
            ),
        ],
        Some(CompositeOutputId::new(1)),
    );
    let mut document = DrawingDocument::new(
        DrawingDocumentId::new(1),
        "Untitled Drawing",
        CanvasRect::new(
            CanvasCoordinate::new(0.0, 0.0),
            CanvasCoordinate::new(4096.0, 4096.0),
        ),
        composition,
    );
    document.brushes.push(BrushDescriptor::new(
        BrushId::new(1),
        "Pressure Ink",
        InkBrushDescriptor::new(
            BrushRange::new(1.0, 24.0),
            BrushRange::new(0.05, 1.0),
            BrushRange::new(0.05, 1.0),
        ),
    ));
    document.papers.push(PaperDescriptor::new(
        PaperId::new(1),
        "Smooth Paper",
        0.25,
        0.5,
        PaperHeightSource::None,
    ));
    document
}
