//! File: domain/ui/ui_core/src/tree.rs
//! Purpose: Retained widget tree storage contracts.

use std::collections::HashMap;

use ui_math::UiRect;

use crate::{Invalidation, WidgetId, WidgetTypeId};

#[derive(Debug, Clone)]
pub struct WidgetNode {
	pub id: WidgetId,
	pub type_id: WidgetTypeId,
	pub parent: Option<WidgetId>,
	pub children: Vec<WidgetId>,
	pub rect: UiRect,
	pub invalidation: Invalidation,
	pub visible: bool,
	pub enabled: bool,
}

impl WidgetNode {
	pub fn new(id: WidgetId, type_id: WidgetTypeId) -> Self {
		Self {
			id,
			type_id,
			parent: None,
			children: Vec::new(),
			rect: UiRect::ZERO,
			invalidation: Invalidation::relayout(),
			visible: true,
			enabled: true,
		}
	}
}

#[derive(Debug, Default)]
pub struct WidgetTree {
	root: Option<WidgetId>,
	nodes: HashMap<WidgetId, WidgetNode>,
}

impl WidgetTree {
	pub fn root(&self) -> Option<WidgetId> {
		self.root
	}

	pub fn set_root(&mut self, id: WidgetId) {
		self.root = Some(id);
	}

	pub fn is_root(&self, id: WidgetId) -> bool {
		self.root == Some(id)
	}

	pub fn insert(&mut self, node: WidgetNode) -> Option<WidgetNode> {
		self.nodes.insert(node.id, node)
	}

	pub fn contains(&self, id: WidgetId) -> bool {
		self.nodes.contains_key(&id)
	}

	pub fn node(&self, id: WidgetId) -> Option<&WidgetNode> {
		self.nodes.get(&id)
	}

	pub fn node_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode> {
		self.nodes.get_mut(&id)
	}

	pub fn parent(&self, id: WidgetId) -> Option<WidgetId> {
		self.nodes.get(&id).and_then(|node| node.parent)
	}

	pub fn children(&self, id: WidgetId) -> &[WidgetId] {
		self.nodes
			.get(&id)
			.map(|node| node.children.as_slice())
			.unwrap_or(&[])
	}

	pub fn iter_children(&self, id: WidgetId) -> impl Iterator<Item = WidgetId> + '_ {
		self.children(id).iter().copied()
	}

	pub fn add_child(&mut self, parent: WidgetId, child: WidgetId) -> Result<(), &'static str> {
		if parent == child {
			return Err("widget cannot be parent of itself");
		}

		if !self.nodes.contains_key(&parent) {
			return Err("parent widget does not exist");
		}

		if !self.nodes.contains_key(&child) {
			return Err("child widget does not exist");
		}

		let old_parent = self.nodes.get(&child).and_then(|n| n.parent);

		if let Some(old_parent) = old_parent {
			if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
				old_parent_node.children.retain(|id| *id != child);
			}
		}

		if let Some(child_node) = self.nodes.get_mut(&child) {
			child_node.parent = Some(parent);
		}

		if let Some(parent_node) = self.nodes.get_mut(&parent) {
			if !parent_node.children.contains(&child) {
				parent_node.children.push(child);
			}
		}

		Ok(())
	}

	pub fn remove(&mut self, id: WidgetId) -> Option<WidgetNode> {
		let node = self.nodes.remove(&id)?;

		if self.root == Some(id) {
			self.root = None;
		}

		if let Some(parent) = node.parent {
			if let Some(parent_node) = self.nodes.get_mut(&parent) {
				parent_node.children.retain(|child_id| *child_id != id);
			}
		}

		for child_id in &node.children {
			if let Some(child_node) = self.nodes.get_mut(child_id) {
				child_node.parent = None;
			}
		}

		Some(node)
	}

	pub fn iter(&self) -> impl Iterator<Item = (&WidgetId, &WidgetNode)> {
		self.nodes.iter()
	}

	pub fn preorder_ids(&self) -> Vec<WidgetId> {
		let mut out = Vec::new();
		if let Some(root) = self.root {
			self.preorder_from(root, &mut out);
		}
		out
	}

	pub fn postorder_ids(&self) -> Vec<WidgetId> {
		let mut out = Vec::new();
		if let Some(root) = self.root {
			self.postorder_from(root, &mut out);
		}
		out
	}

	fn preorder_from(&self, id: WidgetId, out: &mut Vec<WidgetId>) {
		out.push(id);

		if let Some(node) = self.nodes.get(&id) {
			for child in &node.children {
				self.preorder_from(*child, out);
			}
		}
	}

	fn postorder_from(&self, id: WidgetId, out: &mut Vec<WidgetId>) {
		if let Some(node) = self.nodes.get(&id) {
			for child in &node.children {
				self.postorder_from(*child, out);
			}
		}

		out.push(id);
	}
}