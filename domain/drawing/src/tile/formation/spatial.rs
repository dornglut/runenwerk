//! Private tile coverage and stroke-bound geometry helpers for ink formation.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BrushDescriptor, BrushId, CanvasCoordinate, CanvasRect, CanvasTileId, StrokeRecord,
    TilePyramidLevel,
};

use super::DrawingTileFormationPolicy;

pub(super) fn affected_tiles(
    strokes: &[&StrokeRecord],
    brushes: &BTreeMap<BrushId, &BrushDescriptor>,
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
                tiles.insert(CanvasTileId::new(TilePyramidLevel::new(0), x, y));
            }
        }
    }
    tiles.into_iter().collect()
}

pub(super) fn expanded_stroke_bounds(
    stroke: &StrokeRecord,
    brushes: &BTreeMap<BrushId, &BrushDescriptor>,
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

pub(super) fn tile_bounds(tile_id: CanvasTileId, tile_size: f64) -> CanvasRect {
    let min_x = tile_id.x as f64 * tile_size;
    let min_y = tile_id.y as f64 * tile_size;
    CanvasRect::new(
        CanvasCoordinate::new(min_x, min_y),
        CanvasCoordinate::new(min_x + tile_size, min_y + tile_size),
    )
}

pub(super) fn rects_intersect(a: CanvasRect, b: CanvasRect) -> bool {
    a.min.x <= b.max.x && a.max.x >= b.min.x && a.min.y <= b.max.y && a.max.y >= b.min.y
}

