//! App domain.
//!
//! `App` is the runtime composition root used by engine users.
//! It owns plugin registration, resources, and run-mode selection.

mod domain;
mod platform;
mod runtime;

pub use domain::app::*;
pub(crate) use domain::lifecycle::AppLifecycle;
pub use domain::plugins::*;
pub use domain::runner::*;
pub(crate) use domain::state::WindowedAppState;
pub use platform::windowed::AppNativeHostExt;
