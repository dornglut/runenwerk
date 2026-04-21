//! Allocator primitives for tag-typed identifiers.
//!
//! This module is allocator-only infrastructure. It does not own payload
//! storage, registries, or lookup indexes.

pub mod generational;
pub mod monotonic;

pub use generational::{GenerationalId, GenerationalIdAllocator};
pub use monotonic::MonotonicIdAllocator;
