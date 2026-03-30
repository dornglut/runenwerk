//! Engine plugin registry and default plugin stack helpers.
//!
//! See `engine/src/plugins/README.md` for plugin geometry-old/navigation.

pub mod debug_metrics;
pub mod fixed_step;
pub mod grid;
pub mod input;
pub mod net;
pub mod render;
pub mod replay;
pub mod scene;
pub mod scheduler_diagnostics;
pub(crate) mod shared;
pub mod time;
pub mod ui;
pub mod world;

pub use debug_metrics::*;
pub use fixed_step::*;
pub use grid::*;
pub use input::*;
pub use render::*;
pub use replay::*;
pub use scene::*;
pub use scheduler_diagnostics::*;
pub use time::*;
pub use ui::*;
pub use world::*;

use crate::plugin::Plugin;

pub fn default_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(TimePlugin),
        Box::new(FixedStepPlugin),
        Box::new(ReplayPlugin),
        Box::new(InputFinalizePlugin),
    ]
}

pub fn default_plugins_with_diagnostics() -> Vec<Box<dyn Plugin>> {
    let mut plugins = default_plugins();
    plugins.push(Box::new(SchedulerDiagnosticsPlugin));
    plugins
}
