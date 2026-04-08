//! File: domain/ui/ui_input/src/pointer.rs
//! Purpose: Pointer/mouse event primitives.

use ui_math::{UiPoint, UiVector};

pub type PointerPosition = UiPoint;
pub type PointerDelta = UiVector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEventKind {
    Move,
    Down,
    Up,
    Enter,
    Leave,
    Scroll,
}
