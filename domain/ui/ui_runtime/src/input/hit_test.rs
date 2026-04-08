//! File: domain/ui/ui_runtime/src/input/hit_test.rs
//! Purpose: Hit testing against computed layout records.

use ui_math::UiPoint;

use crate::{ComputedLayoutMap, UiTree, WidgetId};

pub fn hit_test_widget(
	tree: &UiTree,
	layouts: &ComputedLayoutMap,
	point: UiPoint,
) -> Option<WidgetId> {
	hit_test_node(&tree.root, layouts, point)
}

fn hit_test_node(
	node: &crate::UiNode,
	layouts: &ComputedLayoutMap,
	point: UiPoint,
) -> Option<WidgetId> {
	let layout = layouts.get(&node.id)?;

	if !layout.bounds.contains(point) {
		return None;
	}

	for child in node.children.iter().rev() {
		if let Some(hit) = hit_test_node(child, layouts, point) {
			return Some(hit);
		}
	}

	Some(node.id)
}