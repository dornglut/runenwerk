//! File: domain/editor/editor_tools/src/dispatcher.rs
//! Purpose: Apply tool behavior to inputs and collect tool results.

use crate::{ToolBehavior, ToolContext, ToolInputEvent, ToolResult};

pub struct ToolDispatcher;

impl ToolDispatcher {
	pub fn activate(
		tool: &mut dyn ToolBehavior,
		ctx: &ToolContext<'_>,
	) -> ToolResult {
		tool.on_activate(ctx)
	}

	pub fn deactivate(
		tool: &mut dyn ToolBehavior,
		ctx: &ToolContext<'_>,
	) -> ToolResult {
		tool.on_deactivate(ctx)
	}

	pub fn dispatch_input(
		tool: &mut dyn ToolBehavior,
		ctx: &ToolContext<'_>,
		event: &ToolInputEvent,
	) -> ToolResult {
		tool.handle_input(ctx, event)
	}

	pub fn update(
		tool: &mut dyn ToolBehavior,
		ctx: &ToolContext<'_>,
	) -> ToolResult {
		tool.update(ctx)
	}
}