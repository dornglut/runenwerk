//! File: domain/drawing/src/tile/formation.rs
//! Purpose: Deterministic CPU ink tile formation from committed drawing strokes.

use std::collections::{BTreeMap, BTreeSet};

use ratification::RatificationSeverity;

use crate::{
    BrushDescriptor, BrushId, BrushLineageRef, CanvasCoordinate, CanvasRect, CanvasTileId,
    ColorRgba, CompositeOutputId, DrawingDocument, DrawingDocumentRevision, DrawingProductLineage,
    DrawingRatificationReport, DrawingTileProduct, DrawingTileProductId, DrawingTileProductSource,
    FormationVersion, PaintTarget, PaperLineageRef, ProductQualityClass, StrokeId,
    StrokeLineageRange, StrokeRecord, StrokeSample, StrokeToolKind,
};

use super::determinism::StableDrawingHasher;

pub const DEFAULT_INK_TILE_SIZE_CANVAS_UNITS: f64 = 256.0;
pub const DEFAULT_INK_TILE_PIXEL_WIDTH: u32 = 64;
pub const DEFAULT_INK_TILE_PIXEL_HEIGHT: u32 = 64;
pub const DEFAULT_MAX_AFFECTED_INK_TILES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawingTileFormationPolicy {
    pub quality_class: ProductQualityClass,
    pub formation_version: FormationVersion,
    pub tile_size_canvas_units: f64,
    pub tile_pixel_width: u32,
    pub tile_pixel_height: u32,
    pub max_affected_tiles: usize,
}

impl Default for DrawingTileFormationPolicy {
    fn default() -> Self {
        Self {
            quality_class: ProductQualityClass::Preview,
            formation_version: FormationVersion::new(2),
            tile_size_canvas_units: DEFAULT_INK_TILE_SIZE_CANVAS_UNITS,
            tile_pixel_width: DEFAULT_INK_TILE_PIXEL_WIDTH,
            tile_pixel_height: DEFAULT_INK_TILE_PIXEL_HEIGHT,
            max_affected_tiles: DEFAULT_MAX_AFFECTED_INK_TILES,
        }
    }
}

impl DrawingTileFormationPolicy {
    pub fn is_valid(self) -> bool {
        self.formation_version.raw() > 0
            && self.tile_size_canvas_units.is_finite()
            && self.tile_size_canvas_units > 0.0
            && self.tile_pixel_width > 0
            && self.tile_pixel_height > 0
            && self.max_affected_tiles > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawingTileFormationDiagnosticCode {
    InvalidDocument,
    InvalidPolicy,
    MissingCompositeOutput,
    NoSupportedStroke,
    UnsupportedEraser,
    TooManyAffectedTiles,
    EmptyPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingTileFormationDiagnostic {
    pub code: DrawingTileFormationDiagnosticCode,
    pub severity: RatificationSeverity,
    pub message: String,
}

impl DrawingTileFormationDiagnostic {
    pub fn blocking(code: DrawingTileFormationDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: RatificationSeverity::Error,
            message: message.into(),
        }
    }

    pub fn warning(code: DrawingTileFormationDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: RatificationSeverity::Warning,
            message: message.into(),
        }
    }

    pub fn is_blocking(&self) -> bool {
        self.severity.is_blocking()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingInkTilePayload {
    pub width: u32,
    pub height: u32,
    pub rgba8_premultiplied: Vec<u8>,
}

impl DrawingInkTilePayload {
    pub fn new(width: u32, height: u32, rgba8_premultiplied: Vec<u8>) -> Self {
        Self {
            width,
            height,
            rgba8_premultiplied,
        }
    }

    pub fn sample_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn byte_len(&self) -> usize {
        self.rgba8_premultiplied.len()
    }

    pub fn non_transparent_sample_count(&self) -> usize {
        self.rgba8_premultiplied
            .chunks_exact(4)
            .filter(|pixel| pixel[3] != 0)
            .count()
    }

    pub fn is_transparent(&self) -> bool {
        self.non_transparent_sample_count() == 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingInkTileProduct {
    pub metadata: DrawingTileProduct,
    pub payload: DrawingInkTilePayload,
    pub cache_key: String,
    pub descriptor_generation: u64,
    pub diagnostics: Vec<DrawingTileFormationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingInkTileFormation {
    pub products: Vec<DrawingInkTileProduct>,
    pub cleared_tiles: Vec<CanvasTileId>,
    pub diagnostics: Vec<DrawingTileFormationDiagnostic>,
    pub determinism_key: String,
}

impl DrawingInkTileFormation {
    pub fn is_accepted(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(DrawingTileFormationDiagnostic::is_blocking)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingInkTileInvalidation {
    pub tile_ids: Vec<CanvasTileId>,
    pub diagnostics: Vec<DrawingTileFormationDiagnostic>,
    pub determinism_key: String,
}

impl DrawingInkTileInvalidation {
    pub fn is_accepted(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(DrawingTileFormationDiagnostic::is_blocking)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingInkPreviewStroke {
    pub stroke_id: StrokeId,
    pub target: PaintTarget,
    pub brush_id: BrushId,
    pub color: ColorRgba,
    pub samples: Vec<StrokeSample>,
    pub source_revision: DrawingDocumentRevision,
}

impl DrawingInkPreviewStroke {
    pub fn new(
        stroke_id: StrokeId,
        target: PaintTarget,
        brush_id: BrushId,
        color: ColorRgba,
        source_revision: DrawingDocumentRevision,
    ) -> Self {
        Self {
            stroke_id,
            target,
            brush_id,
            color,
            samples: Vec::new(),
            source_revision,
        }
    }

    pub fn with_samples(mut self, samples: impl IntoIterator<Item = StrokeSample>) -> Self {
        self.samples = samples.into_iter().collect();
        self
    }

    pub fn append_sample(&mut self, sample: StrokeSample) {
        self.samples.push(sample);
    }

    fn to_stroke_record(&self) -> Option<StrokeRecord> {
        StrokeRecord::new(
            self.stroke_id,
            self.target,
            self.brush_id,
            self.color,
            self.samples.iter().copied(),
            self.source_revision,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawingInkTileFormationKind {
    Committed,
    Preview,
}

impl DrawingInkTileFormationKind {
    fn hash_tag(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Preview => "preview",
        }
    }

    fn cache_key_prefix(self) -> &'static str {
        match self {
            Self::Committed => "drawing.ink_tile",
            Self::Preview => "drawing.ink_tile.preview",
        }
    }

    fn no_supported_message(self) -> &'static str {
        match self {
            Self::Committed => "drawing ink tile formation found no supported committed strokes",
            Self::Preview => "drawing ink preview tile formation found no supported stroke",
        }
    }

    fn formation_name(self) -> &'static str {
        match self {
            Self::Committed => "drawing ink tile formation",
            Self::Preview => "drawing ink preview tile formation",
        }
    }
}

pub fn form_drawing_ink_tiles(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
) -> DrawingInkTileFormation {
    form_drawing_ink_tile_records(
        document,
        document.strokes.clone(),
        policy,
        DrawingInkTileFormationKind::Committed,
    )
}

pub fn form_drawing_ink_tiles_for_ids(
    document: &DrawingDocument,
    tile_ids: impl IntoIterator<Item = CanvasTileId>,
    policy: DrawingTileFormationPolicy,
) -> DrawingInkTileFormation {
    form_drawing_ink_tile_records_for_ids(
        document,
        document.strokes.clone(),
        tile_ids,
        policy,
        DrawingInkTileFormationKind::Committed,
    )
}

pub fn form_drawing_ink_preview_tiles(
    document: &DrawingDocument,
    preview_stroke: &DrawingInkPreviewStroke,
    policy: DrawingTileFormationPolicy,
) -> DrawingInkTileFormation {
    let strokes = preview_stroke
        .to_stroke_record()
        .into_iter()
        .collect::<Vec<_>>();
    form_drawing_ink_tile_records(
        document,
        strokes,
        policy,
        DrawingInkTileFormationKind::Preview,
    )
}

pub fn form_drawing_ink_preview_tiles_for_ids(
    document: &DrawingDocument,
    preview_stroke: &DrawingInkPreviewStroke,
    tile_ids: impl IntoIterator<Item = CanvasTileId>,
    policy: DrawingTileFormationPolicy,
) -> DrawingInkTileFormation {
    let strokes = preview_stroke
        .to_stroke_record()
        .into_iter()
        .collect::<Vec<_>>();
    form_drawing_ink_tile_records_for_ids(
        document,
        strokes,
        tile_ids,
        policy,
        DrawingInkTileFormationKind::Preview,
    )
}

pub fn drawing_ink_tile_invalidation_for_strokes(
    document: &DrawingDocument,
    strokes: &[StrokeRecord],
    policy: DrawingTileFormationPolicy,
) -> DrawingInkTileInvalidation {
    let determinism_key = drawing_tile_determinism_key_for_records(
        document,
        policy,
        DrawingInkTileFormationKind::Committed,
        strokes,
    );
    let mut diagnostics = Vec::new();

    if !policy.is_valid() {
        diagnostics.push(DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::InvalidPolicy,
            "drawing ink tile invalidation policy is invalid",
        ));
        return DrawingInkTileInvalidation {
            tile_ids: Vec::new(),
            diagnostics,
            determinism_key,
        };
    }

    let report = crate::ratify_drawing_document(document);
    if report.has_blocking_issues() {
        diagnostics.extend(report_to_diagnostics(&report));
        return DrawingInkTileInvalidation {
            tile_ids: Vec::new(),
            diagnostics,
            determinism_key,
        };
    }

    if strokes.is_empty() {
        return DrawingInkTileInvalidation {
            tile_ids: Vec::new(),
            diagnostics,
            determinism_key,
        };
    }

    let brush_by_id = document
        .brushes
        .iter()
        .map(|brush| (brush.brush_id, brush))
        .collect::<BTreeMap<_, _>>();
    let supported_strokes = supported_strokes(strokes, &mut diagnostics);
    if supported_strokes.is_empty() {
        diagnostics.push(DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::NoSupportedStroke,
            "drawing ink tile invalidation found no supported stroke",
        ));
        return DrawingInkTileInvalidation {
            tile_ids: Vec::new(),
            diagnostics,
            determinism_key,
        };
    }

    DrawingInkTileInvalidation {
        tile_ids: affected_tiles(&supported_strokes, &brush_by_id, policy),
        diagnostics,
        determinism_key,
    }
}

pub fn drawing_ink_tile_invalidation_for_preview_stroke(
    document: &DrawingDocument,
    preview_stroke: &DrawingInkPreviewStroke,
    policy: DrawingTileFormationPolicy,
) -> DrawingInkTileInvalidation {
    let strokes = preview_stroke
        .to_stroke_record()
        .into_iter()
        .collect::<Vec<_>>();
    drawing_ink_tile_invalidation_for_strokes(document, &strokes, policy)
}

fn form_drawing_ink_tile_records(
    document: &DrawingDocument,
    strokes: Vec<StrokeRecord>,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
) -> DrawingInkTileFormation {
    form_drawing_ink_tile_records_inner(document, strokes, None, policy, kind)
}

fn form_drawing_ink_tile_records_for_ids(
    document: &DrawingDocument,
    strokes: Vec<StrokeRecord>,
    tile_ids: impl IntoIterator<Item = CanvasTileId>,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
) -> DrawingInkTileFormation {
    let tile_ids = tile_ids.into_iter().collect::<Vec<_>>();
    form_drawing_ink_tile_records_inner(document, strokes, Some(tile_ids), policy, kind)
}

fn form_drawing_ink_tile_records_inner(
    document: &DrawingDocument,
    strokes: Vec<StrokeRecord>,
    requested_tile_ids: Option<Vec<CanvasTileId>>,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
) -> DrawingInkTileFormation {
    let determinism_key = formation_key_with_requested_tiles(
        drawing_tile_determinism_key_for_records(document, policy, kind, &strokes),
        requested_tile_ids.as_deref(),
    );
    let mut diagnostics = Vec::new();

    if !policy.is_valid() {
        diagnostics.push(DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::InvalidPolicy,
            format!("{} policy is invalid", kind.formation_name()),
        ));
        return empty_formation(determinism_key, diagnostics);
    }

    let report = crate::ratify_drawing_document(document);
    if report.has_blocking_issues() {
        diagnostics.extend(report_to_diagnostics(&report));
        return empty_formation(determinism_key, diagnostics);
    }

    let Some(source_output) = active_output_id(document) else {
        diagnostics.push(DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::MissingCompositeOutput,
            format!(
                "{} requires an active composite output",
                kind.formation_name()
            ),
        ));
        return empty_formation(determinism_key, diagnostics);
    };

    if strokes.is_empty() {
        return DrawingInkTileFormation {
            products: Vec::new(),
            cleared_tiles: Vec::new(),
            diagnostics,
            determinism_key,
        };
    }

    let brush_by_id = document
        .brushes
        .iter()
        .map(|brush| (brush.brush_id, brush))
        .collect::<BTreeMap<_, _>>();
    let supported_strokes = supported_strokes(&strokes, &mut diagnostics);
    if supported_strokes.is_empty() {
        diagnostics.push(DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::NoSupportedStroke,
            kind.no_supported_message(),
        ));
        return empty_formation(determinism_key, diagnostics);
    }

    let affected_tiles = requested_tile_ids
        .unwrap_or_else(|| affected_tiles(&supported_strokes, &brush_by_id, policy));
    if affected_tiles.len() > policy.max_affected_tiles {
        diagnostics.push(DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::TooManyAffectedTiles,
            format!(
                "{} touched {} tiles, exceeding the {} tile limit",
                kind.formation_name(),
                affected_tiles.len(),
                policy.max_affected_tiles
            ),
        ));
        return empty_formation(determinism_key, diagnostics);
    }

    let mut products = Vec::new();
    let mut cleared_tiles = Vec::new();
    for tile_id in affected_tiles {
        let tile_bounds = tile_bounds(tile_id, policy.tile_size_canvas_units);
        let mut payload = DrawingInkTilePayload::new(
            policy.tile_pixel_width,
            policy.tile_pixel_height,
            vec![0; policy.tile_pixel_width as usize * policy.tile_pixel_height as usize * 4],
        );
        for stroke in &supported_strokes {
            if !rects_intersect(expanded_stroke_bounds(stroke, &brush_by_id), tile_bounds) {
                continue;
            }
            if let Some(brush) = brush_by_id.get(&stroke.brush_id).copied() {
                rasterize_stroke(&mut payload, tile_bounds, stroke, brush, policy);
            }
        }
        if payload.is_transparent() {
            cleared_tiles.push(tile_id);
            continue;
        }

        let descriptor_generation =
            ink_tile_descriptor_generation(document, policy, kind, &strokes, tile_id, &payload);
        let product_id = DrawingTileProductId::new(descriptor_generation);
        let metadata = DrawingTileProduct::new(
            product_id,
            tile_id,
            DrawingTileProductSource::new(
                policy.quality_class,
                document.revision,
                source_output,
                lineage_for_strokes(document, &strokes),
                policy.formation_version,
                tile_bounds,
            ),
        );
        products.push(DrawingInkTileProduct {
            metadata,
            payload,
            cache_key: format!(
                "{}:{}:{}:{}:{}",
                kind.cache_key_prefix(),
                document.document_id.raw(),
                document.revision.raw(),
                tile_id.x,
                tile_id.y
            ),
            descriptor_generation,
            diagnostics: Vec::new(),
        });
    }

    if !supported_strokes.is_empty() && products.is_empty() {
        diagnostics.push(DrawingTileFormationDiagnostic::warning(
            DrawingTileFormationDiagnosticCode::EmptyPayload,
            format!("{} produced no visible ink samples", kind.formation_name()),
        ));
    }

    products.sort_by_key(|product| {
        (
            product.metadata.tile_id.level.raw(),
            product.metadata.tile_id.x,
            product.metadata.tile_id.y,
            product.metadata.product_id.raw(),
        )
    });

    DrawingInkTileFormation {
        products,
        cleared_tiles,
        diagnostics,
        determinism_key,
    }
}

pub fn drawing_tile_determinism_key(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
) -> String {
    drawing_tile_determinism_key_for_records(
        document,
        policy,
        DrawingInkTileFormationKind::Committed,
        &document.strokes,
    )
}

fn drawing_tile_determinism_key_for_records(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
    strokes: &[StrokeRecord],
) -> String {
    let mut hasher = StableDrawingHasher::new();
    hash_formation_inputs(&mut hasher, document, policy, kind, strokes);
    format!("{:016x}", hasher.finish())
}

fn empty_formation(
    determinism_key: String,
    diagnostics: Vec<DrawingTileFormationDiagnostic>,
) -> DrawingInkTileFormation {
    DrawingInkTileFormation {
        products: Vec::new(),
        cleared_tiles: Vec::new(),
        diagnostics,
        determinism_key,
    }
}

fn formation_key_with_requested_tiles(
    base: String,
    requested_tile_ids: Option<&[CanvasTileId]>,
) -> String {
    let Some(tile_ids) = requested_tile_ids else {
        return base;
    };
    let mut tile_parts = tile_ids
        .iter()
        .map(|tile_id| format!("{}:{}:{}", tile_id.level.raw(), tile_id.x, tile_id.y))
        .collect::<Vec<_>>();
    tile_parts.sort();
    format!("{base}:tiles={}", tile_parts.join(","))
}

fn report_to_diagnostics(
    report: &DrawingRatificationReport,
) -> Vec<DrawingTileFormationDiagnostic> {
    report
        .iter()
        .map(|issue| {
            DrawingTileFormationDiagnostic::blocking(
                DrawingTileFormationDiagnosticCode::InvalidDocument,
                format!(
                    "drawing document rejected: {:?}: {}",
                    issue.code(),
                    issue.message()
                ),
            )
        })
        .collect()
}

fn active_output_id(document: &DrawingDocument) -> Option<CompositeOutputId> {
    document.composition.active_output.or_else(|| {
        document
            .composition
            .nodes
            .values()
            .find_map(|node| match node {
                crate::DrawingCompositeNode::CompositeOutput(output) => Some(output.output_id),
                _ => None,
            })
    })
}

fn supported_strokes<'a>(
    strokes: &'a [StrokeRecord],
    diagnostics: &mut Vec<DrawingTileFormationDiagnostic>,
) -> Vec<&'a StrokeRecord> {
    strokes
        .iter()
        .filter(|stroke| {
            if stroke
                .samples
                .iter()
                .all(|sample| sample.tool_kind == Some(StrokeToolKind::Eraser))
            {
                diagnostics.push(DrawingTileFormationDiagnostic::blocking(
                    DrawingTileFormationDiagnosticCode::UnsupportedEraser,
                    format!(
                        "stroke {} uses only eraser samples; eraser compositing is deferred",
                        stroke.stroke_id.raw()
                    ),
                ));
                return false;
            }
            true
        })
        .collect()
}

fn affected_tiles(
    strokes: &[&StrokeRecord],
    brushes: &BTreeMap<crate::BrushId, &BrushDescriptor>,
    policy: DrawingTileFormationPolicy,
) -> Vec<CanvasTileId> {
    let mut tiles = BTreeSet::new();
    for stroke in strokes {
        let bounds = expanded_stroke_bounds(stroke, brushes);
        let min_x = (bounds.min.x / policy.tile_size_canvas_units).floor() as i64;
        let max_x = (bounds.max.x / policy.tile_size_canvas_units).floor() as i64;
        let min_y = (bounds.min.y / policy.tile_size_canvas_units).floor() as i64;
        let max_y = (bounds.max.y / policy.tile_size_canvas_units).floor() as i64;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                tiles.insert(CanvasTileId::new(crate::TilePyramidLevel::new(0), x, y));
            }
        }
    }
    tiles.into_iter().collect()
}

fn expanded_stroke_bounds(
    stroke: &StrokeRecord,
    brushes: &BTreeMap<crate::BrushId, &BrushDescriptor>,
) -> CanvasRect {
    let radius = brushes
        .get(&stroke.brush_id)
        .map(|brush| f64::from(brush.ink.size.max.max(1.0)) * 0.5)
        .unwrap_or(0.5);
    CanvasRect::new(
        CanvasCoordinate::new(stroke.bounds.min.x - radius, stroke.bounds.min.y - radius),
        CanvasCoordinate::new(stroke.bounds.max.x + radius, stroke.bounds.max.y + radius),
    )
}

fn tile_bounds(tile_id: CanvasTileId, tile_size: f64) -> CanvasRect {
    let min_x = tile_id.x as f64 * tile_size;
    let min_y = tile_id.y as f64 * tile_size;
    CanvasRect::new(
        CanvasCoordinate::new(min_x, min_y),
        CanvasCoordinate::new(min_x + tile_size, min_y + tile_size),
    )
}

fn rects_intersect(a: CanvasRect, b: CanvasRect) -> bool {
    a.min.x <= b.max.x && a.max.x >= b.min.x && a.min.y <= b.max.y && a.max.y >= b.min.y
}

fn rasterize_stroke(
    payload: &mut DrawingInkTilePayload,
    tile_bounds: CanvasRect,
    stroke: &StrokeRecord,
    brush: &BrushDescriptor,
    policy: DrawingTileFormationPolicy,
) {
    let context = StrokeRasterContext {
        tile_bounds,
        stroke,
        brush,
        policy,
    };
    match stroke.samples.as_slice() {
        [] => {}
        [sample] => {
            let radius = f64::from(sample_size(brush, sample) * 0.5).max(0.5);
            rasterize_dab(
                payload,
                context,
                DabRaster {
                    center: sample.position,
                    radius,
                    opacity: sample_opacity(brush, sample),
                    flow: sample_flow(brush, sample),
                    edge_softness: brush.ink.edge_softness,
                },
            );
        }
        samples => {
            for (index, pair) in samples.windows(2).enumerate() {
                rasterize_segment_dabs(payload, context, &pair[0], &pair[1], index == 0);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct StrokeRasterContext<'a> {
    tile_bounds: CanvasRect,
    stroke: &'a StrokeRecord,
    brush: &'a BrushDescriptor,
    policy: DrawingTileFormationPolicy,
}

#[derive(Clone, Copy)]
struct DabRaster {
    center: CanvasCoordinate,
    radius: f64,
    opacity: f32,
    flow: f32,
    edge_softness: f32,
}

fn rasterize_segment_dabs(
    payload: &mut DrawingInkTilePayload,
    context: StrokeRasterContext<'_>,
    start: &StrokeSample,
    end: &StrokeSample,
    include_start: bool,
) {
    let brush = context.brush;
    let start_radius = f64::from(sample_size(brush, start) * 0.5).max(0.5);
    let end_radius = f64::from(sample_size(brush, end) * 0.5).max(0.5);
    let max_radius = start_radius.max(end_radius);
    let segment_bounds = CanvasRect::new(
        CanvasCoordinate::new(
            start.position.x.min(end.position.x) - max_radius,
            start.position.y.min(end.position.y) - max_radius,
        ),
        CanvasCoordinate::new(
            start.position.x.max(end.position.x) + max_radius,
            start.position.y.max(end.position.y) + max_radius,
        ),
    );
    if !rects_intersect(segment_bounds, context.tile_bounds) {
        return;
    }

    let dx = end.position.x - start.position.x;
    let dy = end.position.y - start.position.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f64::EPSILON {
        if include_start {
            rasterize_dab(
                payload,
                context,
                DabRaster {
                    center: start.position,
                    radius: start_radius,
                    opacity: sample_opacity(brush, start),
                    flow: sample_flow(brush, start),
                    edge_softness: brush.ink.edge_softness,
                },
            );
        }
        return;
    }

    let clip_bounds = expanded_rect(context.tile_bounds, max_radius);
    let Some((clip_t0, clip_t1)) =
        segment_rect_parameter_range(start.position, end.position, clip_bounds)
    else {
        return;
    };
    let spacing = dab_spacing(brush, start, end);
    let min_index = if include_start { 0 } else { 1 };
    let first_index = ((clip_t0 * length - 0.000_001) / spacing)
        .ceil()
        .max(min_index as f64) as u32;
    let last_index = ((clip_t1 * length + 0.000_001) / spacing).floor() as u32;
    let mut drew_end = false;
    for index in first_index..=last_index {
        let distance = (f64::from(index) * spacing).min(length);
        let t = (distance / length).clamp(0.0, 1.0);
        if (1.0 - t).abs() <= 0.000_001 {
            drew_end = true;
        }
        rasterize_dab_at_t(payload, context, start, end, t);
    }
    if clip_t1 >= 1.0 - 0.000_001 && !drew_end {
        rasterize_dab_at_t(payload, context, start, end, 1.0);
    }
}

fn rasterize_dab_at_t(
    payload: &mut DrawingInkTilePayload,
    context: StrokeRasterContext<'_>,
    start: &StrokeSample,
    end: &StrokeSample,
    t: f64,
) {
    let brush = context.brush;
    let center = CanvasCoordinate::new(
        lerp_f64(start.position.x, end.position.x, t),
        lerp_f64(start.position.y, end.position.y, t),
    );
    let radius = lerp_f64(
        f64::from(sample_size(brush, start) * 0.5).max(0.5),
        f64::from(sample_size(brush, end) * 0.5).max(0.5),
        t,
    );
    let opacity = lerp_f32(sample_opacity(brush, start), sample_opacity(brush, end), t);
    let flow = lerp_f32(sample_flow(brush, start), sample_flow(brush, end), t);
    rasterize_dab(
        payload,
        context,
        DabRaster {
            center,
            radius,
            opacity,
            flow,
            edge_softness: brush.ink.edge_softness,
        },
    );
}

fn rasterize_dab(
    payload: &mut DrawingInkTilePayload,
    context: StrokeRasterContext<'_>,
    dab: DabRaster,
) {
    let radius = dab.radius.max(0.5);
    let min_x = dab.center.x - radius;
    let max_x = dab.center.x + radius;
    let min_y = dab.center.y - radius;
    let max_y = dab.center.y + radius;
    let dab_bounds = CanvasRect::new(
        CanvasCoordinate::new(min_x, min_y),
        CanvasCoordinate::new(max_x, max_y),
    );
    if !rects_intersect(dab_bounds, context.tile_bounds) {
        return;
    }

    let pixel_w = context.policy.tile_size_canvas_units / f64::from(payload.width);
    let pixel_h = context.policy.tile_size_canvas_units / f64::from(payload.height);
    let min_px = (((min_x - context.tile_bounds.min.x) / pixel_w).floor() as i64)
        .clamp(0, i64::from(payload.width.saturating_sub(1))) as u32;
    let max_px = (((max_x - context.tile_bounds.min.x) / pixel_w).floor() as i64)
        .clamp(0, i64::from(payload.width.saturating_sub(1))) as u32;
    let min_py = (((min_y - context.tile_bounds.min.y) / pixel_h).floor() as i64)
        .clamp(0, i64::from(payload.height.saturating_sub(1))) as u32;
    let max_py = (((max_y - context.tile_bounds.min.y) / pixel_h).floor() as i64)
        .clamp(0, i64::from(payload.height.saturating_sub(1))) as u32;

    let opacity = dab.opacity.clamp(0.0, 1.0)
        * dab.flow.clamp(0.0, 1.0)
        * context.stroke.color.a.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return;
    }
    let edge_softness = dab.edge_softness.clamp(0.0, 1.0);
    let fade_width = (radius * f64::from(edge_softness))
        .max(pixel_w.max(pixel_h) * 0.5)
        .min(radius);
    let hard_radius = (radius - fade_width).max(0.0);

    for py in min_py..=max_py {
        for px in min_px..=max_px {
            let pixel_center = CanvasCoordinate::new(
                context.tile_bounds.min.x + (f64::from(px) + 0.5) * pixel_w,
                context.tile_bounds.min.y + (f64::from(py) + 0.5) * pixel_h,
            );
            let distance = distance_between(pixel_center, dab.center);
            if distance > radius {
                continue;
            }
            let coverage = if distance <= hard_radius {
                1.0
            } else if fade_width <= f64::EPSILON {
                0.0
            } else {
                1.0 - ((distance - hard_radius) / fade_width).clamp(0.0, 1.0)
            } as f32;
            let alpha = (opacity * coverage).clamp(0.0, 1.0);
            blend_pixel(
                payload,
                px,
                py,
                context.stroke.color.r,
                context.stroke.color.g,
                context.stroke.color.b,
                alpha,
            );
        }
    }
}

fn sample_size(brush: &BrushDescriptor, sample: &StrokeSample) -> f32 {
    range_value(
        brush.ink.size.min,
        brush.ink.size.max,
        pressure_value(sample, brush.ink.dynamics.pressure_to_size),
    )
}

fn sample_opacity(brush: &BrushDescriptor, sample: &StrokeSample) -> f32 {
    range_value(
        brush.ink.opacity.min,
        brush.ink.opacity.max,
        pressure_value(sample, brush.ink.dynamics.pressure_to_opacity),
    )
}

fn sample_flow(brush: &BrushDescriptor, sample: &StrokeSample) -> f32 {
    range_value(
        brush.ink.flow.min,
        brush.ink.flow.max,
        pressure_value(sample, brush.ink.dynamics.pressure_to_opacity),
    )
}

fn pressure_value(sample: &StrokeSample, curve: crate::DynamicsCurve) -> f32 {
    let pressure = sample.pressure.unwrap_or(1.0).clamp(0.0, 1.0);
    if !curve.enabled {
        return pressure;
    }
    curve.minimum_scale + (1.0 - curve.minimum_scale) * pressure.powf(curve.gamma)
}

fn range_value(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}

fn dab_spacing(brush: &BrushDescriptor, start: &StrokeSample, end: &StrokeSample) -> f64 {
    let start_size = f64::from(sample_size(brush, start)).max(1.0);
    let end_size = f64::from(sample_size(brush, end)).max(1.0);
    (start_size.min(end_size) * 0.25).clamp(0.75, 8.0)
}

fn expanded_rect(rect: CanvasRect, amount: f64) -> CanvasRect {
    CanvasRect::new(
        CanvasCoordinate::new(rect.min.x - amount, rect.min.y - amount),
        CanvasCoordinate::new(rect.max.x + amount, rect.max.y + amount),
    )
}

fn segment_rect_parameter_range(
    start: CanvasCoordinate,
    end: CanvasCoordinate,
    rect: CanvasRect,
) -> Option<(f64, f64)> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let mut t0 = 0.0;
    let mut t1 = 1.0;
    clip_segment_axis(-dx, start.x - rect.min.x, &mut t0, &mut t1)?;
    clip_segment_axis(dx, rect.max.x - start.x, &mut t0, &mut t1)?;
    clip_segment_axis(-dy, start.y - rect.min.y, &mut t0, &mut t1)?;
    clip_segment_axis(dy, rect.max.y - start.y, &mut t0, &mut t1)?;
    Some((t0, t1))
}

fn clip_segment_axis(p: f64, q: f64, t0: &mut f64, t1: &mut f64) -> Option<()> {
    if p.abs() <= f64::EPSILON {
        return (q >= 0.0).then_some(());
    }
    let r = q / p;
    if p < 0.0 {
        if r > *t1 {
            return None;
        }
        if r > *t0 {
            *t0 = r;
        }
    } else {
        if r < *t0 {
            return None;
        }
        if r < *t1 {
            *t1 = r;
        }
    }
    Some(())
}

fn lerp_f64(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

fn lerp_f32(start: f32, end: f32, t: f64) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0) as f32
}

fn distance_between(a: CanvasCoordinate, b: CanvasCoordinate) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn blend_pixel(
    payload: &mut DrawingInkTilePayload,
    px: u32,
    py: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) {
    let index = ((py as usize * payload.width as usize + px as usize) * 4)
        .min(payload.rgba8_premultiplied.len().saturating_sub(4));
    let src_a = a.clamp(0.0, 1.0);
    let src_r = r.clamp(0.0, 1.0) * src_a;
    let src_g = g.clamp(0.0, 1.0) * src_a;
    let src_b = b.clamp(0.0, 1.0) * src_a;

    let dst_r = f32::from(payload.rgba8_premultiplied[index]) / 255.0;
    let dst_g = f32::from(payload.rgba8_premultiplied[index + 1]) / 255.0;
    let dst_b = f32::from(payload.rgba8_premultiplied[index + 2]) / 255.0;
    let dst_a = f32::from(payload.rgba8_premultiplied[index + 3]) / 255.0;
    let inv = 1.0 - src_a;

    payload.rgba8_premultiplied[index] = to_u8(src_r + dst_r * inv);
    payload.rgba8_premultiplied[index + 1] = to_u8(src_g + dst_g * inv);
    payload.rgba8_premultiplied[index + 2] = to_u8(src_b + dst_b * inv);
    payload.rgba8_premultiplied[index + 3] = to_u8(src_a + dst_a * inv);
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn lineage_for_strokes(
    document: &DrawingDocument,
    strokes: &[StrokeRecord],
) -> DrawingProductLineage {
    let mut lineage = DrawingProductLineage::new(document.revision);
    if let (Some(first), Some(last)) = (strokes.first(), strokes.last()) {
        lineage.stroke_range = Some(StrokeLineageRange::new(first.stroke_id, last.stroke_id));
    }
    lineage.brush_revisions = document
        .brushes
        .iter()
        .map(|brush| BrushLineageRef {
            brush_id: brush.brush_id,
            revision: brush.revision,
        })
        .collect();
    lineage.paper_revisions = document
        .papers
        .iter()
        .map(|paper| PaperLineageRef {
            paper_id: paper.paper_id,
            revision: paper.revision,
        })
        .collect();
    lineage
}

fn ink_tile_descriptor_generation(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
    strokes: &[StrokeRecord],
    tile_id: CanvasTileId,
    payload: &DrawingInkTilePayload,
) -> u64 {
    let mut hasher = StableDrawingHasher::new();
    hash_formation_inputs(&mut hasher, document, policy, kind, strokes);
    hasher.write_str("tile");
    hasher.write_u32(tile_id.level.raw());
    hasher.write_i64(tile_id.x);
    hasher.write_i64(tile_id.y);
    for byte in &payload.rgba8_premultiplied {
        hasher.write_u8(*byte);
    }
    hasher.finish()
}

fn hash_formation_inputs(
    hasher: &mut StableDrawingHasher,
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
    strokes: &[StrokeRecord],
) {
    hasher.write_str("drawing.ink_tile.v2");
    hasher.write_str(kind.hash_tag());
    hasher.write_u64(document.document_id.raw());
    hasher.write_u64(document.revision.raw());
    hasher.write_u32(document.schema_version);
    hasher.write_f64(document.canvas_bounds.min.x);
    hasher.write_f64(document.canvas_bounds.min.y);
    hasher.write_f64(document.canvas_bounds.max.x);
    hasher.write_f64(document.canvas_bounds.max.y);
    hasher.write_u32(policy.formation_version.raw());
    hasher.write_f64(policy.tile_size_canvas_units);
    hasher.write_u32(policy.tile_pixel_width);
    hasher.write_u32(policy.tile_pixel_height);
    hasher.write_u64(policy.max_affected_tiles as u64);
    for brush in &document.brushes {
        hasher.write_u64(brush.brush_id.raw());
        hasher.write_u64(brush.revision);
        hasher.write_f32(brush.ink.size.min);
        hasher.write_f32(brush.ink.size.max);
        hasher.write_f32(brush.ink.opacity.min);
        hasher.write_f32(brush.ink.opacity.max);
        hasher.write_f32(brush.ink.flow.min);
        hasher.write_f32(brush.ink.flow.max);
        hasher.write_f32(brush.ink.edge_softness);
        hasher.write_f32(brush.ink.viscosity);
        hasher.write_f32(brush.ink.absorption_response);
        hasher.write_bool(brush.ink.dynamics.pressure_to_size.enabled);
        hasher.write_f32(brush.ink.dynamics.pressure_to_size.minimum_scale);
        hasher.write_f32(brush.ink.dynamics.pressure_to_size.gamma);
        hasher.write_bool(brush.ink.dynamics.pressure_to_opacity.enabled);
        hasher.write_f32(brush.ink.dynamics.pressure_to_opacity.minimum_scale);
        hasher.write_f32(brush.ink.dynamics.pressure_to_opacity.gamma);
    }
    for stroke in strokes {
        hasher.write_u64(stroke.stroke_id.raw());
        match stroke.target {
            PaintTarget::StackEntry(entry_id) => {
                hasher.write_u8(1);
                hasher.write_u64(entry_id.raw());
            }
            PaintTarget::PaintSource(source_id) => {
                hasher.write_u8(2);
                hasher.write_u64(source_id.raw());
            }
        }
        hasher.write_u64(stroke.brush_id.raw());
        hash_revision(hasher, stroke.source_revision);
        hasher.write_f32(stroke.color.r);
        hasher.write_f32(stroke.color.g);
        hasher.write_f32(stroke.color.b);
        hasher.write_f32(stroke.color.a);
        for sample in &stroke.samples {
            hasher.write_u64(sample.sequence);
            hasher.write_f64(sample.position.x);
            hasher.write_f64(sample.position.y);
            if let Some(timestamp) = sample.timestamp_micros {
                hasher.write_bool(true);
                hasher.write_u64(timestamp);
            } else {
                hasher.write_bool(false);
            }
            if let Some(pressure) = sample.pressure {
                hasher.write_bool(true);
                hasher.write_f32(pressure);
            } else {
                hasher.write_bool(false);
            }
            hasher.write_u8(match sample.tool_kind {
                Some(StrokeToolKind::Pen) => 1,
                Some(StrokeToolKind::Brush) => 2,
                Some(StrokeToolKind::Marker) => 3,
                Some(StrokeToolKind::Eraser) => 4,
                Some(StrokeToolKind::Unknown) => 5,
                None => 0,
            });
        }
    }
}

fn hash_revision(hasher: &mut StableDrawingHasher, revision: DrawingDocumentRevision) {
    hasher.write_u64(revision.raw());
}
