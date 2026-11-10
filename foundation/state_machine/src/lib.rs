//! Typed, synchronous state transitions and optional lifecycle terminality.
//!
//! No callbacks, scheduler, resource acquisition or ECS semantics.

mod lifecycle;
mod machine;
mod model;

pub use lifecycle::{Lifecycle, LifecycleError, LifecycleModel};
pub use machine::{StateMachine, Transition};
pub use model::TransitionModel;
