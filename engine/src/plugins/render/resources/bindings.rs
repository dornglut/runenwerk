use std::any::{Any, TypeId, type_name};
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub struct RenderFrameDataRegistry<'a> {
    by_type: HashMap<TypeId, &'a dyn Any>,
}

impl<'a> RenderFrameDataRegistry<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<T: 'static>(mut self, value: &'a T) -> Self {
        self.insert(value);
        self
    }

    pub fn insert<T: 'static>(&mut self, value: &'a T) {
        self.by_type.insert(TypeId::of::<T>(), value);
    }

    pub fn extend_from(&mut self, other: &RenderFrameDataRegistry<'a>) {
        self.by_type.extend(
            other
                .by_type
                .iter()
                .map(|(type_id, value)| (*type_id, *value)),
        );
    }

    pub fn get<T: 'static>(&self) -> Option<&'a T> {
        self.by_type
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }
}

type FrameResourceCollector = for<'a> fn(&'a ecs::World, &mut RenderFrameDataRegistry<'a>);

#[derive(Debug, Clone)]
struct FrameResourceBinding {
    type_name: &'static str,
    collect: FrameResourceCollector,
}

#[derive(Debug, Default, Clone, ecs::Component)]
pub struct RenderFrameResourceBindings {
    bindings: BTreeMap<TypeId, FrameResourceBinding>,
}

impl RenderFrameResourceBindings {
    pub fn register_resource<T>(&mut self)
    where
        T: ecs::Component + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        self.bindings.insert(
            type_id,
            FrameResourceBinding {
                type_name: type_name::<T>(),
                collect: collect_resource::<T>,
            },
        );
    }

    pub fn unregister_resource<T>(&mut self) -> bool
    where
        T: ecs::Component + Send + Sync,
    {
        self.bindings.remove(&TypeId::of::<T>()).is_some()
    }

    pub fn collect_frame_data<'a>(
        &self,
        world: &'a ecs::World,
        frame_data: &mut RenderFrameDataRegistry<'a>,
    ) {
        for binding in self.bindings.values() {
            (binding.collect)(world, frame_data);
        }
    }

    pub fn contains_resource<T>(&self) -> bool
    where
        T: ecs::Component + Send + Sync,
    {
        self.bindings.contains_key(&TypeId::of::<T>())
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        self.bindings
            .values()
            .map(|binding| binding.type_name)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

fn collect_resource<'a, T>(world: &'a ecs::World, frame_data: &mut RenderFrameDataRegistry<'a>)
where
    T: ecs::Component + Send + Sync,
{
    if let Ok(resource) = world.resource::<T>() {
        frame_data.insert(resource);
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderFrameDataRegistry, RenderFrameResourceBindings};

    #[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
    struct TestResourceA(pub u32);

    #[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
    struct TestResourceB(pub &'static str);

    #[test]
    fn frame_data_registry_supports_lookup() {
        let value = 42_u32;
        let registry = RenderFrameDataRegistry::new().with(&value);
        assert_eq!(registry.get::<u32>(), Some(&42_u32));
        assert!(registry.get::<u64>().is_none());
    }

    #[test]
    fn collect_frame_data_skips_missing_resources() {
        let world = ecs::World::new();
        let mut bindings = RenderFrameResourceBindings::default();
        bindings.register_resource::<TestResourceA>();

        let mut frame_data = RenderFrameDataRegistry::new();
        bindings.collect_frame_data(&world, &mut frame_data);

        assert!(frame_data.get::<TestResourceA>().is_none());
    }

    #[test]
    fn unregister_resource_removes_binding() {
        let mut bindings = RenderFrameResourceBindings::default();
        bindings.register_resource::<TestResourceA>();
        assert!(bindings.contains_resource::<TestResourceA>());
        assert_eq!(bindings.len(), 1);

        assert!(bindings.unregister_resource::<TestResourceA>());
        assert!(!bindings.contains_resource::<TestResourceA>());
        assert!(bindings.is_empty());
    }

    #[test]
    fn register_resource_is_idempotent_per_type() {
        let mut bindings = RenderFrameResourceBindings::default();
        bindings.register_resource::<TestResourceA>();
        bindings.register_resource::<TestResourceA>();
        bindings.register_resource::<TestResourceB>();

        assert_eq!(bindings.len(), 2);
        let names = bindings.registered_type_names();
        assert!(names.contains(&std::any::type_name::<TestResourceA>()));
        assert!(names.contains(&std::any::type_name::<TestResourceB>()));
    }

    #[test]
    fn collect_frame_data_supports_world_resources() {
        let mut world = ecs::World::new();
        world.insert_resource(TestResourceA(7));
        world.insert_resource(TestResourceB("typed"));

        let mut bindings = RenderFrameResourceBindings::default();
        bindings.register_resource::<TestResourceA>();
        bindings.register_resource::<TestResourceB>();

        let mut frame_data = RenderFrameDataRegistry::new();
        bindings.collect_frame_data(&world, &mut frame_data);

        assert_eq!(frame_data.get::<TestResourceA>(), Some(&TestResourceA(7)));
        assert_eq!(
            frame_data.get::<TestResourceB>(),
            Some(&TestResourceB("typed"))
        );
    }
}
