---
title: ID
description: Foundation typed identity primitives and allocators.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-04-30
---

# ID

Typed identity primitives and allocators for Runenwerk architecture boundaries.

This page is the canonical docs-site location for the `foundation/id` crate.

Crate-local package README text is still kept in `foundation/id/README.md` for Cargo package metadata.

## Public API

Use `TypedId<Tag>` when a domain needs a durable scalar ID whose type prevents
mixing unrelated identity families. Use `GenerationalId<Tag>` when a runtime
handle needs slot/generation stale-handle detection.

Use `MonotonicIdAllocator<Tag>` to issue non-zero `TypedId<Tag>` values. Use
`GenerationalIdAllocator<Tag>` when allocation/free/liveness and generation
invalidation are required.

`IdTag` names the identity family. The crate does not own payload storage,
lookup registries, ECS integration, database IDs, editor identity policy, or
global object identity.

## Invariants

- `TypedId<Tag>` raw values are non-zero.
- `TypedId::try_from_raw` and `TryFrom<u64>` reject zero.
- `MonotonicIdAllocator::try_new` rejects zero start values.
- `u64::MAX` is reserved as the monotonic allocator exhaustion sentinel.
- `GenerationalIdAllocator` invalidates stale generations after free/reuse.

## Example

```rust
use id::{IdTag, MonotonicIdAllocator, TypedId};

enum EntityTag {}

impl IdTag for EntityTag {
    const DEBUG_NAME: &'static str = "EntityId";
}

let mut ids = MonotonicIdAllocator::<EntityTag>::default();
let entity: TypedId<EntityTag> = ids.allocate();

assert_eq!(entity.raw(), 1);
```
