//! Runtime integration for the drawing app.

pub mod app;
pub mod ink;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use app::{build_app, build_headless_app, run};
pub use ink::{
    publish_drawing_ink_products, publish_drawing_ink_products_at_barrier,
    publish_drawing_ink_query_snapshots, publish_drawing_ink_query_snapshots_at_barrier,
};
pub use plugin::{DrawingAppPlugin, DrawingRuntimeSet};
pub use resources::DrawingHostResource;
pub use systems::DRAWING_UI_FRAME_PRODUCER_ID;
