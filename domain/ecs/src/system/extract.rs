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

pub trait SystemParam<'w>: Sized {
    type State: 'static;

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError>;
    fn access(state: &Self::State) -> QueryAccess;
    fn slot_descriptor() -> ParamSlotDescriptor {
        let type_name = std::any::type_name::<Self>();
        ParamSlotDescriptor {
            kind: "unknown",
            label: type_name,
            type_name,
        }
    }

    /// `State` must be lifetime-independent for all `'w` implementations of the same
    /// parameter type. Runtime state caching relies on this invariant.
    ///
    /// # Safety
    ///
    /// `world` and `commands` must point to live values for `'w`. Implementors must only
    /// access world data described by `Self::access(state)` and must preserve the aliasing
    /// guarantees encoded by the scheduler access model.
    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        commands: *mut Commands,
    ) -> Result<Self, SystemParamError>;
}
