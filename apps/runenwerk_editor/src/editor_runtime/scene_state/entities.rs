use editor_core::EntityId;

use crate::editor_runtime::SceneDocumentState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneEntityView {
    pub id: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
}

impl SceneEntityView {
    pub fn new(id: EntityId, display_name: impl Into<String>, parent: Option<EntityId>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            parent,
        }
    }
}

pub fn entity_view(document: &SceneDocumentState, entity: EntityId) -> Option<SceneEntityView> {
    let snapshot = document.entity_snapshot(entity)?;
    Some(SceneEntityView::new(
        snapshot.id,
        snapshot.display_name,
        snapshot.parent,
    ))
}

pub fn all_entity_views(document: &SceneDocumentState) -> Vec<SceneEntityView> {
    let mut entities = document
        .entity_ids()
        .filter_map(|entity| entity_view(document, entity))
        .collect::<Vec<_>>();

    entities.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.id.cmp(&right.id))
    });

    entities
}
