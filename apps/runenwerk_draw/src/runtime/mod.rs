//! Runtime integration for the drawing app.

pub mod app;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use app::{build_app, build_headless_app, run};
pub use plugin::{DrawingAppPlugin, DrawingRuntimeSet};
pub use resources::DrawingHostResource;
pub use systems::DRAWING_UI_FRAME_PRODUCER_ID;
