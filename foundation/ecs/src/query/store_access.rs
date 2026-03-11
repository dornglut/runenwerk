// Owner: Grotto Quest ECS - Query Runtime
use crate::component::Component;
use crate::entity::Entity;
use crate::world::TypedStore;

pub(crate) trait StoreAccess<T: Component> {
    fn get(&self, entity: Entity) -> Option<&T>;
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T>;
}

impl<T: Component> StoreAccess<T> for TypedStore<T> {
    fn get(&self, entity: Entity) -> Option<&T> {
        self.values.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.values.get_mut(&entity)
    }
}
