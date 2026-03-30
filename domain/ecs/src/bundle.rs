use crate::{Component, Entity, EntityError, World};

/// Group of components that can be inserted or removed together.
pub trait Bundle: Sized + 'static {
    fn register(world: &mut World);
    fn insert(self, world: &mut World, entity: Entity) -> Result<(), EntityError>;
    fn remove(world: &mut World, entity: Entity) -> Result<Self, EntityError>;
}

impl<T: Component> Bundle for T {
    fn register(world: &mut World) {
        world.__register_component::<T>();
    }

    fn insert(self, world: &mut World, entity: Entity) -> Result<(), EntityError> {
        world.__insert_component(entity, self)
    }

    fn remove(world: &mut World, entity: Entity) -> Result<Self, EntityError> {
        world.__remove_component::<T>(entity)
    }
}

macro_rules! impl_bundle_tuple {
    ($(($ty:ident, $var:ident)),+ $(,)?) => {
        impl<$($ty: Component),+> Bundle for ($($ty,)+) {
            fn register(world: &mut World) {
                $(world.__register_component::<$ty>();)+
            }

            fn insert(self, world: &mut World, entity: Entity) -> Result<(), EntityError> {
                let ($($var,)+) = self;
                $(world.__insert_component(entity, $var)?;)+
                Ok(())
            }

            fn remove(world: &mut World, entity: Entity) -> Result<Self, EntityError> {
                Ok((
                    $(world.__remove_component::<$ty>(entity)?,)+
                ))
            }
        }
    };
}

impl_bundle_tuple!((A, a), (B, b));
impl_bundle_tuple!((A, a), (B, b), (C, c));
impl_bundle_tuple!((A, a), (B, b), (C, c), (D, d));
impl_bundle_tuple!((A, a), (B, b), (C, c), (D, d), (E, e));
impl_bundle_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f));
