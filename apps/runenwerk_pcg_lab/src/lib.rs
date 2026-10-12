//! Source-backed headless PCG Lab model.
//!
//! This first application slice owns only inspection state and derived
//! topology diagram coordinates. Native RunenUI/Render/Present remains gated
//! separately; there is deliberately no second procedural generator.

mod session;
mod topology_view;

pub use session::{CaveInspectionAttempt, CaveInspectionSession};
pub use topology_view::{
    CaveSelection, CaveTopologyView, CaveViewError, ChamberView, PassageView, ViewPoint,
    project_cave_topology,
};

#[cfg(test)]
mod tests;
