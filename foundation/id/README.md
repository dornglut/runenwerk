# Runenwerk ID

Typed identity primitives for Runenwerk.

## Mission

This crate provides narrow, reusable primitives:

- `TypedId<Tag>` for durable typed scalar IDs
- `GenerationalId<Tag>` for runtime handles with stale-handle invalidation
- `MonotonicIdAllocator<Tag>` for scalar ID allocation
- `GenerationalIdAllocator<Tag>` for slot-reusing generation-tracked handles

It does **not** provide registries, storage maps, ECS adapters, or authority systems.

## Identity Families

- Use `TypedId<Tag>` for durable/document/workflow/protocol identity.
- Use `GenerationalId<Tag>` for live runtime handles where stale invalidation matters.

## Features

- `std` (default): enables standard library support.
- `alloc`: enables allocator modules.
- `serde`: enables serde for value types.
- `legacy`: enables deprecated compatibility façade `TypedIdSequence<Tag>`.

## Stability Notes

- `TypedId<Tag>` is non-zero and rejects `0`.
- `GenerationalId<Tag>` uses stable packed layout:
  - low 32 bits = slot
  - high 32 bits = generation
- `MonotonicIdAllocator<Tag>` reserves `u64::MAX` as exhaustion sentinel.
- `GenerationalIdAllocator<Tag>` retires slots on generation overflow instead of wrapping.

## Migration Notes (2026-04-21)

Breaking changes in the strict canonical surface:

- `TypedId<Tag>` no longer implements `Default`.
- `TypedId<Tag>` is non-zero; raw `0` is invalid.
- `TypedId<Tag>` now uses checked construction for fallible paths:
  - use `TypedId::try_from_raw(...)` or `TryFrom<u64>`.
- `IdTag` is removed from the canonical surface.
- `TypedIdSequence<Tag>` moved to `legacy` feature and is deprecated.

Allocator API hardening:

- `MonotonicIdAllocator::new(start_at)` is fallible (`Result`).
- Canonical allocation APIs are `try_*`; panic wrappers remain as convenience.
- `GenerationalIdAllocator::try_free(...) -> Result<(), FreeError>` replaces bool-style failure signaling.

Workspace/dependency path updates in this repository:

- Workspace member moved from `domain/id` to `foundation/id`.
- Crate path dependencies now target `foundation/id`, for example:
  - `id = { path = "../foundation/id" }` (from `engine`)
  - `id = { path = "../../../foundation/id" }` (from `domain/editor/editor_shell`)
