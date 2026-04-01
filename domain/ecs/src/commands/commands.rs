use super::batch::BatchCommands;
use super::deferred::{DeferredCommand, DeferredCommandAdapter};
use super::queue::CommandQueue;
use crate::bundle::Bundle;
use crate::entity::Entity;
use crate::errors::CommandError;
use crate::world::World;
use std::cell::{Cell, RefCell};
use std::ptr::NonNull;
use std::rc::Rc;

pub struct Commands {
    queue: CommandQueueStorage,
}

enum CommandQueueStorage {
    Owned(CommandQueue),
    ExternalOwner(ExternalCommandQueue),
    ExternalBorrowed(ExternalCommandQueue),
}

#[derive(Clone)]
struct ExternalCommandQueue {
    queue: Rc<RefCell<CommandQueue>>,
    active: Rc<Cell<bool>>,
}

impl ExternalCommandQueue {
    fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(Vec::new())),
            active: Rc::new(Cell::new(true)),
        }
    }

    fn assert_active(&self) {
        assert!(
            self.active.get(),
            "commands param escaped its system execution scope"
        );
    }

    fn drain(&self) -> CommandQueue {
        std::mem::take(&mut *self.queue.borrow_mut())
    }
}

impl Commands {
    pub fn new() -> Self {
        Self {
            queue: CommandQueueStorage::Owned(Vec::new()),
        }
    }

    pub(crate) fn new_external_owner() -> Self {
        Self {
            queue: CommandQueueStorage::ExternalOwner(ExternalCommandQueue::new()),
        }
    }

    pub(crate) fn from_external(owner: *mut Commands) -> Self {
        let owner = NonNull::new(owner).expect("command owner pointer must not be null");
        // Safety: owner pointers are only provided by runtime command owner construction.
        let queue = unsafe {
            owner
                .as_ref()
                .external_queue()
                .expect("command owner must provide an external queue")
        };
        Self {
            queue: CommandQueueStorage::ExternalBorrowed(queue),
        }
    }

    fn external_queue(&self) -> Option<ExternalCommandQueue> {
        match &self.queue {
            CommandQueueStorage::ExternalOwner(queue)
            | CommandQueueStorage::ExternalBorrowed(queue) => Some(queue.clone()),
            CommandQueueStorage::Owned(_) => None,
        }
    }

    fn push_erased(&mut self, command: Box<dyn super::deferred::ErasedDeferredCommand>) {
        match &mut self.queue {
            CommandQueueStorage::Owned(queue) => queue.push(command),
            CommandQueueStorage::ExternalOwner(queue) => queue.queue.borrow_mut().push(command),
            CommandQueueStorage::ExternalBorrowed(queue) => {
                queue.assert_active();
                queue.queue.borrow_mut().push(command);
            }
        }
    }

    fn into_queue(self) -> CommandQueue {
        match self.queue {
            CommandQueueStorage::Owned(queue) => queue,
            CommandQueueStorage::ExternalOwner(queue) => queue.drain(),
            CommandQueueStorage::ExternalBorrowed(queue) => {
                queue.assert_active();
                queue.drain()
            }
        }
    }

    pub(crate) fn finalize_external_owner(&mut self) -> Commands {
        let CommandQueueStorage::ExternalOwner(queue) = &self.queue else {
            panic!("external command owner finalization requires runtime command owner");
        };
        queue.active.set(false);
        let staged_queue = queue.drain();
        Commands {
            queue: CommandQueueStorage::Owned(staged_queue),
        }
    }

    fn queue_typed<T: 'static, C>(&mut self, command: C)
    where
        C: DeferredCommand<T>,
    {
        self.push_erased(Box::new(DeferredCommandAdapter::new(command)));
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

    pub fn batch<F>(&mut self, build: F)
    where
        F: FnOnce(&mut BatchCommands),
    {
        let mut batch = BatchCommands::new();
        build(&mut batch);
        self.defer(batch);
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
        for command in self.into_queue() {
            command.apply_erased(world)?;
        }
        Ok(())
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}
