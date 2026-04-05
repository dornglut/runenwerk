//! File: domain/editor/editor_tools/src/context.rs

use editor_core::{EditorSession, ToolId};
use editor_viewport::{SnapSettings, ViewportHitResult, ViewportId};

pub struct ToolContext<'a> {
	pub session: &'a EditorSession,
	pub active_tool: ToolId,
	pub viewport_id: ViewportId,
	pub last_hit: Option<&'a ViewportHitResult>,
	pub snap: SnapSettings,
}