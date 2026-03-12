# Ownership Boundary

This document defines what `foundation/geometry` owns and what belongs in
other domains/crates.

## Geometry vs Spatial

`foundation/geometry` owns:

- explicit primitive/bounds data structures
- geometric relationships and queries between those primitives

`foundation/spatial` (future) should own:

- spatial indexing (BVH, octree, quadtree)
- runtime broad-phase culling/indexing strategies
- chunk/region lookup acceleration structures

Rule of thumb:
- If it is a geometric primitive/query, it belongs in `geometry`.
- If it is an indexing/runtime partition structure, it belongs in `spatial`.

## Geometry vs SDF

`geometry` does not own SDF graph systems, signed-distance field runtime
evaluation pipelines, or render-facing material graph logic.

Simple shape helpers may exist in `geometry`, but SDF systems stay in their
own owning domain/crate.

## Geometry vs Engine Runtime

`geometry` does not own:

- engine plugin wiring
- frame scheduling/orchestration
- scene/camera runtime state models
- ECS/world/entity lifecycle concerns

Engine/runtime systems may consume `geometry` primitives, but `geometry`
must stay engine-agnostic.

## Geometry vs Render Features

`geometry` may provide reusable geometric tests that render features consume,
but it does not own render feature behavior, render graph integration, shader
logic, or backend-specific rendering contracts.

## Geometry vs Game-Specific Systems

`geometry` should not encode game rules or game-specific semantic layers.
Game domains may wrap or compose geometry primitives as needed.
