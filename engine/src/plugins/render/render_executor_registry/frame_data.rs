use super::*;

// Owner: Grotto Quest Engine - Render Domain
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

// Owner: Grotto Quest Engine - Render Domain
