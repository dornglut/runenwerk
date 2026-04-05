//! File: domain/editor/editor_tools/src/result.rs

use crate::ToolIntent;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
	pub intents: Vec<ToolIntent>,
	pub keep_active: bool,
}

impl ToolResult {
	pub fn none() -> Self {
		Self {
			intents: Vec::new(),
			keep_active: true,
		}
	}

	pub fn with_intent(mut self, intent: ToolIntent) -> Self {
		self.intents.push(intent);
		self
	}
}