use crate::{Component, World};
use std::any::Any;

/// Group of components that can be spawned atomically as one entity.
pub trait ComponentBundle {
    fn register_components(world: &mut World);
    fn into_components(self) -> Vec<Box<dyn Any>>;
}

impl<T: Component> ComponentBundle for T {
    fn register_components(world: &mut World) {
        world.ensure_component_registered::<T>();
    }

    fn into_components(self) -> Vec<Box<dyn Any>> {
        vec![Box::new(self) as Box<dyn Any>]
    }
}

macro_rules! impl_component_bundle_tuple {
    ($(($ty:ident, $var:ident)),+ $(,)?) => {
        impl<$($ty: Component),+> ComponentBundle for ($($ty,)+) {
            fn register_components(world: &mut World) {
                $(world.ensure_component_registered::<$ty>();)+
            }

            fn into_components(self) -> Vec<Box<dyn Any>> {
                let ($($var,)+) = self;
                vec![$(Box::new($var) as Box<dyn Any>),+]
            }
        }
    };
}

impl_component_bundle_tuple!((A, a), (B, b));
impl_component_bundle_tuple!((A, a), (B, b), (C, c));
impl_component_bundle_tuple!((A, a), (B, b), (C, c), (D, d));
impl_component_bundle_tuple!((A, a), (B, b), (C, c), (D, d), (E, e));
impl_component_bundle_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f));
impl_component_bundle_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f), (G, g));
impl_component_bundle_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h)
);
