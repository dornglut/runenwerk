use super::render_executor_registry::RenderFrameDataRegistry;
use std::any::{TypeId, type_name};
use std::collections::BTreeMap;

type FrameResourceCollector = for<'a> fn(&'a ecs::World, &mut RenderFrameDataRegistry<'a>);

#[derive(Debug, Clone)]
struct FrameResourceBinding {
    type_name: &'static str,
    collect: FrameResourceCollector,
}

#[derive(Debug, Default, Clone)]
pub struct RenderFrameResourceBindings {
    bindings: BTreeMap<TypeId, FrameResourceBinding>,
}

impl RenderFrameResourceBindings {
    pub fn register_resource<T>(&mut self)
    where
        T: Send + Sync + 'static,
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
        T: Send + Sync + 'static,
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
        T: Send + Sync + 'static,
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
    T: Send + Sync + 'static,
{
    if let Ok(resource) = world.resource::<T>() {
        frame_data.insert(resource);
    }
}

#[cfg(test)]
mod tests {
    use super::RenderFrameResourceBindings;
    use crate::plugins::render::domain::RenderFrameDataRegistry;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestResourceA(pub u32);

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestResourceB(pub &'static str);

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
