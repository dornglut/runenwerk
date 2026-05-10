//! File: domain/drawing/src/tile/mod.rs
//! Purpose: Canvas coordinate, tile identity, and formed product descriptor contracts.

mod coordinate;
mod product;

pub use coordinate::{CanvasCoordinate, CanvasRect, CanvasTileId, TilePyramidLevel};
pub use product::{DrawingTileProduct, DrawingTileProductSource, ProductQualityClass};
