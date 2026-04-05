// Owner: ecs World Resource - Introspection APIs
use crate::reflect::{ReflectValueMut, ReflectValueRef, TypeInfo};
use crate::world::world::World;
use std::any::TypeId;

impl World {
	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: resource_type_info
	pub fn resource_type_info(&self, type_id: TypeId) -> Option<&'static TypeInfo> {
		self.reflected_resource_types
			.get(&type_id)
			.map(|registration| registration.type_info)
	}

	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: has_registered_resource_type
	pub fn has_registered_resource_type(&self, type_id: TypeId) -> bool {
		self.reflected_resource_types.contains_key(&type_id)
	}

	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: registered_resource_type_ids
	pub fn registered_resource_type_ids(&self) -> Vec<TypeId> {
		self.reflected_resource_types.keys().copied().collect()
	}

	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: live_resource_type_ids
	pub fn live_resource_type_ids(&self) -> Vec<TypeId> {
		self.resources.keys().copied().collect()
	}

	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: live_registered_resource_types
	pub fn live_registered_resource_types(&self) -> Vec<&'static TypeInfo> {
		self.live_resource_type_ids()
			.into_iter()
			.filter_map(|type_id| self.resource_type_info(type_id))
			.collect()
	}

	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: reflected_resource_value_ref
	pub fn reflected_resource_value_ref(
		&self,
		type_id: TypeId,
	) -> Option<ReflectValueRef<'_>> {
		let registration = self.reflected_resource_types.get(&type_id)?;
		(registration.value_ref)(self)
	}

	/// File: domain/ecs/src/world/resource/introspection.rs
	/// Method: reflected_resource_value_mut
	pub fn reflected_resource_value_mut(
		&mut self,
		type_id: TypeId,
	) -> Option<ReflectValueMut<'_>> {
		let registration = self.reflected_resource_types.get(&type_id).copied()?;
		(registration.value_mut)(self)
	}
}