//! Allocator primitives for typed identities.
//!
//! This module is allocator-only infrastructure. It does not own payload
//! storage, registries, or lookup indexes.

pub mod generational;
pub mod monotonic;

pub use generational::{GenerationalAllocatorStats, GenerationalIdAllocator};
pub use monotonic::MonotonicIdAllocator;
