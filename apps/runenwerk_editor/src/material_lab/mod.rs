//! App-owned Material Lab workflow.
//!
//! Domain crates own material semantics. This module owns project paths,
//! source document IO, preview-product orchestration, catalog publication, and
//! renderer prepared-data handoff.

pub mod default_material;
pub mod document_io;
pub mod generated_artifacts;
pub mod identity;
pub(crate) mod model_mesh_regions;
pub mod preview_scene_product;
pub mod preview_surface;
pub mod publication;
pub mod recipes;
pub mod renderer_handoff;
pub mod resource_resolution;
pub mod state;
pub mod tool_suite;
pub mod workflow;

pub use default_material::*;
pub use document_io::*;
pub use generated_artifacts::*;
pub use identity::*;
pub use preview_scene_product::*;
pub use preview_surface::*;
pub use publication::*;
pub use recipes::*;
pub use renderer_handoff::*;
pub use resource_resolution::*;
pub use state::*;
pub use tool_suite::material_lab_tool_suite;
pub use workflow::*;
