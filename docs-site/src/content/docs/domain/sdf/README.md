---
title: "SDF Crate"
description: "Documentation for SDF Crate."
status: active
owner: sdf
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# SDF Crate

`sdf` is the shared `domain` crate for signed-distance-field primitives,
composition, transforms, sampling, gradients/normals, and core SDF queries.

## Purpose

This crate provides a stable SDF vocabulary for workspace consumers:

- analytic SDF primitives
- composable field operations
- transform wrappers
- sampling contracts
- gradient and normal helpers
- core field queries (raymarch, projection, classification, sweep foundations)

## Ownership Boundary

Owns:

- SDF field contracts (`SdfField3`, `SdfSample`)
- primitive SDFs
- SDF composition/transforms
- field-local conservative bounds
- query helpers for point/ray/surface interactions

Does not own:

- ECS integration
- render pass orchestration or shader compilation
- scene graph ownership
- chunk residency/clipmaps/streaming runtime systems
- gameplay controllers and rigid-body solvers

## Sign Convention

- negative distance: inside
- zero distance: on surface
- positive distance: outside

## Relation to `domain/geometry`

`geometry` owns explicit primitives and explicit intersections (`Aabb3`, `Ray3`,
`Sphere`, `Plane`, etc). `sdf` owns implicit fields and signed-distance queries.

## Relation to future `domain/spatial`

`spatial` should own indexing/runtime organization concerns (BVH, chunking,
clipmaps, LOD helpers). `sdf` remains focused on field math and field queries.

## Quick Example

```rust
use glam::Vec3;
use sdf::ops::Union;
use sdf::primitives::{SdfBox3, SdfSphere};
use sdf::queries::raymarch::{raymarch_first_hit, RayHit};
use sdf::{SdfField3, SdfSample};
use geometry::Ray3;

let sphere = SdfSphere::new(Vec3::ZERO, 1.0);
let box3 = SdfBox3::new(Vec3::new(1.25, 0.0, 0.0), Vec3::splat(0.6));
let field = Union::new(sphere, box3);

let sample: SdfSample = field.sample(Vec3::new(0.0, 0.0, 0.0));
assert!(sample.distance < 0.0);

let ray = Ray3::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::X);
let hit: Option<RayHit> = raymarch_first_hit(&field, &ray, 128, 10.0, 1e-3);
assert!(hit.is_some());
```

## Documentation

- `docs/index.md`
- `docs/implementation-roadmap.md`
- `docs/ownership-boundary.md`
- `docs/api-notes.md`
- `docs/QUERY_MODEL.md`
- `docs/NUMERICS.md`
