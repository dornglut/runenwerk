use std::any::TypeId;
use std::fmt;
use crate::ComponentKey;

// src/table/archetype_key

/// Unique key identifying an archetype by its set of component types.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ArchetypeKey {
	pub components: Vec<ComponentKey>,
}

impl ArchetypeKey {
	/// Creates a new archetype key from a list of component types. /// Types are sorted for canonical ordering.
	pub fn new(mut components: Vec<ComponentKey>) -> Self {
		components.sort_by_key(|c| c.type_id);
		Self { components }
	}
	/// Returns an iterator over TypeIds of the components
	pub fn types(&self) -> impl Iterator<Item = TypeId> + '_ {
		self.components.iter().map(|c| c.type_id)
	}

	/// Returns an iterator over component names as &str
	pub fn column_names(&self) -> impl Iterator<Item = &str> + '_ {
		self.components.iter().map(|c| c.name.as_str())
	}

	/// Returns a single comma-separated string of component names
	pub fn column_names_joined(&self) -> String {
		self.column_names().collect::<Vec<_>>().join(", ")
	}
}

impl fmt::Display for ArchetypeKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "ArchetypeKey({})", self.column_names_joined())
	}
}