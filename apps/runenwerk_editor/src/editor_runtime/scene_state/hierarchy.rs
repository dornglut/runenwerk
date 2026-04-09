use editor_core::EntityId;

use crate::editor_runtime::SceneDocumentState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HierarchyItem {
    pub entity: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
    pub children: Vec<HierarchyItem>,
}

impl HierarchyItem {
    pub fn new(
        entity: EntityId,
        display_name: impl Into<String>,
        parent: Option<EntityId>,
        children: Vec<HierarchyItem>,
    ) -> Self {
        Self {
            entity,
            display_name: display_name.into(),
            parent,
            children,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HierarchySnapshot {
    pub roots: Vec<HierarchyItem>,
}

pub fn root_entities(document: &SceneDocumentState) -> Vec<EntityId> {
    let mut roots = document.children_of(None);
    roots.sort();
    roots
}

pub fn children_of(document: &SceneDocumentState, parent: EntityId) -> Vec<EntityId> {
    let mut children = document.children_of(Some(parent));
    children.sort();
    children
}

pub fn build_hierarchy_snapshot(document: &SceneDocumentState) -> HierarchySnapshot {
    let roots = root_entities(document)
        .into_iter()
        .filter_map(|entity| build_item(document, entity))
        .collect::<Vec<_>>();

    HierarchySnapshot { roots }
}

pub fn validate_reparent(
    document: &SceneDocumentState,
    entity: EntityId,
    new_parent: Option<EntityId>,
) -> Result<(), &'static str> {
    if !document.contains(entity) {
        return Err("editor entity is not registered");
    }

    let Some(parent) = new_parent else {
        return Ok(());
    };

    if !document.contains(parent) {
        return Err("new parent entity is not registered");
    }

    if parent == entity {
        return Err("entity cannot be parented to itself");
    }

    if document.would_create_cycle(entity, parent) {
        return Err("reparent would create a hierarchy cycle");
    }

    Ok(())
}

fn build_item(document: &SceneDocumentState, entity: EntityId) -> Option<HierarchyItem> {
    let snapshot = document.entity_snapshot(entity)?;
    let children = children_of(document, entity)
        .into_iter()
        .filter_map(|child| build_item(document, child))
        .collect::<Vec<_>>();

    Some(HierarchyItem::new(
        snapshot.id,
        snapshot.display_name,
        snapshot.parent,
        children,
    ))
}
