---
title: API
description: API semantics and invariants for domain geometry.
---

# API Notes

This document records semantics and invariants for `domain/geometry`.

## Constructor Policy

- Strict constructors keep caller intent and do not reorder input:
  - `Aabb2::new(min, max)`
  - `Aabb3::new(min, max)`
- Normalizing constructors are explicitly named:
  - `Aabb2::from_corners(a, b)`
  - `Aabb3::from_corners(a, b)`

## Min/Max Ordering

- `new(min, max)` does not normalize.
- `is_valid()` reports whether min/max are ordered and finite.
- Methods like `contains_*` and `intersects` assume meaningful ordered bounds.

## Boundary Semantics

- Boundary-touching counts as intersecting for overlap checks.
- Containment checks are inclusive on boundaries.

## Rays and Direction Normalization

- `Ray2`/`Ray3` do not enforce normalized direction.
- Intersection helpers are implemented for arbitrary non-zero directions.
- Hit distances are ray-parameter values (`origin + direction * t`), not world distance unless direction is unit length.
- `ray_*_first_hit` returns `Some(0.0)` when the ray origin starts inside/on the primitive.

## Plane Convention

- Plane equation: `dot(normal, point) + distance = 0`.
- `Plane::signed_distance` returns metric distance only when `normal` is unit length.
- `Plane::project_point` handles non-normalized normals by dividing by `|normal|^2`.

## Frustum Convention

- Frustum planes are expected to have outward-facing normals.
- A point is inside when every plane signed distance is `<= 0`.
- `Frustum::intersects_aabb` and `Frustum::intersects_sphere` follow the same convention.

## Numeric Policy

- Public geometry APIs are `f32`-based.
- No mixed precision or generic numeric abstractions in the initial surface.

## Genericity Policy

- Prefer concrete explicit types:
  - `Aabb2`, `Aabb3`
  - `Ray2`, `Ray3`
  - `Triangle2`, `Triangle3`
- Avoid early abstraction-heavy generic geometry frameworks.
