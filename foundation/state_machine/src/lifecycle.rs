//! Terminal guard around the same author-supplied transition closure.

use std::borrow::Borrow;
use std::error::Error;
use std::fmt;

use crate::{StateMachine, TransitionOutcome};

/// Distinguishes a terminal lifetime from a domain-specific rejection.
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

/// A state machine with an additional finality predicate.
///
/// Construct with
/// `StateMachine::new(initial).transitions(|state, event| ...).terminal_when(|state| ...)`.
/// Terminality is a *domain-owned* fact, not failure, non-runnability, or
/// initiation of shutdown. No scheduler or cleanup effects are implied.
///
/// Deliberately not `Clone`: cloning a live lifetime would create
/// independent authoritative transition histories.
pub struct Lifecycle<S, E, Err, Policy, Terminal> {
    machine: StateMachine<S, E, Err, Policy>,
    terminal: Terminal,
}

impl<S, E, Err, Policy, Terminal> Lifecycle<S, E, Err, Policy, Terminal> {
    pub(crate) fn from_machine(
        machine: StateMachine<S, E, Err, Policy>,
        terminal: Terminal,
    ) -> Self {
        Self { machine, terminal }
    }

    pub const fn state(&self) -> &S {
        self.machine.state()
    }

    pub fn into_state(self) -> S {
        self.machine.into_state()
    }
}

impl<S, E, Err, Policy, Terminal> Lifecycle<S, E, Err, Policy, Terminal>
where
    Policy: Fn(&S, &E) -> Result<crate::Next<S>, Err>,
    Terminal: Fn(&S) -> bool,
{
    pub fn is_terminal(&self) -> bool {
        (self.terminal)(self.state())
    }

    pub fn transition(
        &mut self,
        event: impl Borrow<E>,
    ) -> Result<TransitionOutcome, LifecycleError<Err>> {
        if self.is_terminal() {
            return Err(LifecycleError::Terminal);
        }
        self.machine
            .transition(event)
            .map_err(LifecycleError::Rejected)
    }

    pub fn apply(&mut self, event: impl Borrow<E>) -> Result<&mut Self, LifecycleError<Err>> {
        self.transition(event)?;
        Ok(self)
    }
}

impl<S: fmt::Debug, E, Err, Policy, Terminal> fmt::Debug
    for Lifecycle<S, E, Err, Policy, Terminal>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lifecycle")
            .field("state", self.machine.state())
            .finish_non_exhaustive()
    }
}
