---
title: Render Product Graph Platform
description: Accepted design for product-first render graph planning, production-track sequencing, and the boundary between Product Graph/Product Jobs and render execution graph compilation.
status: accepted
owner: engine
layer: engine-runtime / product-platform
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ./sdf-product-renderer-and-gpu-residency-design.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ./execution-fabric-and-product-jobs-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./render-contract-ergonomics-design.md
  - ./feature-owned-render-contributions-design.md
  - ./render-execution-graph-compiler-maturity-design.md
  - ./product-surface-platform-hardening-design.md
  - ./render-fragment-data-driven-maturity-design.md
  - ./render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../engine/roadmaps/fully-featured-renderer-roadmap.md
  - ../../workspace/production-tracks.yaml
---

# Render Product Graph Platform

## Status

This is the accepted boundary design for `PT-RENDER-PG`.

It ratifies the long-term renderer production-track doctrine and the ownership
boundary between Product Graph/Product Jobs and render execution graph
compilation. It does not authorize standalone code changes. Implementation
still flows through the WR roadmap, the relevant milestone design gates,
architecture governance when required, validation, and closeout evidence.

## Purpose

Runenwerk needs a product-first render platform that can support viewport
products, material previews, field/debug products, SDF/world products, drawing
surfaces, render fragments, multi-surface presentation, and production
diagnostics without making the renderer the owner of product truth.

The target shape is:

```text
domain products and Product Jobs
  -> prepared render product selections
  -> feature-owned render fragments and prepared contributions
  -> Render Execution Graph Compiler
  -> PreparedRenderFrame + compiled execution plan
  -> backend runtime execution and presentation
```

The renderer remains an execution and presentation layer over prepared products.

## Doctrine Alignment

Runenwerk stays product-first:

- domains and Product Jobs own product truth, lineage, freshness, authority
  class, query policy, fallback legality, rebuild policy, residency intent, and
  diagnostics;
- apps compose workflows, windows, viewports, editors, and product presentation;
- engine render consumes prepared render selections, prepared views, target
  alias bindings, feature contributions, fragments, and backend-neutral
  residency intent;
- renderer-owned GPU buffers, textures, pipelines, bind groups, upload queues,
  target caches, history resources, captures, timings, and presentation state
  are derived execution state only.

No renderer-owned world, material, drawing, prefab, gameplay, editor, or product
truth is allowed.

## Product Graph Versus Render Execution Graph

The Product Graph and Product Jobs own product semantics:

- product dependencies;
- lineage and generations;
- freshness, stale, ghost, and failed-preserved state;
- authority class and query policy;
- fallback legality;
- rebuild policy;
- residency intent;
- product diagnostics.

The Render Execution Graph Compiler owns render execution validation only. It
consumes prepared render product selections and feature-owned render fragments.
It validates:

- render resource declarations and access roles;
- pass order and dependencies;
- target alias resolution;
- dynamic target and prepared-view compatibility;
- history scope and invalidation signatures;
- resource lifetimes;
- backend capability constraints.

The compiler must not compute product freshness, authority, fallback legality,
rebuild policy, or product dependency truth. It receives those decisions as
prepared input and diagnostics.

This is not an RDG-first rewrite. Whole-frame render graph compilation is an
execution-planning layer over existing prepared product contracts. It does not
replace Product Jobs, Product Graph, `RenderFlow`, or prepared-frame ownership.

## Backend Runtime Boundary

The backend runtime owns WGPU execution mechanics only:

- allocation and reuse of backend textures, buffers, samplers, and bind groups;
- command encoding and submission;
- pipeline layouts, shader modules, pipeline caches, and bind group caches;
- dynamic target realization, uploads, captures, timings, and presentation;
- surface and swapchain recovery.

Backend handles must not cross into domain, app, UI, or product descriptions.
Backend resource state must not become product identity, product lineage, or app
workflow state.

## Submit Boundary

Render submit consumes `PreparedRenderFrame` and compiled execution plans only.

Render submit must not perform live ECS extraction for product, scene, view,
target, feature, fallback, shader, residency, or graph decisions. Missing,
stale, fallback, rejected, or over-budget products must be represented in
prepared data and diagnostics before submit.

## Future Refactor Target

The current central feature path remains valid for this planning slice. The
future refactor target is:

- `engine/src/plugins/render/frame/contributions.rs::PreparedFeaturePayload`;
- `engine/src/plugins/render/runtime/frame_prepare.rs::build_frame_feature_contributions`.

The long-term direction is to replace central feature payload growth and central
feature collection with registered, feature-owned contribution collectors and
render-fragment contracts.

This first slice does not refactor those files.

## Feature Contribution Rules

Feature-owned contributions must remain:

- typed;
- inspectable;
- capability-declared;
- diagnostic-producing;
- validated before submit.

The migration must not replace `PreparedFeaturePayload` with `Box<dyn Any>`,
opaque type-erased blobs, stringly payload maps, or unvalidated plugin packets.
Type erasure may only exist behind typed registration and inspection contracts
that preserve validation, diagnostics, and capability declarations.

## Safe Future Migration

A safe later implementation should follow this sequence:

1. Add a feature contribution registry beside the current central path.
2. Migrate one low-risk feature first.
3. Add a compatibility adapter that fills the existing
   `PreparedFrameContributions` structure.
4. Prove equivalent prepared-frame inspection and diagnostics.
5. Migrate material, draw, world, deformation, wind, cave, detail, and
   procedural features incrementally.
6. Keep central enum variants during migration.
7. Add tests that prevent submit-time ECS extraction and unvalidated payloads.
8. Remove central enum variants only after all feature paths are registry-owned.
9. Add a guard that rejects new central variants for newly added features.

## Explicitly Deferred / Requires Separate Design

The following areas remain outside this planning slice and require separate
accepted designs, WR rows, or product-family gates before implementation:

- SDF brick/page-table GPU residency, sparse bricks, clipmaps, distance mips,
  analytic SDF instances, raymarch acceleration, empty-space skipping, tile
  candidate lists, and temporal reprojection.
- Mesh/model/skinning/deformation render products.
- GPU-driven visibility, culling, LOD, indirect drawing, impostors, and ghost
  summaries.
- Material shader graph lowering, shader specialization, pipeline cache policy,
  and last-good shader fallback.
- Particles/VFX GPU simulation, emitters, trails, decals, sorting, SDF
  collision, and lighting.
- Water, wetness, weather, snow, sand, erosion, heat, humidity, flow fields, and
  shoreline response.
- Drawing-app low-latency immediate preview lane.
- Native multi-window and multi-surface lifecycle.
- Plugin/mod render-fragment ABI, schema, migration, and versioning policy.
- Render capture/replay and deterministic diagnostics.
- Asset cooking and build pipeline.
- GPU memory allocator, transient aliasing, dynamic target eviction, history
  target retention, and upload budgets.
- CPU, GPU, editor, and product rebuild performance budgets.

## Acceptance Evidence

This design was accepted for PM-RENDER-PG-001 after:

- stale render docs have been reconciled with implemented product-surface,
  prepared-view, target-alias, and dynamic-target behavior;
- the fully featured renderer roadmap maps FR-0 through FR-8 to
  `PM-RENDER-PG` milestones or explicitly deferred product-family designs;
- production, roadmap, docs, and planning validators pass;
- Product Graph/Product Jobs and Render Execution Graph Compiler ownership is
  unambiguous;
- architecture governance decides that the current accepted ADR/design set is
  sufficient for PM-RENDER-PG-001 and that a future `domain/render_contracts`
  crate should wait for a concrete cross-domain engine-agnostic consumer.

## Validation

The documentation-only planning slice must pass:

```text
task production:render
task docs:validate
task production:validate
task production:check
task roadmap:validate
task roadmap:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

No renderer code, tests, public Rust APIs, or runtime behavior should change in
this slice.
