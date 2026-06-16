---
title: Viewport Camera And Projection Contract Platform
description: Long-term platform design for renderer procedural projection, editor viewport camera semantics, viewport surface presentation, and example evidence.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - workspace-viewport-expression-upgrade-design.md
  - renderer-procedural-population-hardening-platform-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - game-runtime-ui-projection-and-hud-platform-design.md
related_roadmaps:
  - ../workspace/roadmap-deferred.yaml
  - ../workspace/production-tracks.yaml
---

# Viewport Camera And Projection Contract Platform

## Decision

`PT-VIEWPORT-PROJECTION` owns the long-term planning path for camera,
projection, viewport presentation, and surface-fit contracts that cross renderer,
editor viewport, app adapter, example, and UI embedding boundaries.

This is not a renderer-only cleanup and not a UI feature track. The platform
must preserve these source-truth rules:

- camera intent belongs to the producer or editor viewport context that owns the
  interaction semantics;
- renderer code may own derived projection math, uniforms, surface-fit
  transforms, diagnostics, and example evidence;
- `PreparedViewFrame` carries view identity, target size, history identity, and
  render preparation data, not camera intent;
- UI primitives embed product or viewport surfaces and must not own camera or
  projection semantics;
- app/runtime adapters pack uniforms, route input, and bridge CPU picking to GPU
  products without becoming a new semantic owner.

The track exists because the current codebase has correct pieces in separate
owners, but the overall contract is not yet production-complete.

## Current Evidence And Gaps

`WR-101` completed reusable renderer procedural 2D camera projection for the
boids proof. That work is evidence for aspect-correct fill-viewport projection,
equal projected world x/y scale, and producer-owned camera intent.

The remaining platform gaps are broader:

- renderer examples still contain one-off surface/aspect decisions that should
  be reviewed against a typed presentation contract before final hardening;
- editor viewport camera state exposes a supported-looking orthographic
  projection variant without end-to-end runtime, rendering, and picking
  evidence;
- editor CPU picking and GPU projection derivation are close but not yet backed
  by one explicit drift-guard contract;
- viewport surface embedding still needs to stay camera-free while future
  identity typing and product-surface evidence improve.

These gaps should not be patched as local example fixes. They need a production
track, design gates, and follow-on WR rows with disjoint ownership.

## Ownership

Renderer ownership is limited to derived projection and presentation data:

- `engine/src/plugins/render/procedural/camera.rs`;
- module `engine::plugins::render::procedural::camera`;
- future renderer-owned surface-fit or procedural-ray helpers after accepted
  design.

Editor viewport ownership covers semantic camera and projection behavior:

- `domain/editor/editor_viewport/src/camera.rs`;
- module `domain::editor::editor_viewport::camera`;
- perspective, orthographic, orbit, pan, zoom, ray, and screen projection
  vocabulary once accepted.

Editor app adapter ownership covers runtime packing and input/output routing:

- `apps/runenwerk_editor/src/runtime/resources.rs`;
- `apps/runenwerk_editor/src/runtime/systems/picking.rs`;
- target-local uniform packing, viewport-local picking, and CPU/GPU drift tests.

UI ownership remains camera-free:

- `domain/ui`;
- surface embedding primitives, slots, and product or viewport surface identity;
- no camera intent, projection policy, world ray, or gameplay/editor camera
  semantics.

## Production Milestones

`PT-VIEWPORT-PROJECTION` is the long-term production track for this platform.

- `PM-VIEWPORT-PROJECTION-001` records governance, architecture, ADR triage,
  and the accepted follow-on WR split.
- `PM-VIEWPORT-PROJECTION-002` covers renderer surface-fit and procedural
  projection contracts.
- `PM-VIEWPORT-PROJECTION-003` covers editor viewport camera and projection
  hardening.
- `PM-VIEWPORT-PROJECTION-004` covers example migration and product-surface
  evidence.
- `PM-VIEWPORT-PROJECTION-005` closes the track at `runtime_proven` and hands
  renderer-only no-gap audit inputs to `PT-RENDER-PERFECTION`.

`WR-106` is the governance intake row for `PM-VIEWPORT-PROJECTION-001`. It must
not implement renderer, editor, UI, or example behavior.

## Future WR Split

`WR-106` completes only when it produces accepted governance evidence:

- the design status and ADR need are resolved;
- renderer, editor viewport, app adapter, and UI boundaries are named;
- follow-on implementation WR candidates have disjoint write scopes;
- each follow-on WR names focused validations and closeout evidence.

Expected follow-on rows:

- renderer surface-fit and procedural projection contracts;
- editor viewport camera and projection hardening;
- example and product-surface evidence migration;
- runtime-proven closeout and renderer perfection handoff.

## ADR Triggers

No ADR is required while future work preserves producer-owned camera intent,
renderer-derived uniforms, and camera-free UI embedding.

An ADR or accepted design update is required before:

- camera source truth moves into `PreparedViewFrame`;
- UI owns camera or projection semantics;
- a shared cross-domain camera ABI is introduced;
- renderer APIs start carrying editor-specific viewport concepts;
- editor camera contracts become canonical outside editor viewport ownership.

## Exclusions

This track does not own richer boid/flock behavior. Flock identity, overlap
policy, multiple flocks, affinity groups, split/merge behavior, attractors, and
semantic population dynamics remain separate behavior-authoring work in the
`WR-102` direction.

This track does not own game runtime HUD, world-space attachment UI, screen-space
nameplates, damage numbers, or gameplay UI projection. Screen-space game HUD
remains in `WR-104` and `PT-GAME-RUNTIME-UI`; world-space and entity-attached
game UI remains deferred to `PT-GAME-WORLDSPACE-UI`.

This track does not reopen `WR-101`. `WR-101` is completed renderer procedural
camera evidence and remains an input dependency.

## Fitness Functions

Before implementation promotion, follow-on rows must name tests for:

- renderer landscape, portrait, square, and extreme aspect projection;
- equal projected world x/y scale where a world projection contract requires it;
- surface-fit behavior that does not hide camera source-truth movement;
- editor CPU picking and GPU projection drift;
- orthographic projection support or explicit removal/deferment;
- examples proving migration without boids-only shortcuts;
- guards proving no viewport-specific camera truth enters `PreparedViewFrame`;
- guards proving generic UI primitives remain camera-free.
