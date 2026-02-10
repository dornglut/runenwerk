// src/world.rs
use std::any::TypeId;
use std::collections::HashMap;
use crate::{ComponentTable, ComponentTableTrait, Entity, Fk, Pk};

pub struct World {
    tables: HashMap<TypeId, Box<dyn ComponentTableTrait>>,
    next_fk: Fk, // internal FK allocator
}

impl World {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            next_fk: 0,
        }
    }

    /// Spawn a new entity, returning its FK
    pub fn spawn_entity(&mut self) -> Entity {
        let e = Entity { id: self.next_fk };
        self.next_fk += 1;
        e
    }

    /// Add a new table for a component type
    pub fn add_table<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        if self.tables.contains_key(&type_id) {
            panic!("Table for this component type already exists");
        }
        self.tables.insert(type_id, Box::new(ComponentTable::<T>::new()));
    }

    /// Add a component row to a table for a given entity
    pub fn add_component<T: 'static>(&mut self, entity: Entity, data: T) -> Pk {
        // Get or create the table automatically
        let table = self.tables
          .entry(TypeId::of::<T>())
          .or_insert_with(|| Box::new(ComponentTable::<T>::new()))
          .as_any_mut()
          .downcast_mut::<ComponentTable<T>>()
          .expect("Failed to downcast table");

        table.add(entity.id, data)
    }

    /// Get mutable access to a table
    pub fn get_table_mut<T: 'static>(&mut self) -> Option<&mut ComponentTable<T>> {
        let type_id = TypeId::of::<T>();
        self.tables.get_mut(&type_id)
          .and_then(|tbl| tbl.as_any_mut().downcast_mut::<ComponentTable<T>>())
    }

    /// Get immutable access to a table
    pub fn get_table<T: 'static>(&self) -> Option<&ComponentTable<T>> {
        let type_id = TypeId::of::<T>();
        self.tables.get(&type_id)
          .and_then(|tbl| tbl.as_any().downcast_ref::<ComponentTable<T>>())
    }

    /// Returns all component rows of type T
    pub fn query<T: 'static>(&self) -> Vec<&crate::ComponentRow<T>> {
        if let Some(table) = self.get_table::<T>() {
            table.rows.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Returns all component rows of type T for a specific entity
    pub fn query_entity<T: 'static>(&self, entity: Entity) -> Vec<&crate::ComponentRow<T>> {
        if let Some(table) = self.get_table::<T>() {
            table.get_by_fk(entity.id).unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}

/// Spawns a new entity in the world and attaches any number of components
#[macro_export]
macro_rules! spawn_entity {
    ($world:expr, $($comp:expr),+ $(,)?) => {{
        // create the entity
        let e = $world.spawn_entity();

        // add all components
        $(
            $world.add_component(e, $comp);
        )+

        // return the entity
        e
    }};
}