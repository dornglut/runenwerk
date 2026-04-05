//! File: domain/editor/editor_core/src/selection.rs
//! Purpose: Editor selection model that can represent scene, resource,
//! component, and asset targets without coupling editor_core to ECS internals.

use std::collections::BTreeSet;

use crate::DocumentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentTypeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceTypeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SelectionTarget {
	Document(DocumentId),
	Entity(EntityId),
	Component {
		entity: EntityId,
		component_type: ComponentTypeId,
	},
	Resource(ResourceTypeId),
	Asset(AssetId),
	Custom {
		domain: &'static str,
		id: u64,
	},
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionSet {
	items: BTreeSet<SelectionTarget>,
	primary: Option<SelectionTarget>,
}

impl SelectionSet {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	pub fn len(&self) -> usize {
		self.items.len()
	}

	pub fn primary(&self) -> Option<&SelectionTarget> {
		self.primary.as_ref()
	}

	pub fn contains(&self, target: &SelectionTarget) -> bool {
		self.items.contains(target)
	}

	pub fn iter(&self) -> impl Iterator<Item = &SelectionTarget> {
		self.items.iter()
	}

	pub fn clear(&mut self) {
		self.items.clear();
		self.primary = None;
	}

	pub fn set_single(&mut self, target: SelectionTarget) {
		self.items.clear();
		self.items.insert(target.clone());
		self.primary = Some(target);
	}

	pub fn add(&mut self, target: SelectionTarget) {
		let was_empty = self.items.is_empty();
		self.items.insert(target.clone());

		if was_empty || self.primary.is_none() {
			self.primary = Some(target);
		}
	}

	pub fn remove(&mut self, target: &SelectionTarget) -> bool {
		let removed = self.items.remove(target);

		if removed && self.primary.as_ref() == Some(target) {
			self.primary = self.items.iter().next().cloned();
		}

		removed
	}

	pub fn set_primary(&mut self, target: &SelectionTarget) -> Result<(), &'static str> {
		if !self.items.contains(target) {
			return Err("cannot set primary selection to a target not in the selection set");
		}

		self.primary = Some(target.clone());
		Ok(())
	}
}