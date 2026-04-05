//! File: domain/editor/editor_ui/src/workspace.rs
//! Purpose: Workspace identity and top-level workspace layout model.

use crate::{DockNodeId, PanelId, TabId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceLayout {
	pub id: WorkspaceId,
	pub display_name: String,
	pub root_dock: Option<DockNodeId>,
	pub open_panels: Vec<PanelId>,
	pub open_tabs: Vec<TabId>,
}

impl WorkspaceLayout {
	pub fn new(id: WorkspaceId, display_name: impl Into<String>) -> Self {
		Self {
			id,
			display_name: display_name.into(),
			root_dock: None,
			open_panels: Vec::new(),
			open_tabs: Vec::new(),
		}
	}
}