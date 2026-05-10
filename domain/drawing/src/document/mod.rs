//! File: domain/drawing/src/document/mod.rs
//! Purpose: Drawing document root contracts.

mod model;
mod revision;

pub use model::{DRAWING_DOCUMENT_SCHEMA_VERSION, DrawingDocument};
pub use revision::DrawingDocumentRevision;
