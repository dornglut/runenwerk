//! File: domain/drawing/src/paper/mod.rs
//! Purpose: Paper descriptor and height source contracts.

mod descriptor;
mod height_source;

pub use descriptor::PaperDescriptor;
pub use height_source::PaperHeightSource;
