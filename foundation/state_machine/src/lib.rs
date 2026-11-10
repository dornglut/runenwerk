//! Stateless typed transition policies, stored state, and lifetime terminality.
//!
//! Domain owners decide which events are meaningful and when external work has
//! actually completed. No scheduler, callback dispatcher or runtime graph.

mod lifecycle;
mod machine;
mod model;

pub use lifecycle::{Lifecycle, LifecycleError, LifecycleState};
pub use machine::{StateMachine, Transition};
pub use model::{Next, TransitionState};
