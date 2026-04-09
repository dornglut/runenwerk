use std::any::TypeId;
use std::collections::HashMap;

use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_scene::SceneEntitySnapshot;

type RemovedComponentKey = (EntityId, ComponentTypeId);

trait RemovedComponentValue {
    fn restore(
        self: Box<Self>,
        world: &mut ecs::World,
        entity: ecs::Entity,
    ) -> Result<(), &'static str>;
}

struct RemovedComponent<T> {
    value: T,
}

impl<T> RemovedComponentValue for RemovedComponent<T>
where
    T: ecs::Component + 'static,
{
    fn restore(
        self: Box<Self>,
        world: &mut ecs::World,
        entity: ecs::Entity,
    ) -> Result<(), &'static str> {
        world
            .insert(entity, self.value)
            .map_err(|_| "failed to restore removed component")
    }
}

struct SceneEntityRecord {
    ecs_entity: ecs::Entity,
    display_name: String,
    parent: Option<EntityId>,
}

struct ComponentRegistration {
    rust_type_id: TypeId,
    display_name: String,
    insert_default: fn(&mut ecs::World, ecs::Entity) -> Result<(), &'static str>,
    remove_and_capture: fn(
        &mut ecs::World,
        ecs::Entity,
        &mut HashMap<RemovedComponentKey, Box<dyn RemovedComponentValue>>,
        RemovedComponentKey,
    ) -> Result<(), &'static str>,
}

#[derive(Default)]
pub struct EditorRuntimeIdRegistry {
    next_entity_id: u64,
    entities: HashMap<EntityId, SceneEntityRecord>,
    component_types: HashMap<ComponentTypeId, ComponentRegistration>,
    resource_type_ids: HashMap<ResourceTypeId, TypeId>,
    removed_components: HashMap<RemovedComponentKey, Box<dyn RemovedComponentValue>>,
}

impl EditorRuntimeIdRegistry {
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
            ..Self::default()
        }
    }

    pub fn allocate_entity_id(&mut self) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    pub fn register_entity(
        &mut self,
        editor_id: EntityId,
        ecs_entity: ecs::Entity,
        display_name: impl Into<String>,
        parent: Option<EntityId>,
    ) {
        self.entities.insert(
            editor_id,
            SceneEntityRecord {
                ecs_entity,
                display_name: display_name.into(),
                parent,
            },
        );
    }

    pub fn unregister_entity(&mut self, editor_id: EntityId) -> Option<SceneEntitySnapshot> {
        self.entities
            .remove(&editor_id)
            .map(|record| SceneEntitySnapshot::new(editor_id, record.display_name, record.parent))
    }

    pub fn resolve_entity(&self, entity_id: EntityId) -> Option<ecs::Entity> {
        self.entities
            .get(&entity_id)
            .map(|record| record.ecs_entity)
    }

    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.keys().copied()
    }

    pub fn entity_snapshot(&self, entity_id: EntityId) -> Option<SceneEntitySnapshot> {
        self.entities.get(&entity_id).map(|record| {
            SceneEntitySnapshot::new(entity_id, record.display_name.clone(), record.parent)
        })
    }

    pub fn entity_display_name(&self, entity_id: EntityId) -> Option<&str> {
        self.entities
            .get(&entity_id)
            .map(|record| record.display_name.as_str())
    }

    pub fn parent_of(&self, entity_id: EntityId) -> Option<Option<EntityId>> {
        self.entities.get(&entity_id).map(|record| record.parent)
    }

    pub fn children_of(&self, parent: Option<EntityId>) -> Vec<EntityId> {
        self.entities
            .iter()
            .filter_map(|(entity_id, record)| (record.parent == parent).then_some(*entity_id))
            .collect()
    }

    pub fn has_children(&self, entity_id: EntityId) -> bool {
        self.entities
            .values()
            .any(|record| record.parent == Some(entity_id))
    }

    pub fn rename_entity(
        &mut self,
        entity_id: EntityId,
        new_display_name: impl Into<String>,
    ) -> Result<String, &'static str> {
        let record = self
            .entities
            .get_mut(&entity_id)
            .ok_or("editor entity is not registered")?;

        let previous = std::mem::replace(&mut record.display_name, new_display_name.into());
        Ok(previous)
    }

    pub fn would_create_cycle(&self, entity: EntityId, candidate_parent: EntityId) -> bool {
        let mut current = Some(candidate_parent);

        while let Some(current_entity) = current {
            if current_entity == entity {
                return true;
            }

            current = self.parent_of(current_entity).flatten();
        }

        false
    }

    pub fn reparent_entity(
        &mut self,
        entity_id: EntityId,
        new_parent: Option<EntityId>,
    ) -> Result<Option<EntityId>, &'static str> {
        let record = self
            .entities
            .get_mut(&entity_id)
            .ok_or("editor entity is not registered")?;

        let previous_parent = record.parent;
        record.parent = new_parent;
        Ok(previous_parent)
    }

    pub fn register_component_type<T>(&mut self, editor_id: ComponentTypeId)
    where
        T: ecs::Component + ecs::Reflect + Default + 'static,
    {
        self.component_types.insert(
            editor_id,
            ComponentRegistration {
                rust_type_id: TypeId::of::<T>(),
                display_name: short_type_name::<T>(),
                insert_default: insert_default_component::<T>,
                remove_and_capture: remove_and_capture_component::<T>,
            },
        );
    }

    pub fn component_type_ids(&self) -> impl Iterator<Item = ComponentTypeId> + '_ {
        self.component_types.keys().copied()
    }

    pub fn resolve_component_rust_type_id(
        &self,
        component_type: ComponentTypeId,
    ) -> Option<TypeId> {
        self.component_types
            .get(&component_type)
            .map(|registration| registration.rust_type_id)
    }

    pub fn component_display_name(&self, component_type: ComponentTypeId) -> Option<&str> {
        self.component_types
            .get(&component_type)
            .map(|registration| registration.display_name.as_str())
    }

    pub fn add_default_component(
        &self,
        world: &mut ecs::World,
        entity: ecs::Entity,
        component_type: ComponentTypeId,
    ) -> Result<(), &'static str> {
        let registration = self
            .component_types
            .get(&component_type)
            .ok_or("component type is not registered in editor runtime")?;

        (registration.insert_default)(world, entity)
    }

    pub fn remove_component_and_capture(
        &mut self,
        world: &mut ecs::World,
        editor_entity: EntityId,
        ecs_entity: ecs::Entity,
        component_type: ComponentTypeId,
    ) -> Result<(), &'static str> {
        let registration = self
            .component_types
            .get(&component_type)
            .ok_or("component type is not registered in editor runtime")?;

        (registration.remove_and_capture)(
            world,
            ecs_entity,
            &mut self.removed_components,
            (editor_entity, component_type),
        )
    }

    pub fn restore_removed_component(
        &mut self,
        world: &mut ecs::World,
        editor_entity: EntityId,
        ecs_entity: ecs::Entity,
        component_type: ComponentTypeId,
    ) -> Result<(), &'static str> {
        let key = (editor_entity, component_type);

        if let Some(removed) = self.removed_components.remove(&key) {
            return removed.restore(world, ecs_entity);
        }

        self.add_default_component(world, ecs_entity, component_type)
    }

    pub fn register_resource_type<T: 'static>(&mut self, editor_id: ResourceTypeId) {
        self.resource_type_ids.insert(editor_id, TypeId::of::<T>());
    }

    pub fn resolve_resource_rust_type_id(&self, resource_type: ResourceTypeId) -> Option<TypeId> {
        self.resource_type_ids.get(&resource_type).copied()
    }
}

fn short_type_name<T>() -> String {
    let full = std::any::type_name::<T>();
    full.rsplit("::").next().unwrap_or(full).to_string()
}

fn insert_default_component<T>(
    world: &mut ecs::World,
    entity: ecs::Entity,
) -> Result<(), &'static str>
where
    T: ecs::Component + Default + 'static,
{
    world
        .insert(entity, T::default())
        .map_err(|_| "failed to insert default component")
}

fn remove_and_capture_component<T>(
    world: &mut ecs::World,
    entity: ecs::Entity,
    store: &mut HashMap<RemovedComponentKey, Box<dyn RemovedComponentValue>>,
    key: RemovedComponentKey,
) -> Result<(), &'static str>
where
    T: ecs::Component + 'static,
{
    let value = world
        .remove::<T>(entity)
        .map_err(|_| "failed to remove component")?;

    store.insert(key, Box::new(RemovedComponent::<T> { value }));
    Ok(())
}
