//! A single typed state machine instance.

use std::marker::PhantomData;

use crate::TransitionModel;

/// Accepted transition, including explicitly allowed idempotent transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition<S> {
    pub from: S,
    pub to: S,
}

impl<S: Eq> Transition<S> {
    pub fn changed(&self) -> bool {
        self.from != self.to
    }
}

/// Stores state; it is not a scheduler, event queue or transaction manager.
#[derive(Debug)]
pub struct StateMachine<M: TransitionModel> {
    state: M::State,
    model: PhantomData<fn() -> M>,
}

impl<M: TransitionModel> Default for StateMachine<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: TransitionModel> StateMachine<M> {
    pub fn new() -> Self {
        Self {
            state: M::initial(),
            model: PhantomData,
        }
    }

    pub fn state(&self) -> M::State {
        self.state
    }

    /// A rejection or an unwind in the reducer leaves this state unchanged.
    /// No external side effects are rolled back by this method.
    pub fn transition(
        &mut self,
        event: M::Event,
    ) -> Result<Transition<M::State>, M::Error> {
        let from = self.state;
        let to = M::next(from, &event)?;
        self.state = to;
        Ok(Transition { from, to })
    }
}
