//! File: domain/editor/editor_viewport/src/overlay.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportOverlay {
    Grid,
    OriginAxes,
    Bounds,
    SelectionOutline,
    DebugText(String),
}
