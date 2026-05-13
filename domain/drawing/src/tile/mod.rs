//! File: domain/drawing/src/tile/mod.rs
//! Purpose: Canvas coordinate, tile identity, and formed product descriptor contracts.

mod coordinate;
mod determinism;
mod formation;
mod product;
mod product_contracts;

pub use coordinate::{CanvasCoordinate, CanvasRect, CanvasTileId, TilePyramidLevel};
pub(crate) use determinism::StableDrawingHasher;
pub use formation::{
    DEFAULT_INK_TILE_PIXEL_HEIGHT, DEFAULT_INK_TILE_PIXEL_WIDTH,
    DEFAULT_INK_TILE_SIZE_CANVAS_UNITS, DEFAULT_MAX_AFFECTED_INK_TILES, DrawingInkPreviewStroke,
    DrawingInkTileFormation, DrawingInkTileInvalidation, DrawingInkTilePayload,
    DrawingInkTileProduct, DrawingTileFormationDiagnostic, DrawingTileFormationDiagnosticCode,
    DrawingTileFormationPolicy, drawing_ink_tile_invalidation_for_preview_stroke,
    drawing_ink_tile_invalidation_for_strokes, drawing_tile_determinism_key,
    form_drawing_ink_preview_tiles, form_drawing_ink_preview_tiles_for_ids, form_drawing_ink_tiles,
    form_drawing_ink_tiles_for_ids,
};
pub use product::{DrawingTileProduct, DrawingTileProductSource, ProductQualityClass};
pub use product_contracts::{
    DRAWING_INK_TILE_JOB_KIND, DRAWING_INK_TILE_PRODUCER, DRAWING_INK_TILE_PRODUCT_KIND,
    DrawingInkTileProductContracts, build_drawing_ink_tile_product_contracts,
    build_drawing_ink_tile_publication_outcome, drawing_ink_tile_diagnostic_to_field_product,
    drawing_ink_tile_product_descriptor, drawing_ink_tile_query_snapshot_for_descriptor,
};
