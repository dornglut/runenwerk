//! File: domain/editor/editor_ui/src/dock.rs
//! Purpose: Dock tree identity and split/tab dock model.

use crate::TabId;
use ui_math::Axis;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DockNodeId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum DockNode {
	Split {
		id: DockNodeId,
		axis: Axis,
		ratio: f32,
		first: DockNodeId,
		second: DockNodeId,
	},
	Tabs {
		id: DockNodeId,
		tabs: Vec<TabId>,
		active_tab: Option<TabId>,
	},
}

impl DockNode {
	pub fn id(&self) -> DockNodeId {
		match self {
			DockNode::Split { id, .. } | DockNode::Tabs { id, .. } => *id,
		}
	}
}