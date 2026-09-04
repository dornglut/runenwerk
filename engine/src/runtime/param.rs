use ecs::{QueryAccess, SystemParam, SystemParamError, World};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub use ecs::{
    BroadcastReader, BroadcastWriter, Commands, Query, Res, ResMut, TickBufferDrainer,
    TickBufferReader, TickBufferWriter, WorkQueueDrainer, WorkQueueReader, WorkQueueWriter,
};

pub struct WorldRef {
    world: NonNull<World>,
}

impl WorldRef {
    fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
        }
    }
}

impl Deref for WorldRef {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_ref() }
    }
}

// Safety: this legacy read-only whole-World parameter reports no borrow facts and
// is retained only until the C3 lifetime-bound extraction cut removes its unused
// surface. It does not manufacture mutable access.
unsafe impl<'w> SystemParam<'w> for WorldRef {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        QueryAccess::default()
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(Self::new(world))
    }
}

pub struct WorldMut {
    world: NonNull<World>,
}

impl WorldMut {
    fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
        }
    }
}

impl Deref for WorldMut {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_ref() }
    }
}

impl DerefMut for WorldMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut() }
    }
}

// Safety: WorldMut declares exclusive-world access. C3 registration rejects any
// sibling immediate World access before extraction, so recovering the invocation's
// unique World borrow is consistent with the declared access contract.
unsafe impl<'w> SystemParam<'w> for WorldMut {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        QueryAccess::exclusive_world()
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(Self::new(world))
    }
}
