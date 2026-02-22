// src/table/archetype.rs

use crate::{AnyStorage, ComponentKey, ComponentRegistry, Column, EntityHandle, Row, ArchetypeKey};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub enum ArchetypeError {
	MissingComponent(&'static str),
	WrongComponentCount { expected: usize, got: usize },
}

/// An archetype groups entities with the same set of component types.
/// - Rows = EntityHandle
/// - Columns = component storages
pub struct Archetype {
	key: ArchetypeKey,
	entity_column: Column<EntityHandle>,
	columns: Vec<Box<dyn AnyStorage>>,
	column_indices: HashMap<TypeId, usize>,
}

impl Archetype {
	pub fn new(
		component_keys: Vec<ComponentKey>,
		constructors: &HashMap<TypeId, fn() -> Box<dyn AnyStorage>>,
	) -> Self {
		let types_sorted = component_keys
			.iter()
			.map(|c| c.type_id)
			.collect::<Vec<_>>();
		assert_eq!(
			types_sorted.len(),
			component_keys.len(),
			"Duplicate component types in Archetype::new"
		);

		let key = ArchetypeKey::new(component_keys);
		let mut column_indices = HashMap::new();
		let mut columns = Vec::with_capacity(key.components.len());

		for (i, ckey) in key.components.iter().enumerate() {
			let ctor = constructors
				.get(&ckey.type_id)
				.expect(&format!("Component type {:?} not registered", ckey.name));
			columns.push(ctor());

			assert!(
				!column_indices.contains_key(&ckey.type_id),
				"Duplicate column index for type {:?}",
				ckey.name
			);
			column_indices.insert(ckey.type_id, i);
		}

		debug!("Created archetype {}", key);

		Self {
			key,
			entity_column: Column::new(),
			columns,
			column_indices,
		}
	}

	/// Adds a new entity and its components (unordered map of TypeId → Box<dyn Any>)
	pub fn add_row(
		&mut self,
		entity: EntityHandle,
		mut components: HashMap<TypeId, Box<dyn Any>>,
	) {
		assert_eq!(
			components.len(),
			self.columns.len(),
			"Component count mismatch in add_row"
		);

		for component in &self.key.components {
			let ty = component.type_id;
			let value = components
				.remove(&ty)
				.expect("Missing component required by archetype");
			let idx = self
				.column_indices
				.get(&ty)
				.expect("Component type not in archetype");
			self.columns[*idx]
				.push_box(value)
				.expect("Type mismatch in add_row");
		}

		self.entity_column.push(entity);

		debug_assert!(self.validate(), "Columns out of sync after adding row");
	}

	/// Adds a row with a complete set of components in archetype order
	pub fn add_row_complete(
		&mut self,
		entity: EntityHandle,
		components: Vec<Box<dyn Any>>,
	) -> Result<(), ArchetypeError> {
		if components.len() != self.columns.len() {
			return Err(ArchetypeError::WrongComponentCount {
				expected: self.columns.len(),
				got: components.len(),
			});
		}

		for (column, component) in self.columns.iter_mut().zip(components.into_iter()) {
			if column.push_box(component).is_err() {
				return Err(ArchetypeError::MissingComponent("Unknown type"));
			}
		}

		self.entity_column.push(entity);
		debug_assert!(self.validate(), "Columns out of sync after adding row");

		Ok(())
	}

	/// Removes the entity and its components at the specified row
	pub fn remove_row(&mut self, row: usize) -> EntityHandle {
		for column in self.columns.iter_mut() {
			column.swap_remove(row);
		}

		let removed_entity = self
			.entity_column
			.swap_remove(row)
			.expect("row out of bounds in remove_row");
		debug_assert!(self.validate(), "Columns out of sync after removing row");

		removed_entity
	}

	pub fn add_row_typed<T: 'static>(&mut self, entity: EntityHandle, component: T) {
		let mut map = HashMap::new();
		map.insert(TypeId::of::<T>(), Box::new(component) as Box<dyn Any>);
		self.add_row(entity, map);
	}

	/// Add multiple components given as explicit (TypeId, Box<dyn Any>) pairs
	pub fn add_row_multiple(
		&mut self,
		entity: EntityHandle,
		components: impl IntoIterator<Item = (TypeId, Box<dyn Any>)>,
	) {
		let mut map = HashMap::new();
		for (ty, c) in components {
			map.insert(ty, c);
		}
		self.add_row(entity, map);
	}

	/// Spawns a new entity with given components
	pub fn spawn(
		&mut self,
		allocator: &mut crate::EntityAllocator,
		components: impl IntoIterator<Item = (TypeId, Box<dyn Any>)>,
	) -> EntityHandle {
		let entity = allocator.allocate();
		self.add_row_multiple(entity, components);
		entity
	}

	pub fn spawn_typed<T: 'static>(
		&mut self,
		allocator: &mut crate::EntityAllocator,
		component: T,
	) -> EntityHandle {
		let entity = allocator.allocate();
		self.add_row_typed(entity, component);
		entity
	}

	pub fn entity(&self, row: usize) -> Option<&EntityHandle> {
		self.entity_column.get(row)
	}

	pub fn iter_entities(&self) -> impl Iterator<Item = &EntityHandle> {
		self.entity_column.iter()
	}

	pub fn iter_entities_mut(&mut self) -> impl Iterator<Item = &mut EntityHandle> {
		self.entity_column.iter_mut()
	}

	pub fn column_index(&self, ty: TypeId) -> Option<usize> {
		self.column_indices.get(&ty).copied()
	}

	pub fn column_storage(&self, ty: TypeId) -> Option<&dyn AnyStorage> {
		self.column_index(ty).map(|i| self.columns[i].as_ref())
	}

	pub fn column_storage_mut(&mut self, ty: TypeId) -> Option<&mut (dyn AnyStorage + '_)> {
		let idx = self.column_index(ty)?;
		Some(self.columns[idx].as_mut())
	}

	pub fn len(&self) -> usize {
		self.entity_column.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entity_column.len() == 0
	}

	pub fn key(&self) -> &ArchetypeKey {
		&self.key
	}

	pub fn has_all(&self, types: &[TypeId]) -> bool {
		types.iter().all(|ty| self.column_indices.contains_key(ty))
	}

	pub fn has_any(&self, types: &[TypeId]) -> bool {
		types.iter().any(|ty| self.column_indices.contains_key(ty))
	}

	pub fn column<T: 'static>(&self, ty: TypeId) -> Option<&Column<T>> {
		self.column_storage(ty)?.as_any().downcast_ref::<Column<T>>()
	}

	pub fn column_mut<T: 'static>(&mut self, ty: TypeId) -> Option<&mut Column<T>> {
		self.column_storage_mut(ty)?.as_any_mut().downcast_mut::<Column<T>>()
	}

	/// Borrow two different columns at once: one mutable and one immutable.
	pub fn columns2_mut_immut<A: 'static, B: 'static>(
		&mut self,
		mut_ty: TypeId,
		read_ty: TypeId,
	) -> Option<(&mut Column<A>, &Column<B>)> {
		let mut_idx = self.column_index(mut_ty)?;
		let read_idx = self.column_index(read_ty)?;
		if mut_idx == read_idx {
			return None;
		}

		if mut_idx < read_idx {
			let (left, right) = self.columns.split_at_mut(read_idx);
			let mut_col = left[mut_idx]
				.as_mut()
				.as_any_mut()
				.downcast_mut::<Column<A>>()?;
			let read_col = right[0]
				.as_ref()
				.as_any()
				.downcast_ref::<Column<B>>()?;
			Some((mut_col, read_col))
		} else {
			let (left, right) = self.columns.split_at_mut(mut_idx);
			let read_col = left[read_idx]
				.as_ref()
				.as_any()
				.downcast_ref::<Column<B>>()?;
			let mut_col = right[0]
				.as_mut()
				.as_any_mut()
				.downcast_mut::<Column<A>>()?;
			Some((mut_col, read_col))
		}
	}

	pub fn new_from_registry(keys: &[ComponentKey], registry: &ComponentRegistry) -> Self {
		let constructors: HashMap<_, _> = keys.iter()
			.map(|k| (k.type_id, *registry.get_constructor(k).expect("Unregistered")))
			.collect();
		Self::new(keys.to_vec(), &constructors)
	}

	pub fn debug_summary(&self) {
		debug!(
            "Archetype {}: {} entities, columns: {}",
            self.key,
            self.entity_column.len(),
            self.key.column_names_joined()
        );
	}

	pub fn validate(&self) -> bool {
		self.columns.iter().all(|c| c.len() == self.entity_column.len())
	}

	pub fn row(&mut self, index: usize) -> Row<'_> {
		Row::new(&mut self.columns, &mut self.entity_column, index)
	}
}
