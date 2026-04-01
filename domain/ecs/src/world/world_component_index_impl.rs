// Owner: ecs World - Secondary Index APIs
use super::component_indexes::{
    ComponentIndexKey, ComponentSecondaryIndex, DEFAULT_COMPONENT_INDEX_NAME,
};
use super::world_struct::World;
use crate::component::Component;
use crate::entity::Entity;
use std::any::TypeId;

impl World {
    pub fn ensure_component_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        extractor: fn(&T) -> K,
    ) -> bool {
        self.__register_component::<T>();
        let key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        let mut indexes = self.component_indexes.borrow_mut();
        if indexes.contains_key(&key) {
            return false;
        }
        indexes.insert(
            key,
            Box::new(ComponentSecondaryIndex::<T, K>::new(extractor)),
        );
        drop(indexes);
        self.mark_component_indexes_dirty(TypeId::of::<T>());
        true
    }

    pub fn find_entity_by_index<T: Component, K: Ord + Clone + 'static>(
        &self,
        key: &K,
    ) -> Option<Entity> {
        self.find_entity_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    pub fn find_entity_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<Entity> {
        let index_key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        let mut indexes = self.component_indexes.borrow_mut();
        let Some(index) = indexes.get_mut(&index_key) else {
            return None;
        };
        index.rebuild(self);
        index
            .as_any()
            .downcast_ref::<ComponentSecondaryIndex<T, K>>()
            .and_then(|index| index.first_entity_for(key))
    }

    pub fn find_entities_by_index<T: Component, K: Ord + Clone + 'static>(
        &self,
        key: &K,
    ) -> Vec<Entity> {
        self.find_entities_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    pub fn find_entities_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &self,
        name: impl Into<String>,
        key: &K,
    ) -> Vec<Entity> {
        let index_key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        let mut indexes = self.component_indexes.borrow_mut();
        let Some(index) = indexes.get_mut(&index_key) else {
            return Vec::new();
        };
        index.rebuild(self);
        index
            .as_any()
            .downcast_ref::<ComponentSecondaryIndex<T, K>>()
            .map(|index| index.entities_for(key))
            .unwrap_or_default()
    }

    pub fn find_component_by_index<T: Component, K: Ord + Clone + 'static>(
        &self,
        key: &K,
    ) -> Option<&T> {
        self.find_component_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    pub fn find_component_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<&T> {
        let entity = self.find_entity_by_index_named::<T, K>(name, key)?;
        self.get::<T>(entity)
    }
}
