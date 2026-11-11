//! Typed, closure-authored logical state transitions and optional lifetime finality.
//!
//! Domain owners define transition rules; this crate stores state, commits accepted
//! replacements, and optionally seals a genuinely terminal lifetime. It never
//! schedules work, executes callbacks, or owns external resource settlement.

mod lifecycle;
mod machine;
mod model;

pub use lifecycle::{Lifecycle, LifecycleError};
pub use machine::{StateMachine, TransitionOutcome};
pub use model::{InvalidTransition, Next};
