#![cfg_attr(not(feature = "std"), no_std)]
//! Typed identity primitives for Runenwerk architecture boundaries.
//!
//! Identity families:
//! - [`TypedId<Tag>`] for durable typed scalar identifiers.
//! - [`GenerationalId<Tag>`] for runtime handles with stale-handle invalidation.
//!
//! This crate provides value types and allocator primitives only. It does not
//! provide storage registries, slab maps, ECS integration helpers, or policy
//! systems.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod allocator;
pub mod error;
#[cfg(feature = "legacy")]
pub mod legacy;
pub mod tag;
pub mod typed_id;

#[cfg(feature = "alloc")]
pub use allocator::{GenerationalId, GenerationalIdAllocator, MonotonicIdAllocator};
pub use error::{AllocationError, FreeError, InvalidRawId};
#[cfg(feature = "legacy")]
#[allow(deprecated)]
pub use legacy::TypedIdSequence;
pub use tag::IdTag;
pub use typed_id::TypedId;
