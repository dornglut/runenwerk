use std::collections::BTreeMap;

use editor_core::EntityId;
use editor_scene::SceneEntitySnapshot;

use crate::editor_runtime::{
    HierarchySnapshot, SceneEntityView, all_entity_views, build_hierarchy_snapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SceneDocumentEntityRecord {
    display_name: String,
    parent: Option<EntityId>,
}

#[derive(Debug, Default, Clone)]
pub struct SceneDocumentState {
    entities: BTreeMap<EntityId, SceneDocumentEntityRecord>,
}

impl SceneDocumentState {
    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: new
    pub fn new() -> Self {
        Self::default()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: contains
    pub fn contains(&self, entity: EntityId) -> bool {
        self.entities.contains_key(&entity)
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: entity_ids
    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.keys().copied()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: entity_snapshot
    pub fn entity_snapshot(&self, entity: EntityId) -> Option<SceneEntitySnapshot> {
        let record = self.entities.get(&entity)?;
        Some(SceneEntitySnapshot::new(
            entity,
            record.display_name.clone(),
            record.parent,
        ))
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: entity_display_name
    pub fn entity_display_name(&self, entity: EntityId) -> Option<&str> {
        self.entities
            .get(&entity)
            .map(|record| record.display_name.as_str())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: parent_of
    pub fn parent_of(&self, entity: EntityId) -> Option<Option<EntityId>> {
        self.entities.get(&entity).map(|record| record.parent)
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: children_of
    pub fn children_of(&self, parent: Option<EntityId>) -> Vec<EntityId> {
        self.entities
            .iter()
            .filter_map(|(entity_id, record)| (record.parent == parent).then_some(*entity_id))
            .collect()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: has_children
    pub fn has_children(&self, entity: EntityId) -> bool {
        self.entities
            .values()
            .any(|record| record.parent == Some(entity))
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: register_entity
    pub fn register_entity(
        &mut self,
        entity: EntityId,
        display_name: impl Into<String>,
        parent: Option<EntityId>,
    ) -> Result<(), &'static str> {
        if let Some(parent) = parent {
            if !self.contains(parent) {
                return Err("new parent entity is not registered");
            }
        }

        self.entities.insert(
            entity,
            SceneDocumentEntityRecord {
                display_name: display_name.into(),
                parent,
            },
        );
        Ok(())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: restore_entity
    pub fn restore_entity(&mut self, snapshot: SceneEntitySnapshot) -> Result<(), &'static str> {
        self.register_entity(snapshot.id, snapshot.display_name, snapshot.parent)
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: unregister_entity
    pub fn unregister_entity(&mut self, entity: EntityId) -> Option<SceneEntitySnapshot> {
        self.entities
            .remove(&entity)
            .map(|record| SceneEntitySnapshot::new(entity, record.display_name, record.parent))
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: rename_entity
    pub fn rename_entity(
        &mut self,
        entity: EntityId,
        new_display_name: impl Into<String>,
    ) -> Result<String, &'static str> {
        let record = self
            .entities
            .get_mut(&entity)
            .ok_or("editor entity is not registered")?;

        let previous = std::mem::replace(&mut record.display_name, new_display_name.into());
        Ok(previous)
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: would_create_cycle
    pub fn would_create_cycle(&self, entity: EntityId, candidate_parent: EntityId) -> bool {
        let mut current = Some(candidate_parent);

        while let Some(current_entity) = current {
            if current_entity == entity {
                return true;
            }

            current = self.parent_of(current_entity).flatten();
        }

        false
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: validate_reparent
    pub fn validate_reparent(
        &self,
        entity: EntityId,
        new_parent: Option<EntityId>,
    ) -> Result<(), &'static str> {
        if !self.contains(entity) {
            return Err("editor entity is not registered");
        }

        let Some(parent) = new_parent else {
            return Ok(());
        };

        if !self.contains(parent) {
            return Err("new parent entity is not registered");
        }

        if parent == entity {
            return Err("entity cannot be parented to itself");
        }

        if self.would_create_cycle(entity, parent) {
            return Err("reparent would create a hierarchy cycle");
        }

        Ok(())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: reparent_entity
    pub fn reparent_entity(
        &mut self,
        entity: EntityId,
        new_parent: Option<EntityId>,
    ) -> Result<Option<EntityId>, &'static str> {
        self.validate_reparent(entity, new_parent)?;
        let record = self
            .entities
            .get_mut(&entity)
            .ok_or("editor entity is not registered")?;

        let previous_parent = record.parent;
        record.parent = new_parent;
        Ok(previous_parent)
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: all_entity_views
    pub fn all_entity_views(&self) -> Vec<SceneEntityView> {
        all_entity_views(self)
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/document/mod.rs
    /// Method: hierarchy_snapshot
    pub fn hierarchy_snapshot(&self) -> HierarchySnapshot {
        build_hierarchy_snapshot(self)
    }
}
