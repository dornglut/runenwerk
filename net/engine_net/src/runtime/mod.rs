//! Runtime-facing network contracts.
//!
//! This module is intentionally small in this migration step. Concrete runtime
//! behavior remains in host adapters while contracts stay in `engine_net`.

pub mod client;
pub mod events;
pub mod server;
