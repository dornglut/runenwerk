use drawing::*;
use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId,
};
use product::{
    ProductQueryPolicy, ProductScaleBand, ratify_product_job, ratify_product_publication,
    ratify_query_snapshot_product,
};

const ROOT_STACK: NodeId = NodeId(1);
const PAINT_SOURCE: NodeId = NodeId(2);
const OUTPUT: NodeId = NodeId(3);

fn valid_document() -> DrawingDocument {
    let root_out = PortId(10);
    let output_in = PortId(30);
    let graph = GraphDefinition::new(
        GraphId::new(1),
        "drawing_ink_tile_test",
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

fn sample(sequence: u64, x: f64, y: f64, pressure: f32) -> StrokeSample {
    StrokeSample::new(CanvasCoordinate::new(x, y), sequence)
        .with_timestamp_micros(sequence * 100)
        .with_pressure(pressure)
        .with_tool_kind(StrokeToolKind::Pen)
}

fn stroke(samples: Vec<StrokeSample>) -> StrokeRecord {
    StrokeRecord::new(
        StrokeId::new(1),
        PaintTarget::StackEntry(LayerStackEntryId::new(1)),
        BrushId::new(1),
        ColorRgba::new(0.1, 0.2, 0.3, 1.0),
        samples,
        DrawingDocumentRevision::new(1),
    )
    .expect("stroke should be valid")
}

fn document_with_stroke(samples: Vec<StrokeSample>) -> DrawingDocument {
    let mut document = valid_document();
    document.strokes.push(stroke(samples));
    document
}

#[test]
fn ink_tile_forms_deterministic_payloads_and_product_contracts() {
    let document = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.7),
        sample(2, 180.0, 142.0, 0.9),
    ]);

    let formation = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::default());
    let repeated = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::default());

    assert!(formation.is_accepted(), "{:?}", formation.diagnostics);
    assert_eq!(formation, repeated);
    assert_eq!(formation.products.len(), 1);
    let product = &formation.products[0];
    assert_eq!(product.payload.width, 64);
    assert_eq!(product.payload.height, 64);
    assert!(product.payload.non_transparent_sample_count() > 0);

    let contracts = build_drawing_ink_tile_product_contracts(&document, &formation.products)
        .expect("formed products should produce product contracts");
    assert!(ratify_product_job(&contracts.product_job).is_accepted());
    let outcome = build_drawing_ink_tile_publication_outcome(&document, &formation.products, 1)
        .expect("formed products should produce publication outcome");
    assert!(ratify_product_publication(&outcome).is_accepted());
    let snapshot =
        drawing_ink_tile_query_snapshot_for_descriptor(contracts.output_descriptors[0].clone());
    assert_eq!(
        snapshot.requested_policy,
        ProductQueryPolicy::StrictCurrentOnly
    );
    assert!(ratify_query_snapshot_product(&snapshot).is_accepted());
}

#[test]
fn preview_tile_formation_uses_committed_raster_payload_behavior() {
    let samples = vec![sample(1, 128.0, 128.0, 0.7), sample(2, 180.0, 142.0, 0.9)];
    let committed = form_drawing_ink_tiles(
        &document_with_stroke(samples.clone()),
        DrawingTileFormationPolicy::default(),
    );
    let preview_document = valid_document();
    let preview_stroke = DrawingInkPreviewStroke::new(
        StrokeId::new(1),
        PaintTarget::StackEntry(LayerStackEntryId::new(1)),
        BrushId::new(1),
        ColorRgba::new(0.1, 0.2, 0.3, 1.0),
        preview_document.revision,
    )
    .with_samples(samples);
    let preview = form_drawing_ink_preview_tiles(
        &preview_document,
        &preview_stroke,
        DrawingTileFormationPolicy::default(),
    );

    assert!(committed.is_accepted(), "{:?}", committed.diagnostics);
    assert!(preview.is_accepted(), "{:?}", preview.diagnostics);
    assert_eq!(committed.products.len(), preview.products.len());
    assert_eq!(committed.products[0].payload, preview.products[0].payload);
    assert_ne!(
        committed.products[0].metadata.product_id,
        preview.products[0].metadata.product_id
    );
}

#[test]
fn preview_tile_formation_is_deterministic() {
    let document = valid_document();
    let preview_stroke = DrawingInkPreviewStroke::new(
        StrokeId::new(7),
        PaintTarget::StackEntry(LayerStackEntryId::new(1)),
        BrushId::new(1),
        ColorRgba::new(0.1, 0.2, 0.3, 1.0),
        document.revision,
    )
    .with_samples(vec![
        sample(1, 128.0, 128.0, 0.7),
        sample(2, 180.0, 142.0, 0.9),
    ]);

    let first = form_drawing_ink_preview_tiles(
        &document,
        &preview_stroke,
        DrawingTileFormationPolicy::default(),
    );
    let second = form_drawing_ink_preview_tiles(
        &document,
        &preview_stroke,
        DrawingTileFormationPolicy::default(),
    );

    assert!(first.is_accepted(), "{:?}", first.diagnostics);
    assert_eq!(first, second);
}

#[test]
fn per_tile_formation_matches_full_document_reference_for_same_tile() {
    let document = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.7),
        sample(2, 180.0, 142.0, 0.9),
        sample(3, 260.0, 180.0, 0.8),
    ]);
    let policy = DrawingTileFormationPolicy::default();

    let reference = form_drawing_ink_tiles(&document, policy);
    assert!(reference.is_accepted(), "{:?}", reference.diagnostics);
    let reference_product = reference
        .products
        .first()
        .expect("reference stroke should form at least one tile product");

    let bounded =
        form_drawing_ink_tiles_for_ids(&document, [reference_product.metadata.tile_id], policy);

    assert!(bounded.is_accepted(), "{:?}", bounded.diagnostics);
    assert_eq!(bounded.products.len(), 1);
    assert_eq!(
        bounded.products[0].metadata.tile_id,
        reference_product.metadata.tile_id
    );
    assert_eq!(bounded.products[0].payload, reference_product.payload);
}

#[test]
fn long_stroke_invalidation_is_deterministic_without_batch_rejection() {
    let document = document_with_stroke(vec![
        sample(1, 0.0, 0.0, 1.0),
        sample(2, 4096.0, 4096.0, 1.0),
    ]);
    let policy = DrawingTileFormationPolicy::default();

    let first = drawing_ink_tile_invalidation_for_strokes(&document, &document.strokes, policy);
    let second = drawing_ink_tile_invalidation_for_strokes(&document, &document.strokes, policy);

    assert!(first.is_accepted(), "{:?}", first.diagnostics);
    assert_eq!(first, second);
    assert!(
        first.tile_ids.len() > policy.max_affected_tiles,
        "whole-stroke invalidation may exceed one interactive batch"
    );
}

#[test]
fn requested_transparent_tiles_are_reported_as_cleared_tiles() {
    let document = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.7),
        sample(2, 180.0, 142.0, 0.9),
    ]);
    let policy = DrawingTileFormationPolicy::default();
    let empty_tile = CanvasTileId::new(TilePyramidLevel::new(0), 8, 8);

    let formation = form_drawing_ink_tiles_for_ids(&document, [empty_tile], policy);

    assert!(formation.is_accepted(), "{:?}", formation.diagnostics);
    assert!(formation.products.is_empty());
    assert_eq!(formation.cleared_tiles, vec![empty_tile]);
    assert!(
        formation.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DrawingTileFormationDiagnosticCode::EmptyPayload
        })
    );
}

#[test]
fn pressure_and_formation_version_change_deterministic_outputs() {
    let low_pressure = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.2),
        sample(2, 180.0, 142.0, 0.2),
    ]);
    let high_pressure = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.9),
        sample(2, 180.0, 142.0, 0.9),
    ]);

    let low = form_drawing_ink_tiles(&low_pressure, DrawingTileFormationPolicy::default());
    let high = form_drawing_ink_tiles(&high_pressure, DrawingTileFormationPolicy::default());
    assert_ne!(low.determinism_key, high.determinism_key);
    assert_ne!(
        low.products[0].payload.rgba8_premultiplied,
        high.products[0].payload.rgba8_premultiplied
    );

    let policy = DrawingTileFormationPolicy {
        formation_version: FormationVersion::new(
            DrawingTileFormationPolicy::default()
                .formation_version
                .raw()
                + 1,
        ),
        ..DrawingTileFormationPolicy::default()
    };
    let versioned = form_drawing_ink_tiles(&low_pressure, policy);
    assert_ne!(
        low.products[0].metadata.product_id,
        versioned.products[0].metadata.product_id
    );
}

#[test]
fn preview_and_final_tile_identity_include_quality_and_cache_identity() {
    let document = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.7),
        sample(2, 180.0, 142.0, 0.9),
    ]);

    let preview = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::preview());
    let final_quality =
        form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::final_quality());

    assert!(preview.is_accepted(), "{:?}", preview.diagnostics);
    assert!(
        final_quality.is_accepted(),
        "{:?}",
        final_quality.diagnostics
    );
    assert_eq!(
        preview.products[0].metadata.quality_class,
        ProductQualityClass::Preview
    );
    assert_eq!(
        final_quality.products[0].metadata.quality_class,
        ProductQualityClass::Final
    );
    assert_eq!(
        preview.products[0].payload.width,
        DEFAULT_INK_TILE_PIXEL_WIDTH
    );
    assert_eq!(
        final_quality.products[0].payload.width,
        DEFAULT_FINAL_INK_TILE_PIXEL_WIDTH
    );
    assert_ne!(preview.determinism_key, final_quality.determinism_key);
    assert_ne!(
        preview.products[0].metadata.product_id,
        final_quality.products[0].metadata.product_id
    );
    assert!(preview.products[0].cache_key.contains(":preview:"));
    assert!(final_quality.products[0].cache_key.contains(":final:"));

    let preview_descriptor = drawing_ink_tile_product_descriptor(&preview.products[0]);
    let final_descriptor = drawing_ink_tile_product_descriptor(&final_quality.products[0]);
    assert_eq!(preview_descriptor.scale_band, ProductScaleBand::Preview);
    assert_eq!(final_descriptor.scale_band, ProductScaleBand::Final);

    let preview_cache = drawing_ink_tile_product_cache_identity(&preview.products[0]);
    let final_cache = drawing_ink_tile_product_cache_identity(&final_quality.products[0]);
    assert_ne!(preview_cache.stable_key(), final_cache.stable_key());
    assert_eq!(preview_cache.formation_version.as_deref(), Some("2"));
    assert_eq!(final_cache.scale_band, ProductScaleBand::Final);
}

#[test]
fn brush_flow_and_edge_softness_change_deterministic_payloads() {
    let mut low_flow = document_with_stroke(vec![
        sample(1, 40.0, 128.0, 1.0),
        sample(2, 220.0, 128.0, 1.0),
    ]);
    low_flow.brushes[0].ink.flow = BrushRange::new(0.2, 0.2);
    low_flow.brushes[0].ink.edge_softness = 0.2;

    let mut high_flow = document_with_stroke(vec![
        sample(1, 40.0, 128.0, 1.0),
        sample(2, 220.0, 128.0, 1.0),
    ]);
    high_flow.brushes[0].ink.flow = BrushRange::new(1.0, 1.0);
    high_flow.brushes[0].ink.edge_softness = 0.8;

    let low = form_drawing_ink_tiles(&low_flow, DrawingTileFormationPolicy::default());
    let high = form_drawing_ink_tiles(&high_flow, DrawingTileFormationPolicy::default());

    assert!(low.is_accepted(), "{:?}", low.diagnostics);
    assert!(high.is_accepted(), "{:?}", high.diagnostics);
    assert_ne!(low.determinism_key, high.determinism_key);
    assert_ne!(low.products[0].payload, high.products[0].payload);
    assert!(
        alpha_sum(&high.products[0].payload) > alpha_sum(&low.products[0].payload),
        "higher brush flow should deposit more ink"
    );
}

#[test]
fn sparse_fast_segment_deposits_ink_between_input_samples() {
    let document = document_with_stroke(vec![
        sample(1, 40.0, 128.0, 1.0),
        sample(2, 220.0, 128.0, 1.0),
    ]);

    let formation = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::default());

    assert!(formation.is_accepted(), "{:?}", formation.diagnostics);
    let payload = &formation.products[0].payload;
    let row = 32;
    assert!(
        max_transparent_gap_in_row(payload, row) <= 2,
        "a fast sparse stroke must form a continuous dab-spaced ink path, not only endpoint blobs"
    );
}

#[test]
fn oversized_tile_sets_reject_with_diagnostics() {
    let document = document_with_stroke(vec![
        sample(1, 0.0, 0.0, 1.0),
        sample(2, 4096.0, 4096.0, 1.0),
    ]);

    let formation = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::default());

    assert!(formation.products.is_empty());
    assert!(formation.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DrawingTileFormationDiagnosticCode::TooManyAffectedTiles
    }));
}

fn alpha_sum(payload: &DrawingInkTilePayload) -> u64 {
    payload
        .rgba8_premultiplied
        .chunks_exact(4)
        .map(|pixel| u64::from(pixel[3]))
        .sum()
}

fn max_transparent_gap_in_row(payload: &DrawingInkTilePayload, row: u32) -> usize {
    let row_start = row as usize * payload.width as usize * 4;
    let row_end = row_start + payload.width as usize * 4;
    let alphas = payload.rgba8_premultiplied[row_start..row_end]
        .chunks_exact(4)
        .map(|pixel| pixel[3])
        .collect::<Vec<_>>();
    let Some(first) = alphas.iter().position(|alpha| *alpha > 0) else {
        return payload.width as usize;
    };
    let last = alphas
        .iter()
        .rposition(|alpha| *alpha > 0)
        .expect("first non-transparent alpha exists");
    let mut current = 0usize;
    let mut max_gap = 0usize;
    for alpha in &alphas[first..=last] {
        if *alpha == 0 {
            current = current.saturating_add(1);
            max_gap = max_gap.max(current);
        } else {
            current = 0;
        }
    }
    max_gap
}

#[test]
fn unsupported_eraser_only_strokes_reject_without_products() {
    let mut eraser_sample = sample(1, 128.0, 128.0, 0.8);
    eraser_sample.tool_kind = Some(StrokeToolKind::Eraser);
    let mut second = sample(2, 132.0, 132.0, 0.8);
    second.tool_kind = Some(StrokeToolKind::Eraser);
    let document = document_with_stroke(vec![eraser_sample, second]);

    let formation = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::default());

    assert!(formation.products.is_empty());
    assert!(formation.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DrawingTileFormationDiagnosticCode::UnsupportedEraser
    }));
}

#[test]
fn invalid_documents_reject_with_diagnostics() {
    let mut document = document_with_stroke(vec![
        sample(1, 128.0, 128.0, 0.8),
        sample(2, 132.0, 132.0, 0.8),
    ]);
    document.display_name.clear();

    let formation = form_drawing_ink_tiles(&document, DrawingTileFormationPolicy::default());

    assert!(formation.products.is_empty());
    assert!(formation.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DrawingTileFormationDiagnosticCode::InvalidDocument
    }));
}
