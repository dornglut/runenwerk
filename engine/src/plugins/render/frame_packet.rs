#[deprecated(
    note = "compatibility shim only; active runtime uses crate::plugins::render::frame::*"
)]
// Compatibility shim. Active runtime uses `crate::plugins::render::frame` types.
pub use crate::plugins::render::frame::*;
