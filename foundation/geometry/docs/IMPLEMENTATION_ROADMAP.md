# Foundation Geometry Implementation Roadmap

Status: target roadmap  
Scope: long-term, high-quality implementation plan for `foundation/geometry`  
Audience: engine/runtime/render/tooling developers  
Math substrate: `glam`

## Purpose

`foundation/geometry` is the shared crate for explicit geometric primitives,
bounds, and geometric queries.

It should provide a stable vocabulary for:

- bounds
- primitive shapes
- containment tests
- overlap/intersection tests
- simple ray queries
- reusable geometric helpers

It should not become:

- a general math crate
- a spatial indexing crate
- an SDF crate
- a physics resolution crate
- a render feature crate
- an ECS-aware crate

## Target Outcomes

At completion, the crate should provide:

- explicit `glam`-backed geometry types
- documented invariants and edge semantics
- predictable reusable APIs
- strong edge-case test coverage
- clear `geometry` vs `spatial` boundary
- no engine/render/game-specific dependencies

## Ownership Summary

Owns:

- bounds (`Aabb2`, `Aabb3`)
- primitives (`Ray2`, `Ray3`, `Sphere`, `Plane`, `Frustum`)
- optional supporting primitives (`LineSegment2/3`, `Triangle2/3`)
- intersection/containment helpers
- closest-point/classification helpers

Does not own:

- BVH/octree/quadtree
- clipmaps/chunk/LOD runtime systems
- navigation and physics resolution
- render feature systems
- ECS/world/entity types

## Dependency Policy

Required:

- `glam`

Avoid initially:

- `serde`
- engine/runtime/render/ECS dependencies
- abstraction-heavy dependency stacks

## Public API Direction

`src/lib.rs` should expose explicit modules and concrete re-exports:

- `Aabb2`, `Aabb3`
- `Ray2`, `Ray3`
- `Sphere`
- `Plane`
- `Frustum`
- optional segments and triangles

API principles:

- explicit concrete type names
- inherent methods where ownership is obvious
- free functions for cross-type relations
- compact, predictable public surface
- avoid premature heavy generic abstractions

## Phases

1. Planning and boundary lock  
2. Crate scaffold (`Cargo.toml`, `README`, `lib.rs`, minimal modules/tests)  
3. AABB completion and hardening  
4. Ray support and ray/AABB intersections  
5. Sphere support and sphere intersections  
6. Plane support (signed distance and projection)  
7. Frustum support (point/AABB tests)  
8. Optional segments/triangles when real users need them  
9. Optional closest-point/classification helpers  
10. API polish and docs closeout  
11. Adoption and regression-driven stabilization

## Testing Roadmap

Required test areas:

- AABB constructors, invariants, containment, overlap, union/expansion
- ray queries (hit/miss/inside/grazing/reverse direction)
- sphere containment/overlap and AABB conversion
- plane signed distance/projection
- frustum inside/outside/intersecting behavior
- cross-type intersections and boundary-touch semantics
- regression tests for future discovered edge cases

## Semantics Rules

- Boundary touching counts as intersection.
- Strict constructors preserve input ordering.
- Normalizing constructors use explicit names.
- Use `f32` consistently.
- Keep invariants documented in public rustdoc/docs.

## Milestones

- M1: minimum useful geometry crate (`Aabb2`, `Aabb3`, tests/docs)
- M2: query-ready crate (rays + spheres + core intersections)
- M3: view/culling-ready crate (planes + frusta)
- M4: mature reusable crate (helpers, docs polish, adoption, regression hardening)

## Exit Criteria

The roadmap is complete when:

- `foundation/geometry` is the clear shared home for explicit geometry primitives
- boundaries with `foundation/spatial` are stable and documented
- core primitives and queries are robust and trusted by tests
- no engine/render/game-specific assumptions leak into the crate
