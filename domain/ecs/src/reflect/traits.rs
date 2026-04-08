//! File: domain/ecs/src/reflect/traits.rs
//! Purpose: Reflection trait contracts for ECS types.

use crate::reflect::{ReflectValueMut, ReflectValueRef, TypeInfo};
use crate::{Component, Resource};

pub trait Reflect: 'static {
    /// File: domain/ecs/src/reflect/traits.rs
    /// Method: type_info
    fn type_info() -> &'static TypeInfo
    where
        Self: Sized;

    /// File: domain/ecs/src/reflect/traits.rs
    /// Method: stable_name
    fn stable_name() -> &'static str
    where
        Self: Sized,
    {
        Self::type_info().stable_name
    }

    /// File: domain/ecs/src/reflect/traits.rs
    /// Method: reflect_ref
    fn reflect_ref(&self) -> ReflectValueRef<'_>
    where
        Self: Sized,
    {
        ReflectValueRef::new(self)
    }

    /// File: domain/ecs/src/reflect/traits.rs
    /// Method: reflect_mut
    fn reflect_mut(&mut self) -> ReflectValueMut<'_>
    where
        Self: Sized,
    {
        ReflectValueMut::new(self)
    }
}

pub trait ReflectComponent: Reflect + Component {}
pub trait ReflectResource: Reflect + Resource {}

impl<T> ReflectComponent for T where T: Reflect + Component {}
impl<T> ReflectResource for T where T: Reflect + Resource {}
