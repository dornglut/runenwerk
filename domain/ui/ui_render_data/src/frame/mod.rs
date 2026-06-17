//! File: domain/ui/ui_render_data/src/frame/mod.rs
//! Purpose: UI frame-level render contracts.

pub mod composition;
pub mod layer_id;
pub mod surface_id;
pub mod ui_frame;
pub mod ui_layer;
pub mod ui_surface;

pub use composition::{UiFrameFragment, UiFramePlacement, compose_frame_fragments};
pub use layer_id::UiLayerId;
pub use surface_id::UiSurfaceId;
pub use ui_frame::UiFrame;
pub use ui_layer::UiLayer;
pub use ui_surface::UiSurface;
