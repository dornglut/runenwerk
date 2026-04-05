//! File: domain/editor/editor_tools/src/input.rs

use ui_input::{KeyboardEvent, PointerEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum ToolInputEvent {
	Pointer(PointerEvent),
	Keyboard(KeyboardEvent),
	Cancel,
	Confirm,
}