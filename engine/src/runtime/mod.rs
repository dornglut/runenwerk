//! Runtime domain.
//!
//! Owns schedule labels, runtime system params, platform event normalization,
//! fixed-step state/resources, canonical fixed-step execution, canonical frame
//! lifecycle execution, and window runtime helpers.

pub(crate) mod fixed_step_executor;
pub mod fixed_time;
pub(crate) mod frame_lifecycle;
pub mod param;
pub mod platform;
pub mod product_publication;
pub mod query_snapshot;
pub mod schedules;
pub mod system;
pub mod window;
pub mod winit_runner;

pub use fixed_time::*;
pub use param::*;
pub use product_publication::*;
pub use query_snapshot::*;
pub use schedules::*;
pub use system::*;
pub use window::*;
