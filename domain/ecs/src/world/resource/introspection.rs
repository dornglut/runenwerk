// Owner: ecs World Resource - Introspection APIs
use crate::reflect::{ReflectValueMut, ReflectValueRef, TypeInfo};
use crate::world::World;
use std::any::TypeId;

impl World {
    pub fn resource_type_info(&self, type_id: TypeId) -> Option<&'static TypeInfo> {
        self.reflected_resource_types
            .get(&type_id)
            .map(|registration| registration.type_info)
    }

    pub fn has_registered_resource_type(&self, type_id: TypeId) -> bool {
        self.reflected_resource_types.contains_key(&type_id)
    }

    pub fn registered_resource_type_ids(&self) -> Vec<TypeId> {
        self.reflected_resource_types.keys().copied().collect()
    }

    pub fn live_resource_type_ids(&self) -> Vec<TypeId> {
        self.resources.keys().copied().collect()
    }

    pub fn live_registered_resource_types(&self) -> Vec<&'static TypeInfo> {
        self.live_resource_type_ids()
            .into_iter()
            .filter_map(|type_id| self.resource_type_info(type_id))
            .collect()
    }

    pub fn reflected_resource_value_ref(&self, type_id: TypeId) -> Option<ReflectValueRef<'_>> {
        let registration = self.reflected_resource_types.get(&type_id)?;
        (registration.value_ref)(self)
    }

    pub fn reflected_resource_value_mut(&mut self, type_id: TypeId) -> Option<ReflectValueMut<'_>> {
        let registration = self.reflected_resource_types.get(&type_id).copied()?;
        (registration.value_mut)(self)
    }
}
