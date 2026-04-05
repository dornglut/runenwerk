//! File: domain/editor/editor_tools/src/behavior.rs
//! Purpose: Tool lifecycle and input handling contracts.

use crate::{ToolContext, ToolInputEvent, ToolResult};

pub trait ToolBehavior: Send + Sync {
	fn on_activate(&mut self, _ctx: &ToolContext<'_>) -> ToolResult {
		ToolResult::none()
	}

	fn on_deactivate(&mut self, _ctx: &ToolContext<'_>) -> ToolResult {
		ToolResult::none()
	}

	fn handle_input(
		&mut self,
		ctx: &ToolContext<'_>,
		event: &ToolInputEvent,
	) -> ToolResult;

	fn update(&mut self, _ctx: &ToolContext<'_>) -> ToolResult {
		ToolResult::none()
	}
}