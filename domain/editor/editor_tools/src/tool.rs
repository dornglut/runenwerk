//! File: domain/editor/editor_tools/src/tool.rs

use editor_core::ToolId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorToolDescriptor {
	pub id: ToolId,
	pub stable_name: &'static str,
	pub display_name: String,
}

impl EditorToolDescriptor {
	pub fn new(id: ToolId, stable_name: &'static str, display_name: impl Into<String>) -> Self {
		Self {
			id,
			stable_name,
			display_name: display_name.into(),
		}
	}
}