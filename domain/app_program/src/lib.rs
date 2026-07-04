//! Typed app-program contracts.
//!
//! This crate owns the local, UI-independent app-program proof vocabulary:
//! model snapshots, typed actions, route-action mapping, pure reducers,
//! inert effect plans, deterministic projections, replay traces, and reports.
//! UI crates are proving consumers through tests/examples only.

pub mod action;
pub mod counter;
pub mod effect;
pub mod ids;
pub mod model;
pub mod projection;
pub mod reducer;
pub mod replay;
pub mod report;
pub mod route_action;

pub use action::*;
pub use counter::*;
pub use effect::*;
pub use ids::*;
pub use model::*;
pub use projection::*;
pub use reducer::*;
pub use replay::*;
pub use report::*;
pub use route_action::*;
