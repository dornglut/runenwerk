use crate::plugins::render::{RenderPassId, RenderResourceId};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PassHandle {
    id: RenderPassId,
}

impl PassHandle {
    pub(crate) fn new(id: RenderPassId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &RenderPassId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorageArrayHandle<T> {
    id: RenderResourceId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> StorageArrayHandle<T> {
    pub(crate) fn new(id: RenderResourceId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> &RenderResourceId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniformHandle<U> {
    id: RenderResourceId,
    _marker: PhantomData<fn() -> U>,
}

impl<U> UniformHandle<U> {
    pub(crate) fn new(id: RenderResourceId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> &RenderResourceId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoubleBufferHandle<T> {
    name: String,
    a: StorageArrayHandle<T>,
    b: StorageArrayHandle<T>,
}

impl<T> DoubleBufferHandle<T> {
    pub(crate) fn new(name: String, a: StorageArrayHandle<T>, b: StorageArrayHandle<T>) -> Self {
        Self { name, a, b }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn a(&self) -> &StorageArrayHandle<T> {
        &self.a
    }

    pub fn b(&self) -> &StorageArrayHandle<T> {
        &self.b
    }
}
