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
//!
//! # Semver Guarantees
//!
//! - Within `1.x`, existing public APIs are preserved unless explicitly marked
//!   as deprecated first.
//! - Existing semantic contracts for allocator exhaustion, generation
//!   invalidation, and liveness checks are part of the compatibility surface.
//! - Changes that alter serialized representation or documented handle
//!   semantics require a major version bump.
//!
//! # Serialization Stability
//!
//! - `TypedId<Tag>` serializes as a `u64` raw scalar and rejects `0` on
//!   deserialization.
//! - `GenerationalId<Tag>` serializes as the packed `u64` handle value using
//!   the stable layout:
//!   - low 32 bits: slot
//!   - high 32 bits: generation
//! - These wire formats are stable for all `1.x` releases.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod allocator;
pub mod error;
pub mod generational_id;
pub mod typed_id;

#[cfg(feature = "alloc")]
pub use allocator::{GenerationalAllocatorStats, GenerationalIdAllocator, MonotonicIdAllocator};
pub use error::{AllocationError, FreeError, InvalidRawId};
pub use generational_id::GenerationalId;
pub use typed_id::TypedId;
