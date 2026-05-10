//! File: domain/drawing/src/paper/height_source.rs
//! Purpose: Authored paper height source references.

#[derive(Debug, Clone, PartialEq)]
pub enum PaperHeightSource {
    None,
    ProceduralNoise {
        seed: u64,
        scale: f32,
        amplitude: f32,
    },
    FormedProductReference {
        product_ref: String,
    },
    ImportedHeightField {
        asset_ref: String,
    },
    SdfDerived {
        source_ref: String,
    },
}
