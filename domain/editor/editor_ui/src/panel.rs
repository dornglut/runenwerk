//! File: domain/editor/editor_ui/src/panel.rs
//! Purpose: Panel identity, descriptors, and panel registry contracts.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelKind {
	SceneHierarchy,
	Inspector,
	ContentBrowser,
	Viewport,
	Console,
	Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
	pub id: PanelId,
	pub kind: PanelKind,
	pub stable_name: &'static str,
	pub display_name: String,
	pub is_closable: bool,
}

impl PanelDescriptor {
	pub fn new(
		id: PanelId,
		kind: PanelKind,
		stable_name: &'static str,
		display_name: impl Into<String>,
	) -> Self {
		Self {
			id,
			kind,
			stable_name,
			display_name: display_name.into(),
			is_closable: true,
		}
	}

	pub fn with_closable(mut self, is_closable: bool) -> Self {
		self.is_closable = is_closable;
		self
	}
}

#[derive(Debug, Default)]
pub struct PanelRegistry {
	panels: BTreeMap<PanelId, PanelDescriptor>,
}

impl PanelRegistry {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn register(&mut self, descriptor: PanelDescriptor) -> Option<PanelDescriptor> {
		self.panels.insert(descriptor.id, descriptor)
	}

	pub fn get(&self, id: PanelId) -> Option<&PanelDescriptor> {
		self.panels.get(&id)
	}

	pub fn iter(&self) -> impl Iterator<Item = &PanelDescriptor> {
		self.panels.values()
	}
}