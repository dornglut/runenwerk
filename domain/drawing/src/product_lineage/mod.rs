//! File: domain/drawing/src/product_lineage/mod.rs
//! Purpose: Source lineage contracts for derived drawing products.

mod source_map;

pub use source_map::{
    BrushLineageRef, DrawingProductLineage, PaperLineageRef, SourceRange, StrokeLineageRange,
};
