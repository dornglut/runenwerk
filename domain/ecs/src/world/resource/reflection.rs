// Owner: ecs World Resource - Reflection Registration APIs
use crate::component::Resource;
use crate::reflect::Reflect;
use crate::world::world::World;
use std::any::TypeId;

impl World {
    pub fn register_resource_type<T>(&mut self)
    where
        T: Resource + Reflect,
    {
        self.ensure_reflected_resource_registered::<T>();
    }

    pub fn registered_resource_types(&self) -> Vec<&'static crate::reflect::TypeInfo> {
        self.reflected_resource_types
            .values()
            .map(|registration| registration.type_info)
            .collect()
    }

    pub fn insert_registered_resource<R>(&mut self, resource: R)
    where
        R: Resource + Reflect,
    {
        self.ensure_reflected_resource_registered::<R>();
        self.insert_resource(resource);
    }

    pub(crate) fn ensure_reflected_resource_registered<T>(&mut self)
    where
        T: crate::Resource + crate::reflect::Reflect,
    {
        let type_id = TypeId::of::<T>();
        self.reflected_resource_types
            .entry(type_id)
            .or_insert_with(|| {
                let _ = crate::reflect::register_reflect_type::<T>();
                crate::reflect::reflected_resource_registration::<T>()
            });
    }
}
