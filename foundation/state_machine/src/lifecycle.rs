//! Lifetime-specific terminality built on the general state machine.

use std::error::Error;
use std::fmt;

use crate::{StateMachine, Transition, TransitionModel};

/// A state machine whose owning domain declares terminal states.
pub trait LifecycleModel: TransitionModel {
    fn is_terminal(state: Self::State) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleError<E> {
    Terminal,
    Rejected(E),
}

impl<E: fmt::Display> fmt::Display for LifecycleError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Terminal => formatter.write_str("lifecycle is terminal"),
            Self::Rejected(error) => write!(formatter, "{error}"),
        }
    }
}

impl<E: Error + 'static> Error for LifecycleError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Terminal => None,
            Self::Rejected(error) => Some(error),
        }
    }
}

/// Enforces terminality without imposing universal phases or cleanup behavior.
pub struct Lifecycle<M: LifecycleModel> {
    machine: StateMachine<M>,
}

impl<M: LifecycleModel> Default for Lifecycle<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: LifecycleModel> Lifecycle<M> {
    pub fn new() -> Self {
        Self {
            machine: StateMachine::new(),
        }
    }

    pub fn state(&self) -> M::State {
        self.machine.state()
    }

    pub fn is_terminal(&self) -> bool {
        M::is_terminal(self.state())
    }

    pub fn transition(
        &mut self,
        event: M::Event,
    ) -> Result<Transition<M::State>, LifecycleError<M::Error>> {
        if self.is_terminal() {
            return Err(LifecycleError::Terminal);
        }
        self.machine
            .transition(event)
            .map_err(LifecycleError::Rejected)
    }
}

impl<M: LifecycleModel> fmt::Debug for Lifecycle<M>
where
    M::State: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Lifecycle")
            .field("state", &self.state())
            .finish()
    }
}
