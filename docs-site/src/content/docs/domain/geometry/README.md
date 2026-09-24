---
title: "Geometry Crate"
description: "Documentation for Geometry Crate."
status: active
owner: geometry
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# Geometry Crate

`geometry` is the shared `domain` crate for explicit geometric primitives,
bounds, and geometric queries built on `glam`.

## Purpose

This crate provides a stable geometry vocabulary for workspace consumers:

- bounds (`Aabb2`, `Aabb3`)
- primitive shapes (`Sphere`, `Plane`, `Frustum`, segments, triangles)
- containment checks
- overlap/intersection checks
- simple ray query helpers
- closest-point and classification helpers

## Ownership Boundary

Owns:

- explicit geometric primitives and bounds
- geometric containment/intersection helpers
- closest-point/classification helpers for owned primitives

Does not own:

- spatial indexing (BVH, octree, quadtree)
- chunk/LOD runtime structures
- physics/collision response
- SDF graph systems
- render feature logic
- ECS/world/entity concepts

## Why `glam`

`geometry` uses `glam` as the math substrate (`Vec2`, `Vec3`) to avoid
duplicating vector types and to stay aligned with workspace math usage.

## Current Modules

- `aabb`
- `ray`
- `sphere`
- `plane`
- `frustum`
- `segment`
- `triangle`
- `closest_point`
- `intersection`
- `classification`

## Examples

```rust
use geometry::{Aabb3, Ray3};
use geometry::intersection::ray_aabb3_first_hit;
use glam::Vec3;

let bounds = Aabb3::from_corners(Vec3::ZERO, Vec3::splat(2.0));
let ray = Ray3::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::X);
let hit = ray_aabb3_first_hit(&ray, &bounds);

assert_eq!(hit, Some(1.0));
```

```rust
use geometry::{Plane, Sphere};
use geometry::intersection::sphere_aabb3_intersects;
use geometry::Aabb3;
use glam::Vec3;

let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
let bounds = Aabb3::from_corners(Vec3::new(1.0, -0.5, -0.5), Vec3::new(2.0, 0.5, 0.5));
assert!(sphere_aabb3_intersects(&sphere, &bounds));

let ground = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
assert_eq!(ground.signed_distance(Vec3::new(0.0, 2.0, 0.0)), 2.0);
```

## Additional Docs

- [`implementation-roadmap.md`](implementation-roadmap.md)
- [`api-notes.md`](api-notes.md)
- [`ownership-boundary.md`](ownership-boundary.md)
