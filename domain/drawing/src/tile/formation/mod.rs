//! File: domain/drawing/src/tile/formation/mod.rs
//! Purpose: Public deterministic ink-tile formation contracts and orchestration.

use std::collections::BTreeMap;

use ratification::RatificationSeverity;

use crate::{
    BrushDescriptor, BrushId, CanvasRect, CanvasTileId, ColorRgba, CompositeOutputId,
    DrawingDocument, DrawingDocumentRevision, DrawingRatificationReport, DrawingTileProduct,
    DrawingTileProductId, DrawingTileProductSource, FormationVersion, PaintTarget,
    ProductQualityClass, StrokeId, StrokeRecord, StrokeSample, StrokeToolKind,
};

mod identity;
mod raster;
mod spatial;

use identity::{
    drawing_ink_tile_source_cache_key, drawing_tile_determinism_key_for_records,
    formation_key_with_requested_tiles, ink_tile_descriptor_generation, lineage_for_strokes,
};
use raster::rasterize_stroke;
use spatial::{affected_tiles, expanded_stroke_bounds, rects_intersect, tile_bounds};

pub const DEFAULT_INK_TILE_SIZE_CANVAS_UNITS: f64 = 256.0;
pub const DEFAULT_INK_TILE_PIXEL_WIDTH: u32 = 64;
pub const DEFAULT_INK_TILE_PIXEL_HEIGHT: u32 = 64;
pub const DEFAULT_FINAL_INK_TILE_PIXEL_WIDTH: u32 = 256;
pub const DEFAULT_FINAL_INK_TILE_PIXEL_HEIGHT: u32 = 256;
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
    pub fn preview() -> Self {
        Self::default()
    }

    pub fn final_quality() -> Self {
        Self {
            quality_class: ProductQualityClass::Final,
            tile_pixel_width: DEFAULT_FINAL_INK_TILE_PIXEL_WIDTH,
            tile_pixel_height: DEFAULT_FINAL_INK_TILE_PIXEL_HEIGHT,
            ..Self::default()
        }
    }

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
            .as_chunks::<4>()
            .0
            .iter()
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

pub fn drawing_committed_ink_tile_source_cache_key(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
    tile_id: CanvasTileId,
) -> Option<String> {
    drawing_ink_tile_source_cache_key(
        document,
        policy,
        DrawingInkTileFormationKind::Committed,
        tile_id,
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
                "{}:{}:{}:{}:{}:L{}:{}:{}:v{}",
                kind.cache_key_prefix(),
                policy.quality_class.cache_token(),
                document.document_id.raw(),
                document.revision.raw(),
                source_output.raw(),
                tile_id.level.raw(),
                tile_id.x,
                tile_id.y,
                policy.formation_version.raw()
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
