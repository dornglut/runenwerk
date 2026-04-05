// Owner: ecs World Component - Reflection Registration APIs
use crate::reflect::Reflect;
use crate::world::world::World;
use std::any::TypeId;

impl World {
	pub fn register_component_type<T>(&mut self)
	where
		T: crate::Component + Reflect,
	{
		self.ensure_reflected_component_registered::<T>();
	}

	pub fn registered_component_types(&self) -> Vec<&'static crate::reflect::TypeInfo> {
		self.reflected_component_types
			.values()
			.map(|registration| registration.type_info)
			.collect()
	}

	pub(crate) fn ensure_reflected_component_registered<T>(&mut self)
	where
		T: crate::Component + Reflect,
	{
		self.__register_component::<T>();
		let type_id = TypeId::of::<T>();
		self.reflected_component_types.entry(type_id).or_insert_with(|| {
			let _ = crate::reflect::register_reflect_type::<T>();
			crate::reflect::reflected_component_registration::<T>()
		});
	}
}