# id

Typed identity primitives and allocators for Runenwerk architecture boundaries.

## Purpose

`id` owns reusable identity value types that can cross foundation, domain,
engine, app, and adapter boundaries without leaking the storage policy of any
one subsystem.

## Public Surface

- `TypedId<Tag>`: durable typed scalar identifier.
- `IdTag`: marker trait used to separate identity families at the type level.
- `MonotonicIdAllocator`: allocator for sequential typed ids.
- `GenerationalId<Tag>` and `GenerationalIdAllocator`: runtime handles with
  stale-handle invalidation.
- `AllocationError`, `FreeError`, and `InvalidRawId`: explicit error types for
  invalid allocation, freeing, and raw id conversion.

## Feature Policy

- `default = ["std"]`
- `std` enables standard-library support.
- `alloc` enables allocator-backed types in `no_std` contexts.
- `serde` enables serialization for supported identity types.

There is no `legacy` feature. Older migration-only identity helpers should be
removed or replaced by the current typed id and allocator APIs.

## Non-Scope

This crate does not own registries, slab storage, ECS integration, persistence
schemas, or domain-specific identity policies. Those belong to the owning
domain or app crate.
