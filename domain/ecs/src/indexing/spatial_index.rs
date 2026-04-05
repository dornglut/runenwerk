use crate::Entity;
use crate::errors::SpatialIndexError;
use geometry::Aabb3;

pub const DEFAULT_SPATIAL_INDEX_NAME: &str = "__default";

pub trait SpatialIndex {
    fn insert(&mut self, entity: Entity, bounds: Aabb3) -> Result<(), SpatialIndexError>;
    fn update(&mut self, entity: Entity, bounds: Aabb3) -> Result<(), SpatialIndexError>;
    fn remove(&mut self, entity: Entity) -> bool;
    fn query_aabb(&self, bounds: Aabb3) -> Result<Vec<Entity>, SpatialIndexError>;
}

pub trait SpatialIndexStorage: SpatialIndex {}
