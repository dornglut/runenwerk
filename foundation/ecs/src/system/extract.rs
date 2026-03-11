use crate::query::QueryAccess;
use crate::{Commands, ResourceError, World};
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

    /// `State` must be lifetime-independent for all `'w` implementations of the same
    /// parameter type. Runtime state caching relies on this invariant.
    ///
    /// Safety: `world` and `commands` must point to live values for `'w`.
    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        commands: *mut Commands,
    ) -> Result<Self, SystemParamError>;
}
