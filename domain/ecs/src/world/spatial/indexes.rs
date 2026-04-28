// Owner: ecs World Spatial - Spatial Index APIs
use crate::entity::Entity;
use crate::errors::SpatialIndexError;
use crate::indexing::{DEFAULT_SPATIAL_INDEX_NAME, SpatialHashConfig, SpatialHashIndex};
use crate::world::World;
use geometry::Aabb3;

fn normalized_index_name(name: impl Into<String>) -> String {
    let mut name = name.into();
    name = name.trim().to_string();
    if name.is_empty() {
        name = DEFAULT_SPATIAL_INDEX_NAME.to_string();
    }
    name
}

impl World {
    pub fn ensure_spatial_hash_index(
        &mut self,
        config: SpatialHashConfig,
    ) -> Result<bool, SpatialIndexError> {
        self.ensure_spatial_hash_index_named(DEFAULT_SPATIAL_INDEX_NAME, config)
    }

    pub fn ensure_spatial_hash_index_named(
        &mut self,
        name: impl Into<String>,
        config: SpatialHashConfig,
    ) -> Result<bool, SpatialIndexError> {
        let name = normalized_index_name(name);
        if self.spatial_indexes.contains_key(&name) {
            return Ok(false);
        }
        let index = SpatialHashIndex::new(config)?;
        self.spatial_indexes.insert(name, Box::new(index));
        Ok(true)
    }

    pub fn spatial_insert(
        &mut self,
        entity: Entity,
        bounds: Aabb3,
    ) -> Result<(), SpatialIndexError> {
        self.spatial_insert_named(DEFAULT_SPATIAL_INDEX_NAME, entity, bounds)
    }

    pub fn spatial_insert_named(
        &mut self,
        name: impl Into<String>,
        entity: Entity,
        bounds: Aabb3,
    ) -> Result<(), SpatialIndexError> {
        self.ensure_entity_exists(entity)?;
        let name = normalized_index_name(name);
        let index = self
            .spatial_indexes
            .get_mut(&name)
            .ok_or(SpatialIndexError::MissingIndex { name })?;
        index.insert(entity, bounds)
    }

    pub fn spatial_update(
        &mut self,
        entity: Entity,
        bounds: Aabb3,
    ) -> Result<(), SpatialIndexError> {
        self.spatial_update_named(DEFAULT_SPATIAL_INDEX_NAME, entity, bounds)
    }

    pub fn spatial_update_named(
        &mut self,
        name: impl Into<String>,
        entity: Entity,
        bounds: Aabb3,
    ) -> Result<(), SpatialIndexError> {
        self.ensure_entity_exists(entity)?;
        let name = normalized_index_name(name);
        let index = self
            .spatial_indexes
            .get_mut(&name)
            .ok_or(SpatialIndexError::MissingIndex { name })?;
        index.update(entity, bounds)
    }

    pub fn spatial_remove(&mut self, entity: Entity) -> Result<bool, SpatialIndexError> {
        self.spatial_remove_named(DEFAULT_SPATIAL_INDEX_NAME, entity)
    }

    pub fn spatial_remove_named(
        &mut self,
        name: impl Into<String>,
        entity: Entity,
    ) -> Result<bool, SpatialIndexError> {
        self.ensure_entity_exists(entity)?;
        let name = normalized_index_name(name);
        let index = self
            .spatial_indexes
            .get_mut(&name)
            .ok_or(SpatialIndexError::MissingIndex { name })?;
        Ok(index.remove(entity))
    }

    pub fn spatial_query_aabb(&self, bounds: Aabb3) -> Result<Vec<Entity>, SpatialIndexError> {
        self.spatial_query_aabb_named(DEFAULT_SPATIAL_INDEX_NAME, bounds)
    }

    pub fn spatial_query_aabb_named(
        &self,
        name: impl Into<String>,
        bounds: Aabb3,
    ) -> Result<Vec<Entity>, SpatialIndexError> {
        let name = normalized_index_name(name);
        let index = self
            .spatial_indexes
            .get(&name)
            .ok_or(SpatialIndexError::MissingIndex { name })?;
        index.query_aabb(bounds)
    }

    pub(crate) fn remove_entity_from_spatial_indexes(&mut self, entity: Entity) {
        for index in self.spatial_indexes.values_mut() {
            index.remove(entity);
        }
    }
}
