//! File: domain/editor/editor_ui/src/tab.rs
//! Purpose: Tab identity and tab-to-panel association.

use crate::PanelId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabDescriptor {
	pub id: TabId,
	pub panel_id: PanelId,
	pub title: String,
	pub is_dirty: bool,
}

impl TabDescriptor {
	pub fn new(id: TabId, panel_id: PanelId, title: impl Into<String>) -> Self {
		Self {
			id,
			panel_id,
			title: title.into(),
			is_dirty: false,
		}
	}

	pub fn with_dirty(mut self, is_dirty: bool) -> Self {
		self.is_dirty = is_dirty;
		self
	}
}