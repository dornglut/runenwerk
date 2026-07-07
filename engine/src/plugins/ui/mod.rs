//! Engine-owned UI runtime plugin foundation.
//!
//! This module establishes the plugin, resources, report, diagnostics, and
//! schedule labels that later UI runtime phases build on.

pub mod app_ext;
pub mod diagnostics;
pub mod mount;
pub mod plugin;
pub mod report;
pub mod resources;
pub mod schedule;

pub use app_ext::*;
pub use diagnostics::*;
pub use mount::*;
pub use plugin::UiPlugin;
pub use report::*;
pub use resources::*;
pub use schedule::*;
