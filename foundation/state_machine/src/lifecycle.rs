//! Terminal lifetime specialization; delegates transitions to StateMachine.

use std::borrow::Borrow;
use std::error::Error;
use std::fmt;

use crate::{StateMachine, Transition, TransitionState};

/// Adds genuine lifetime finality to a domain transition state.
///
/// A state may be non-runnable, failed, closing, or suspended without being
/// terminal. Only return `true` when that domain has actually reached a
/// state from which *no* further transitions are permitted.
pub trait LifecycleState: TransitionState {
    fn is_terminal(&self) -> bool;
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

/// A lifecycle uses the *same* transition storage and commit mechanism.
/// Its only additional responsibility is to reject events after finality.
///
/// This does not coordinate cleanup, shutdown tickets, workers or the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle<S> {
    machine: StateMachine<S>,
}

impl<S: LifecycleState> Lifecycle<S> {
    pub fn new(initial: S) -> Self {
        Self {
            machine: StateMachine::new(initial),
        }
    }

    pub const fn state(&self) -> &S {
        self.machine.state()
    }

    pub fn is_terminal(&self) -> bool {
        self.state().is_terminal()
    }

    pub fn into_state(self) -> S {
        self.machine.into_state()
    }

    pub fn transition(
        &mut self,
        event: impl Borrow<S::Event>,
    ) -> Result<Transition, LifecycleError<S::Error>> {
        if self.is_terminal() {
            return Err(LifecycleError::Terminal);
        }
        self.machine
            .transition(event)
            .map_err(LifecycleError::Rejected)
    }
}
