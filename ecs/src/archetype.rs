use std::any::TypeId;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::{AnyStorage, Entity};

// src/archetype.rs

/// An archetype groups entities with the same set of component types.
/// Think of it as a table:
/// - Rows = entities
/// - Columns = component storages
/// Methods: `new()`, `add_entity()`, `remove_entity()`, `contains_components()`.
pub struct Archetype {
	pub entities: Vec<Entity>,
	pub component_types: Vec<TypeId>,
	pub component_indices: HashMap<TypeId, usize>,
	pub storages: Vec<Box<dyn AnyStorage>>
}

impl Archetype {
	pub fn new(
		mut component_types: Vec<TypeId>,
		constructors: &HashMap<TypeId, fn() -> Box<dyn AnyStorage>>,
	) -> Self {
		component_types.sort();
		let mut component_indices = HashMap::new();
		let mut storages = Vec::new();

		for (i, ty) in component_types.iter().enumerate() {
			component_indices.insert(*ty, i);

			let ctor = constructors.get(ty)
				.expect(&format!("Component type `{:?}` does not exist.", ty));
			storages.push(ctor());
		}
		Self {
			entities: vec![],
			component_types,
			component_indices,
			storages,
		}
	}

	pub fn add_entity(&mut self, entity: Entity) {
		self.entities.push(entity);
	}

	pub fn remove_entity(&mut self, index: usize) -> Entity {
		for storage in self.storages.iter_mut() {
			storage.swap_remove(index);
		}
		self.entities.swap_remove(index)
	}

	pub fn contains_components(&self, query: &[TypeId]) -> bool {
		query.iter().all(|ty| self.component_types.contains(ty))
	}

	pub fn storage_index(&self, ty: TypeId) -> Option<usize> {
		self.component_indices.get(&ty).copied()
	}

	pub fn storage(&self, ty: TypeId) -> Option<&dyn AnyStorage> {
		match self.component_indices.get(&ty) {
			Some(&i) => Some(self.storages[i].as_ref()),
			None => None,
		}
	}

	pub fn storage_mut(&mut self, ty: TypeId) -> Option<&mut dyn AnyStorage> {
		match self.component_indices.get(&ty) {
			Some(&i) => Some(self.storages[i].as_mut()),
			None => None,
		}
	}

	pub fn len(&self) -> usize {
		self.entities.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entities.is_empty()
	}
}


/// Unique key identifying an archetype by its set of component types
#[derive(Clone)]
#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub struct ArchetypeKey {
	pub types: Vec<TypeId>,
}

impl ArchetypeKey {
	pub fn new(mut types: Vec<TypeId>) -> Self {
		types.sort();
		Self { types }
	}

	pub fn types(&self) -> &[TypeId] {
		&self.types
	}
}

impl Hash for ArchetypeKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for ty in &self.types {
			ty.hash(state);
		}
	}
}