//! An author-facing, directly implemented transition contract.

/// Defines one domain's transitions without a separate marker/policy type.
///
/// `next` must be pure with respect to externally owned state. It receives a
/// borrow of the current state and event, and returns either a complete new
/// state, a deliberate no-op, or an error. No partial state is committed on
/// rejection or panic. Actual IO, rendering, ECS and cleanup remain with the
/// owning domain; successful evaluation does not make those effects atomic.
pub trait TransitionState: Sized {
    type Event;
    type Error;

    fn next(&self, event: &Self::Event) -> Result<Next<Self>, Self::Error>;
}

/// The complete logical state update proposed by a transition policy.
///
/// `Stay` makes an allowed, idempotent event explicit without requiring
/// cloning potentially large or non-`Copy` states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next<S> {
    Stay,
    Enter(S),
}
