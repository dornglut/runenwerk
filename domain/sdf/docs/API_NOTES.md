# API Notes

Semantics and invariants for `foundation/sdf`.

## Core Contracts

- `SdfField3::sample(point)` returns `SdfSample`.
- Distances follow the crate-wide sign convention:
  - negative = inside
  - zero = surface
  - positive = outside
- `SdfField3::bounds()` returns `FieldBounds` and must be conservative.

## Bounds Policy

- If a field can provide a conservative `Aabb3`, return `FieldBounds::Bounded`.
- If it cannot, return `FieldBounds::Unbounded`.
- Conservative means the field's interior/surface is fully contained by the
  returned bound.

## Gradient and Normal Policy

- Finite differences are the baseline implementation.
- Epsilon is centralized in `epsilon.rs`.
- Helpers use defensive fallbacks when gradient magnitude is too small.

## Query Contracts

- Raymarch queries are sphere-tracing style and rely on field behavior being
  close to a signed distance metric in sampled regions.
- Projection iterates using local normal and signed distance until convergence
  or termination.
- Classification is epsilon-aware and intentionally explicit.

## Ergonomics Policy

- Prefer explicit concrete wrappers (`Union`, `Translate`, `SdfSphere`) over
  graph-heavy abstractions in the initial crate surface.
