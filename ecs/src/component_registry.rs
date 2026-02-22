use std::any::TypeId;
use std::collections::HashMap;
use crate::{AnyStorage, Column};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComponentKey {
	pub type_id: TypeId,
	pub name: String,
}

impl ComponentKey {
	pub fn new<T: 'static>(name: impl Into<String>) -> Self {
		Self { type_id: TypeId::of::<T>(), name: name.into() }
	}
}

pub struct ComponentRegistry {
	constructors: HashMap<ComponentKey, fn() -> Box<dyn AnyStorage>>,
	keys_by_type: HashMap<TypeId, ComponentKey>,
}

impl ComponentRegistry {
	pub fn new() -> Self {
		Self {
			constructors: HashMap::new(),
			keys_by_type: HashMap::new(),
		}
	}

	pub fn register<T: 'static>(&mut self, name: impl Into<String>) -> ComponentKey {
		let key = ComponentKey::new::<T>(name);
		self.constructors.insert(key.clone(), || Box::new(Column::<T>::new()));
		self.keys_by_type.insert(key.type_id, key.clone());
		key
	}

	pub fn get_constructor(&self, key: &ComponentKey) -> Option<&fn() -> Box<dyn AnyStorage>> {
		self.constructors.get(key)
	}

	pub fn get_key_by_type(&self, type_id: TypeId) -> Option<&ComponentKey> {
		self.keys_by_type.get(&type_id)
	}
}
