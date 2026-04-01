use super::deferred::{DeferredCommand, DeferredCommandAdapter};
use super::queue::CommandQueue;
use crate::bundle::Bundle;
use crate::entity::Entity;
use crate::errors::CommandError;
use crate::world::World;

pub struct BatchCommands {
    queue: CommandQueue,
}

impl BatchCommands {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    fn queue_typed<T: 'static, C>(&mut self, command: C)
    where
        C: DeferredCommand<T>,
    {
        self.queue
            .push(Box::new(DeferredCommandAdapter::new(command)));
    }

    pub fn defer<T: 'static, C>(&mut self, command: C)
    where
        C: DeferredCommand<T>,
    {
        self.queue_typed(command);
    }

    pub fn queue<F>(&mut self, command: F)
    where
        F: FnOnce(&mut World) -> Result<(), CommandError> + 'static,
    {
        self.queue_typed(command);
    }

    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B) {
        self.queue(move |world: &mut World| {
            world.spawn(bundle);
            Ok(())
        });
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue(move |world: &mut World| {
            world.despawn(entity)?;
            Ok(())
        });
    }

    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B) {
        self.queue(move |world: &mut World| {
            world.insert(entity, bundle)?;
            Ok(())
        });
    }

    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity) {
        self.queue(move |world: &mut World| {
            let _: B = world.remove(entity)?;
            Ok(())
        });
    }

    pub fn apply(self, world: &mut World) -> Result<(), CommandError> {
        for command in self.queue {
            command.apply_erased(world)?;
        }
        Ok(())
    }
}

impl Default for BatchCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl DeferredCommand<()> for BatchCommands {
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError> {
        (*self).apply(world)
    }
}
