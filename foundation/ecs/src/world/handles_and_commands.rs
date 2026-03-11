// Owner: ECS World - Borrow Wrappers, Entity Views, and Commands
use super::world_struct::World;
use crate::bundle::Bundle;
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::{CommandError, EntityError};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct Mut<'a, T> {
    pub(super) value: &'a mut T,
}

impl<'a, T> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct EntityRef<'w> {
    pub(super) world: &'w World,
    pub(super) entity: Entity,
}

impl<'w> EntityRef<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.world.contains_component::<T>(self.entity)
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    pub fn require<T: Component>(&self) -> Result<&T, EntityError> {
        self.world.require::<T>(self.entity)
    }
}

pub struct EntityMut<'w> {
    pub(super) world: &'w mut World,
    pub(super) entity: Entity,
}

impl<'w> EntityMut<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.world.contains_component::<T>(self.entity)
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    pub fn get_mut<T: Component>(&mut self) -> Option<Mut<'_, T>> {
        self.world.get_mut::<T>(self.entity)
    }

    pub fn require<T: Component>(&self) -> Result<&T, EntityError> {
        self.world.require::<T>(self.entity)
    }

    pub fn require_mut<T: Component>(&mut self) -> Result<Mut<'_, T>, EntityError> {
        self.world.require_mut::<T>(self.entity)
    }

    pub fn insert<B: Bundle>(&mut self, bundle: B) -> Result<(), EntityError> {
        self.world.insert(self.entity, bundle)
    }

    pub fn remove<B: Bundle>(&mut self) -> Result<B, EntityError> {
        self.world.remove::<B>(self.entity)
    }

    pub fn despawn(self) -> Result<(), EntityError> {
        self.world.despawn(self.entity)
    }
}

trait WorldCommand {
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError>;
}

impl<F> WorldCommand for F
where
    F: FnOnce(&mut World) -> Result<(), CommandError> + 'static,
{
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError> {
        (*self)(world)
    }
}

pub struct Commands {
    queue: CommandQueueStorage,
}

type CommandQueue = Vec<Box<dyn WorldCommand>>;

enum CommandQueueStorage {
    Owned(CommandQueue),
    Borrowed(NonNull<Commands>),
}

impl Commands {
    pub fn new() -> Self {
        Self {
            queue: CommandQueueStorage::Owned(Vec::new()),
        }
    }

    pub(crate) fn from_external(owner: *mut Commands) -> Self {
        Self {
            queue: CommandQueueStorage::Borrowed(
                NonNull::new(owner).expect("command owner pointer must not be null"),
            ),
        }
    }

    fn queue_mut(&mut self) -> &mut CommandQueue {
        match &mut self.queue {
            CommandQueueStorage::Owned(queue) => queue,
            // Safety: borrowed command owners come from a live `Commands` owner.
            CommandQueueStorage::Borrowed(owner) => unsafe { owner.as_mut().queue_mut() },
        }
    }

    fn into_queue(self) -> CommandQueue {
        match self.queue {
            CommandQueueStorage::Owned(queue) => queue,
            // Safety: borrowed command owners come from a live `Commands` owner.
            CommandQueueStorage::Borrowed(mut owner) => unsafe {
                std::mem::take(owner.as_mut().queue_mut())
            },
        }
    }

    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B) {
        self.queue_mut().push(Box::new(move |world: &mut World| {
            world.spawn(bundle);
            Ok(())
        }));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue_mut().push(Box::new(move |world: &mut World| {
            world.despawn(entity)?;
            Ok(())
        }));
    }

    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B) {
        self.queue_mut().push(Box::new(move |world: &mut World| {
            world.insert(entity, bundle)?;
            Ok(())
        }));
    }

    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity) {
        self.queue_mut().push(Box::new(move |world: &mut World| {
            let _: B = world.remove(entity)?;
            Ok(())
        }));
    }

    pub fn apply(self, world: &mut World) -> Result<(), CommandError> {
        for command in self.into_queue() {
            command.apply(world)?;
        }
        Ok(())
    }
}
