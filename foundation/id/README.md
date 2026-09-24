# id

Typed identity primitives and allocators for Runenwerk architecture boundaries.

Canonical long-form documentation lives in:

- `docs-site/src/content/docs/foundation/id/README.md`

Public entry points:

- `TypedId<Tag>` for durable scalar IDs.
- `GenerationalId<Tag>` for packed slot/generation handle values.
- `MonotonicIdAllocator<Tag>` for issuing non-zero `TypedId<Tag>` values.
- `GenerationalIdAllocator<Tag>` for allocation/free/liveness of generational IDs.
- `IdTag` for naming an identity family at the type level.
