use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct RenderFrameDataRegistry<'a> {
    by_type: HashMap<TypeId, &'a dyn Any>,
}

impl<'a> RenderFrameDataRegistry<'a> {
    // Projection helper for adapter APIs and tests.
    // Runtime submit/render path uses PreparedRenderFrame and does not consume this registry.
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

    pub fn insert_by_type_id(&mut self, type_id: TypeId, value: &'a dyn Any) {
        self.by_type.insert(type_id, value);
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

    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<&'a dyn Any> {
        self.by_type.get(&type_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::RenderFrameDataRegistry;

    #[test]
    fn frame_data_registry_supports_lookup() {
        let value = 42_u32;
        let registry = RenderFrameDataRegistry::new().with(&value);
        assert_eq!(registry.get::<u32>(), Some(&42_u32));
        assert!(registry.get::<u64>().is_none());
    }

    #[test]
    fn frame_data_registry_supports_type_id_insert_lookup() {
        let value = 9_u32;
        let mut registry = RenderFrameDataRegistry::new();
        registry.insert_by_type_id(std::any::TypeId::of::<u32>(), &value);
        assert_eq!(registry.get::<u32>(), Some(&9_u32));
    }
}
