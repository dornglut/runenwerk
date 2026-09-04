use crate::query::QueryAccess;
use crate::{Commands, ResourceError, World};
use scheduler::system::ParamSlotDescriptor;
use std::marker::PhantomData;
use std::ptr::NonNull;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemParamError {
    #[error(transparent)]
    Resource(#[from] ResourceError),
    #[error("invalid system param extraction for {param}: {reason}")]
    InvalidExtraction {
        param: &'static str,
        reason: &'static str,
    },
    #[error("runtime context error: {0}")]
    RuntimeContext(&'static str),
}

/// Invocation-scoped extraction context owned by the RunenECS runtime.
///
/// Safe system code never constructs this value. It is public only because the
/// low-level [`SystemParam`] implementation contract must remain reachable by
/// downstream derive expansion and the maintained engine-owned exclusive-world
/// parameter.
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct SystemParamContext<'world> {
    world: NonNull<World>,
    commands: NonNull<Commands>,
    _marker: PhantomData<&'world mut World>,
}

impl<'world> SystemParamContext<'world> {
    pub(crate) fn new(world: &'world mut World, commands: &'world mut Commands) -> Self {
        Self {
            world: NonNull::from(world),
            commands: NonNull::from(commands),
            _marker: PhantomData,
        }
    }

    pub(crate) const fn world_ptr(self) -> *mut World {
        self.world.as_ptr()
    }

    pub(crate) const fn commands_ptr(self) -> *mut Commands {
        self.commands.as_ptr()
    }

    /// Recovers the invocation's exclusive World borrow.
    ///
    /// # Safety
    ///
    /// The implementing parameter must be the sole immediate World-access
    /// authority for the invocation. Its access declaration must therefore be
    /// `QueryAccess::exclusive_world()`, and no sibling parameter may expose an
    /// immediate component/resource/messaging borrow. Ordinary framework-owned
    /// parameters must use the narrower extraction facilities added by C3
    /// instead of calling this method.
    pub unsafe fn world_mut(self) -> &'world mut World {
        unsafe { &mut *self.world.as_ptr() }
    }
}

/// Framework-owned low-level system-parameter implementation contract.
///
/// Safe user composition uses the built-in parameters and
/// `#[derive(SystemParam)]`. Manual implementations are unsupported low-level
/// code because access metadata and extracted items participate directly in the
/// runtime's aliasing proof.
///
/// # Safety
///
/// Implementors must report every immediate borrow in `access`, keep `State`
/// lifetime-independent, and ensure every `Item<'world, 'state>` is valid only
/// under the invocation World/state capabilities supplied to `extract`.
#[doc(hidden)]
pub unsafe trait SystemParam: Sized {
    type State: 'static;
    type Item<'world, 'state>;

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError>;
    fn access(state: &Self::State) -> QueryAccess;
    fn slot_descriptor() -> ParamSlotDescriptor {
        let type_name = std::any::type_name::<Self>();
        ParamSlotDescriptor::leaf("unknown", type_name, type_name)
    }

    /// # Safety
    ///
    /// `context` belongs to the current system invocation. Implementors may only
    /// access World domains described by `Self::access(state)`, may not extend
    /// references beyond the corresponding GAT lifetimes, and must preserve the
    /// scheduler-validated aliasing contract.
    unsafe fn extract<'world, 'state>(
        state: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError>;
}
