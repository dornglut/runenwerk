//! Pure, typed transition policy.

/// A domain-owned rule for evaluating one transition.
///
/// Implementations must not mutate external state in `next`. Owning domains
/// perform real work before reporting an established transition fact.
pub trait TransitionModel {
    type State: Copy + Eq;
    type Event;
    type Error;

    fn initial() -> Self::State;

    fn next(from: Self::State, event: &Self::Event) -> Result<Self::State, Self::Error>;
}
