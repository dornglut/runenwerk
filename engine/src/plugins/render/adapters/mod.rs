//! Temporary bridges from current Runenwerk rendering into RunenGPU contracts.
//!
//! Each adapter names its deletion phase. None of these modules is future
//! RunenGPU source or an alternate public GPU contract.

mod gpu_capabilities;
mod gpu_data;
mod gpu_resources;

pub use gpu_capabilities::*;
pub use gpu_data::*;
pub use gpu_resources::*;
