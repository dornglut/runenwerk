//! Runenwerk render host integration: native-window correlation, surface policy,
//! and attached public RunenGPU context/surface contracts.

mod gpu_context;
pub(crate) mod native;
mod surface;

pub use gpu_context::*;
pub use surface::*;
