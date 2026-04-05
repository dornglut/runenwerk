//! File: domain/ui/ui_input/src/event.rs
//! Purpose: High-level UI input events.

use crate::{Key, KeyState, Modifiers, PointerButton, PointerDelta, PointerEventKind, PointerPosition};

#[derive(Debug, Clone, PartialEq)]
pub struct PointerEvent {
	pub kind: PointerEventKind,
	pub position: PointerPosition,
	pub delta: PointerDelta,
	pub button: Option<PointerButton>,
	pub modifiers: Modifiers,
	pub click_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardEvent {
	pub key: Key,
	pub state: KeyState,
	pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputEvent {
	pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiInputEvent {
	Pointer(PointerEvent),
	Keyboard(KeyboardEvent),
	Text(TextInputEvent),
}