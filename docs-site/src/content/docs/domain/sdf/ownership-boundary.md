---
title: "Ownership Boundary"
description: "Documentation for Ownership Boundary."
status: active
owner: sdf
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# Ownership Boundary

This document defines what `domain/sdf` owns and what belongs in other
workspace domains.

## SDF vs Geometry

`domain/sdf` owns:

- implicit fields and signed-distance sampling
- field composition and transform wrappers
- SDF query behavior (raymarch, projection, classification)

`domain/geometry` owns:

- explicit primitives and bounds (`Aabb3`, `Ray3`, `Sphere`, `Plane`)
- explicit geometric intersection helpers

Rule of thumb:
- If the canonical representation is implicit distance, it belongs in `sdf`.
- If the canonical representation is explicit shape/bounds data, it belongs in
  `geometry`.

## SDF vs Spatial

`domain/spatial` (future) should own:

- BVH/chunk/clipmap/LOD indexing/runtime structures
- field residency, streaming, and region addressing systems

`sdf` should remain independent from runtime partitioning/indexing concerns.

## SDF vs Engine Runtime

`sdf` does not own:

- ECS integration
- plugin wiring and frame orchestration
- scene graph/runtime world ownership
- game runtime controller behavior

Engine/runtime systems may consume `sdf`, but `sdf` stays engine-agnostic.

## SDF vs Render

`sdf` may provide query primitives that render code consumes, but it does not
own shader/runtime orchestration, pipeline setup, or render graph policy.
