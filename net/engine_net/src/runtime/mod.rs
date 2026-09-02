//! Retained replication-runtime migration contracts.
//!
//! RunenNet owns connection/session lifecycle. This module remains only for the old replication
//! payload/runtime evidence that will be removed by the later replication cut.

pub mod events;

pub use events::*;
