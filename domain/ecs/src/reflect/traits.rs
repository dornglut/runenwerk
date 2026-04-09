//! File: domain/ecs/src/reflect/traits.rs
//! Purpose: Reflection trait contracts for ECS types.

use crate::reflect::{ReflectValueMut, ReflectValueRef, TypeInfo};
use crate::{Component, Resource};

pub trait Reflect: 'static {
    fn type_info() -> &'static TypeInfo
    where
        Self: Sized;

    fn stable_name() -> &'static str
    where
        Self: Sized,
    {
        Self::type_info().stable_name
    }

    fn reflect_ref(&self) -> ReflectValueRef<'_>
    where
        Self: Sized,
    {
        ReflectValueRef::new(self)
    }

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
