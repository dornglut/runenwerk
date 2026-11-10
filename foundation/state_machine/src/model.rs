//! Decisions made by an author-supplied transition closure.

use std::error::Error;
use std::fmt;

/// The complete logical decision proposed by the transition closure.
///
/// `Stay` explicitly accepts an event without copying or replacing state.
/// `Enter` proposes a new state, even when that value compares equal to the
/// previous state. The mechanism never requires `S: Copy + Clone + Eq`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum Next<S> {
    Stay,
    Enter(S),
}

/// Default error for simple models that do not need domain-specific diagnostics.
/// A custom error type remains an equally supported closure result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition;

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid state transition")
    }
}

impl Error for InvalidTransition {}
