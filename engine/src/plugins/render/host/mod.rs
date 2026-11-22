//! Runenwerk render host integration: native-window correlation, surface policy,
//! and attached public RunenGPU context/surface contracts.

mod gpu_context;
pub(crate) mod native;
// Kept internal until Native #1450 adopts the Render-owned retirement gate.
#[allow(dead_code)]
pub(crate) mod primary_retirement;
mod surface;

pub use gpu_context::*;
pub use surface::*;
