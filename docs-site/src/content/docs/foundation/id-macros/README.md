---
title: ID Macros
description: Foundation proc-macro support for typed ID wrappers.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-05
---

# ID Macros

`foundation/id_macros` provides the `#[id]` attribute for declaring typed ID
newtypes backed by `foundation/id`.

The macro is convenience vocabulary only. It does not create registries,
storage, ECS bindings, editor policy, runtime policy, or global identity.

## Public API

Apply `#[id]` to a unit struct:

```rust
use id_macros::id;

#[id]
pub struct EntityId;
```

The generated type exposes:

- `EntityId::try_from_raw(raw)` and `TryFrom<u64>` for raw values;
- `EntityId::raw()` for serialization or interop;
- conversions to and from the underlying `id::TypedId<Tag>`;
- `EntityIdSequence` as a `MonotonicIdAllocator<Tag>` alias.

## Invariants

- The macro accepts only unit structs.
- The macro injects derives itself; callers should not add a direct
  `#[derive(...)]` to the annotated struct.
- Generated IDs preserve the same non-zero raw-value invariant as
  `id::TypedId`.
- Generated IDs do not implement `From<u64>` and do not expose an infallible raw
  `u64` constructor.
