use crate::query::QueryAccess;
use crate::{Commands, ResourceError, World};
use scheduler::system::ParamSlotDescriptor;
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

/// Framework-owned low-level system-parameter implementation contract.
///
/// Safe user composition uses the built-in parameters and
/// `#[derive(SystemParam)]`. Manual implementations are unsupported low-level
/// code because access metadata, raw extraction pointers, and cached state
/// participate directly in the runtime's aliasing and lifetime proof.
///
/// # Safety
///
/// Implementors must keep `State` lifetime-independent for every `'w`
/// implementation of the same parameter type, report every immediate borrow in
/// `access`, and only dereference `world` / `commands` according to those access
/// facts for the extraction lifetime.
pub unsafe trait SystemParam<'w>: Sized {
    type State: 'static;

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError>;
    fn access(state: &Self::State) -> QueryAccess;
    fn slot_descriptor() -> ParamSlotDescriptor {
        let type_name = std::any::type_name::<Self>();
        ParamSlotDescriptor::leaf("unknown", type_name, type_name)
    }

    /// # Safety
    ///
    /// `world` and `commands` must point to live values for `'w`. Implementors
    /// may only access World domains described by `Self::access(state)` and must
    /// preserve the scheduler-validated aliasing contract.
    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        commands: *mut Commands,
    ) -> Result<Self, SystemParamError>;
}
