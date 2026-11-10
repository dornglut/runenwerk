//! A single state store with one statically typed, author-supplied policy.

use std::borrow::Borrow;
use std::fmt;
use std::marker::PhantomData;

use crate::{Lifecycle, Next};

/// Result of a successfully evaluated event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionOutcome {
    /// The closure explicitly accepted the event without modifying state.
    Stayed,
    /// The closure proposed a replacement, now installed in the machine.
    /// `Replaced` does not require or imply unequal old and new values.
    Replaced,
}

impl TransitionOutcome {
    pub const fn replaced(self) -> bool {
        matches!(self, Self::Replaced)
    }
}

/// A state machine stores exactly one domain state and one transition closure.
///
/// `E` and `Err` describe the policy's event and error types, but the
/// policy is the executable authoring contract. They are phantom parameters
/// because neither event nor error is retained by the machine.
///
/// Use `StateMachine::new(initial).transitions(|state, event| ...)` to
/// construct a usable machine. The unconfigured value has no transition API.
/// No `Copy`, `Clone`, `Eq`, `Send` or `Sync` requirement is imposed on
/// state, event or error; storage/thread-safety follows the owned state/policy.
pub struct StateMachine<S, E = (), Err = (), Policy = ()> {
    state: S,
    policy: Policy,
    marker: PhantomData<fn(E) -> Err>,
}

impl<S> StateMachine<S> {
    /// Accepts a caller-supplied initial state without validating it.
    /// Restoration validation is the owning domain's responsibility.
    pub fn new(initial: S) -> Self {
        Self {
            state: initial,
            policy: (),
            marker: PhantomData,
        }
    }

    /// Supplies the *one* domain policy. A closure can capture immutable
    /// configuration and is stored inline with static dispatch.
    ///
    /// The policy must be callable through `Fn`; it is passed shared
    /// references to state and event and must avoid externally visible
    /// effects. `Fn` does not mechanically prohibit interior mutability.
    pub fn transitions<E, Err, Policy>(
        self,
        policy: Policy,
    ) -> StateMachine<S, E, Err, Policy>
    where
        Policy: Fn(&S, &E) -> Result<Next<S>, Err>,
    {
        StateMachine {
            state: self.state,
            policy,
            marker: PhantomData,
        }
    }
}

impl<S, E, Err, Policy> StateMachine<S, E, Err, Policy> {
    pub const fn state(&self) -> &S {
        &self.state
    }

    pub fn into_state(self) -> S {
        self.state
    }
}

impl<S, E, Err, Policy> StateMachine<S, E, Err, Policy>
where
    Policy: Fn(&S, &E) -> Result<Next<S>, Err>,
{
    /// Processes an owned or borrowed event.
    ///
    /// Rejection or panic in the policy leaves state unchanged. When accepted,
    /// `Enter` installs the new state before dropping the old state. A panic
    /// from the old state's destructor thus leaves the replacement committed.
    /// No external effects are rolled back.
    pub fn transition(
        &mut self,
        event: impl Borrow<E>,
    ) -> Result<TransitionOutcome, Err> {
        let decision = (self.policy)(&self.state, event.borrow())?;
        match decision {
            Next::Stay => Ok(TransitionOutcome::Stayed),
            Next::Enter(next) => {
                let old = std::mem::replace(&mut self.state, next);
                drop(old);
                Ok(TransitionOutcome::Replaced)
            }
        }
    }

    /// Fluent variant: earlier accepted transitions remain committed if a
    /// subsequent event is rejected. This is not a multi-event transaction.
    pub fn apply(&mut self, event: impl Borrow<E>) -> Result<&mut Self, Err> {
        self.transition(event)?;
        Ok(self)
    }

    /// Adds finality to the *same* transition policy and state store.
    /// Only use this when the domain can identify genuinely settled states.
    pub fn terminal_when<Terminal>(
        self,
        terminal: Terminal,
    ) -> Lifecycle<S, E, Err, Policy, Terminal>
    where
        Terminal: Fn(&S) -> bool,
    {
        Lifecycle::from_machine(self, terminal)
    }
}

impl<S: Clone, E, Err, Policy: Clone> Clone for StateMachine<S, E, Err, Policy> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            policy: self.policy.clone(),
            marker: PhantomData,
        }
    }
}

impl<S: fmt::Debug, E, Err, Policy> fmt::Debug
    for StateMachine<S, E, Err, Policy>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateMachine")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}
