use super::spatial_index::{SpatialIndex, SpatialIndexStorage};
use crate::Entity;
use crate::errors::SpatialIndexError;
use geometry::Aabb3;
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Copy, Clone)]
pub struct SpatialHashConfig {
    pub cell_size: f32,
}

impl Default for SpatialHashConfig {
    fn default() -> Self {
        Self { cell_size: 1.0 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct CellCoord {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone, Copy)]
struct EntitySpatialEntry {
    bounds: Aabb3,
    min_cell: CellCoord,
    max_cell: CellCoord,
}

pub struct SpatialHashIndex {
    cell_size: f32,
    cells: HashMap<CellCoord, BTreeSet<Entity>>,
    entities: HashMap<Entity, EntitySpatialEntry>,
}

impl SpatialHashIndex {
    pub fn new(config: SpatialHashConfig) -> Result<Self, SpatialIndexError> {
        if !config.cell_size.is_finite() || config.cell_size <= 0.0 {
            return Err(SpatialIndexError::InvalidCellSize {
                cell_size: config.cell_size,
            });
        }

        Ok(Self {
            cell_size: config.cell_size,
            cells: HashMap::new(),
            entities: HashMap::new(),
        })
    }

    fn ensure_valid_bounds(bounds: Aabb3) -> Result<(), SpatialIndexError> {
        if !bounds.is_valid() {
            return Err(SpatialIndexError::InvalidBounds);
        }
        Ok(())
    }

    fn axis_to_cell(&self, value: f32) -> i32 {
        (value / self.cell_size).floor() as i32
    }

    fn cell_range_for_bounds(&self, bounds: Aabb3) -> (CellCoord, CellCoord) {
        (
            CellCoord {
                x: self.axis_to_cell(bounds.min.x),
                y: self.axis_to_cell(bounds.min.y),
                z: self.axis_to_cell(bounds.min.z),
            },
            CellCoord {
                x: self.axis_to_cell(bounds.max.x),
                y: self.axis_to_cell(bounds.max.y),
                z: self.axis_to_cell(bounds.max.z),
            },
        )
    }

    fn remove_entity_from_cells(
        &mut self,
        entity: Entity,
        min_cell: CellCoord,
        max_cell: CellCoord,
    ) {
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let coord = CellCoord { x, y, z };
                    let mut should_remove_cell = false;
                    if let Some(occupants) = self.cells.get_mut(&coord) {
                        occupants.remove(&entity);
                        should_remove_cell = occupants.is_empty();
                    }
                    if should_remove_cell {
                        self.cells.remove(&coord);
                    }
                }
            }
        }
    }

    fn insert_entity_into_cells(
        &mut self,
        entity: Entity,
        min_cell: CellCoord,
        max_cell: CellCoord,
    ) {
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    self.cells
                        .entry(CellCoord { x, y, z })
                        .or_default()
                        .insert(entity);
                }
            }
        }
    }

    fn upsert_entity(&mut self, entity: Entity, bounds: Aabb3) {
        let (min_cell, max_cell) = self.cell_range_for_bounds(bounds);
        if let Some(previous) = self.entities.remove(&entity) {
            self.remove_entity_from_cells(entity, previous.min_cell, previous.max_cell);
        }
        self.insert_entity_into_cells(entity, min_cell, max_cell);
        self.entities.insert(
            entity,
            EntitySpatialEntry {
                bounds,
                min_cell,
                max_cell,
            },
        );
    }
}

impl SpatialIndex for SpatialHashIndex {
    fn insert(&mut self, entity: Entity, bounds: Aabb3) -> Result<(), SpatialIndexError> {
        Self::ensure_valid_bounds(bounds)?;
        self.upsert_entity(entity, bounds);
        Ok(())
    }

    fn update(&mut self, entity: Entity, bounds: Aabb3) -> Result<(), SpatialIndexError> {
        Self::ensure_valid_bounds(bounds)?;
        self.upsert_entity(entity, bounds);
        Ok(())
    }

    fn remove(&mut self, entity: Entity) -> bool {
        let Some(previous) = self.entities.remove(&entity) else {
            return false;
        };
        self.remove_entity_from_cells(entity, previous.min_cell, previous.max_cell);
        true
    }

    fn query_aabb(&self, bounds: Aabb3) -> Result<Vec<Entity>, SpatialIndexError> {
        Self::ensure_valid_bounds(bounds)?;
        let (min_cell, max_cell) = self.cell_range_for_bounds(bounds);
        let mut candidates = BTreeSet::new();
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    if let Some(occupants) = self.cells.get(&CellCoord { x, y, z }) {
                        candidates.extend(occupants.iter().copied());
                    }
                }
            }
        }

        let mut results = Vec::new();
        for entity in candidates {
            let Some(entry) = self.entities.get(&entity) else {
                continue;
            };
            if entry.bounds.intersects(&bounds) {
                results.push(entity);
            }
        }
        Ok(results)
    }
}

impl SpatialIndexStorage for SpatialHashIndex {}
