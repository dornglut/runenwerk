use editor_core::EntityId;

use crate::editor_runtime::{HierarchyItem, HierarchySnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerItem {
    pub entity: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
    pub depth: usize,
    pub children: Vec<OutlinerItem>,
}

impl OutlinerItem {
    /// File: apps/runenwerk_editor/src/editor_runtime/outliner/model.rs
    /// Method: new
    pub fn new(
        entity: EntityId,
        display_name: impl Into<String>,
        parent: Option<EntityId>,
        depth: usize,
        children: Vec<OutlinerItem>,
    ) -> Self {
        Self {
            entity,
            display_name: display_name.into(),
            parent,
            depth,
            children,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutlinerTree {
    pub roots: Vec<OutlinerItem>,
}

impl OutlinerTree {
    /// File: apps/runenwerk_editor/src/editor_runtime/outliner/model.rs
    /// Method: flatten
    pub fn flatten(&self) -> Vec<OutlinerRow> {
        let mut rows = Vec::new();

        for root in &self.roots {
            flatten_item(root, &mut rows);
        }

        rows
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerRow {
    pub entity: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
    pub depth: usize,
    pub has_children: bool,
}

impl OutlinerRow {
    /// File: apps/runenwerk_editor/src/editor_runtime/outliner/model.rs
    /// Method: new
    pub fn new(
        entity: EntityId,
        display_name: impl Into<String>,
        parent: Option<EntityId>,
        depth: usize,
        has_children: bool,
    ) -> Self {
        Self {
            entity,
            display_name: display_name.into(),
            parent,
            depth,
            has_children,
        }
    }
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/model.rs
/// Method: outliner_tree_from_hierarchy_snapshot
pub fn outliner_tree_from_hierarchy_snapshot(snapshot: &HierarchySnapshot) -> OutlinerTree {
    OutlinerTree {
        roots: snapshot
            .roots
            .iter()
            .map(|item| map_hierarchy_item(item, 0))
            .collect(),
    }
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/model.rs
/// Method: map_hierarchy_item
fn map_hierarchy_item(item: &HierarchyItem, depth: usize) -> OutlinerItem {
    OutlinerItem::new(
        item.entity,
        item.display_name.clone(),
        item.parent,
        depth,
        item.children
            .iter()
            .map(|child| map_hierarchy_item(child, depth + 1))
            .collect(),
    )
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/model.rs
/// Method: flatten_item
fn flatten_item(item: &OutlinerItem, rows: &mut Vec<OutlinerRow>) {
    rows.push(OutlinerRow::new(
        item.entity,
        item.display_name.clone(),
        item.parent,
        item.depth,
        !item.children.is_empty(),
    ));

    for child in &item.children {
        flatten_item(child, rows);
    }
}
