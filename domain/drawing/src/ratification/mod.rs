//! File: domain/drawing/src/ratification/mod.rs
//! Purpose: Drawing domain ratification reports.

mod ratifier;

pub use ratifier::{
    DrawingIssueCode, DrawingIssueSubject, DrawingRatificationReport, ratify_drawing_document,
};
