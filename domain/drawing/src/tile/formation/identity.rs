//! Private deterministic cache, lineage, descriptor, and formation-input identity helpers.

use crate::{
    BrushLineageRef, CanvasTileId, DrawingDocument, DrawingDocumentRevision, DrawingProductLineage,
    PaintTarget, PaperLineageRef, StrokeLineageRange, StrokeRecord, StrokeToolKind,
};

use super::super::determinism::StableDrawingHasher;
use super::{
    DrawingInkTileFormationKind, DrawingInkTilePayload, DrawingTileFormationPolicy,
    active_output_id,
};

pub(super) fn drawing_tile_determinism_key_for_records(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
    strokes: &[StrokeRecord],
) -> String {
    let mut hasher = StableDrawingHasher::new();
    hash_formation_inputs(&mut hasher, document, policy, kind, strokes);
    format!("{:016x}", hasher.finish())
}

pub(super) fn formation_key_with_requested_tiles(
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

pub(super) fn drawing_ink_tile_source_cache_key(
    document: &DrawingDocument,
    policy: DrawingTileFormationPolicy,
    kind: DrawingInkTileFormationKind,
    tile_id: CanvasTileId,
) -> Option<String> {
    if !policy.is_valid() {
        return None;
    }
    let source_output = active_output_id(document)?;
    Some(format!(
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
    ))
}

pub(super) fn lineage_for_strokes(
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

pub(super) fn ink_tile_descriptor_generation(
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
    hasher.write_str(policy.quality_class.cache_token());
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
