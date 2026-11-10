//! Synchronous state storage and transition commitment without core-managed allocation.

use std::borrow::Borrow;

use crate::{Next, TransitionState};

/// Outcome of an accepted event; a domain may deliberately accept a no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Changed,
    Unchanged,
}

impl Transition {
    pub const fn changed(self) -> bool {
        matches!(self, Self::Changed)
    }
}

/// Stores a domain state and commits only fully evaluated transitions.
///
/// `StateMachine<S>` has no `Clone`, `Copy`, `Eq`, `Send` or `Sync`
/// requirement on `S`. Trait implementations on this wrapper remain
/// conditional on whether the domain state supports those operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine<S> {
    state: S,
}

impl<S: TransitionState> StateMachine<S> {
    /// Initial state must be admitted by its domain owner. This also supports
    /// domain-validated restoration without a privileged mutable-state setter.
    pub fn new(initial: S) -> Self {
        Self { state: initial }
    }

    pub const fn state(&self) -> &S {
        &self.state
    }

    pub fn into_state(self) -> S {
        self.state
    }

    /// Accept borrowed or owned events. Borrowing retains non-`Copy` event data
    /// when an event is rejected and allows callers to reuse the same event.
    pub fn transition(&mut self, event: impl Borrow<S::Event>) -> Result<Transition, S::Error> {
        let decision = self.state.next(event.borrow())?;
        match decision {
            Next::Stay => Ok(Transition::Unchanged),
            Next::Enter(next) => {
                self.state = next;
                Ok(Transition::Changed)
            }
        }
    }
}
