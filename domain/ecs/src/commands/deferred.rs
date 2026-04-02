use crate::errors::CommandError;
use crate::world::World;
use std::marker::PhantomData;

pub trait DeferredCommand<T>: 'static {
    fn apply(self: Box<Self>, world: &mut World) -> Result<T, CommandError>;
}

pub(crate) trait ErasedDeferredCommand {
    fn apply_erased(self: Box<Self>, world: &mut World) -> Result<(), CommandError>;
}

pub(crate) struct DeferredCommandAdapter<T, C>
where
    C: DeferredCommand<T>,
{
    command: C,
    _marker: PhantomData<fn() -> T>,
}

impl<T, C> DeferredCommandAdapter<T, C>
where
    C: DeferredCommand<T>,
{
    pub(crate) fn new(command: C) -> Self {
        Self {
            command,
            _marker: PhantomData,
        }
    }
}

impl<T, C> ErasedDeferredCommand for DeferredCommandAdapter<T, C>
where
    C: DeferredCommand<T>,
{
    fn apply_erased(self: Box<Self>, world: &mut World) -> Result<(), CommandError> {
        let Self { command, .. } = *self;
        let _ = Box::new(command).apply(world)?;
        Ok(())
    }
}

impl<F> DeferredCommand<()> for F
where
    F: FnOnce(&mut World) -> Result<(), CommandError> + 'static,
{
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError> {
        (*self)(world)
    }
}
