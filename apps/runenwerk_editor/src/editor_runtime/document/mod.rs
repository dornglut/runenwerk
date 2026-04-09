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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, entity: EntityId) -> bool {
        self.entities.contains_key(&entity)
    }

    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.keys().copied()
    }

    pub fn entity_snapshot(&self, entity: EntityId) -> Option<SceneEntitySnapshot> {
        let record = self.entities.get(&entity)?;
        Some(SceneEntitySnapshot::new(
            entity,
            record.display_name.clone(),
            record.parent,
        ))
    }

    pub fn entity_display_name(&self, entity: EntityId) -> Option<&str> {
        self.entities
            .get(&entity)
            .map(|record| record.display_name.as_str())
    }

    pub fn parent_of(&self, entity: EntityId) -> Option<Option<EntityId>> {
        self.entities.get(&entity).map(|record| record.parent)
    }

    pub fn children_of(&self, parent: Option<EntityId>) -> Vec<EntityId> {
        self.entities
            .iter()
            .filter_map(|(entity_id, record)| (record.parent == parent).then_some(*entity_id))
            .collect()
    }

    pub fn has_children(&self, entity: EntityId) -> bool {
        self.entities
            .values()
            .any(|record| record.parent == Some(entity))
    }

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

    pub fn restore_entity(&mut self, snapshot: SceneEntitySnapshot) -> Result<(), &'static str> {
        self.register_entity(snapshot.id, snapshot.display_name, snapshot.parent)
    }

    pub fn unregister_entity(&mut self, entity: EntityId) -> Option<SceneEntitySnapshot> {
        self.entities
            .remove(&entity)
            .map(|record| SceneEntitySnapshot::new(entity, record.display_name, record.parent))
    }

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

    pub fn all_entity_views(&self) -> Vec<SceneEntityView> {
        all_entity_views(self)
    }

    pub fn hierarchy_snapshot(&self) -> HierarchySnapshot {
        build_hierarchy_snapshot(self)
    }
}
