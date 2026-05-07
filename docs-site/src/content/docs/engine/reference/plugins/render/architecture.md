---
title: "Render Plugin Architecture"
description: "Documentation for Render Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
---

# Render Plugin Architecture

## Primary Architecture Docs

- Target architecture summary:
  - [`render-target-architecture.md`](render-target-architecture.md)
- Final migration roadmap:
  - [`../../../roadmaps/render-final-architecture-migration.md`](../../../roadmaps/render-final-architecture-migration.md)
- Internal roadmap/backlog:
  - [`../../../plugins/render/docs/roadmap.md`](../../../plugins/render/docs/roadmap.md)

## Current Public Surface

- `RenderFlow` v2 builder surface
- pass builders: `compute_pass`, `fullscreen_pass`, `graphics_pass`, `copy_pass`, `present_pass`, `builtin_ui_composite_pass`
- typed handles: storage arrays, uniforms, ping-pong handles
- flow-owned render targets: color targets, depth targets, history textures
- staged product-surface contracts: target aliases, dynamic texture target descriptors, prepared offscreen views, and prepared flow invocations
- GPU params derives: `GpuUniform`, `GpuStorage`, `GpuParams`, `ToGpuValue`
- ECS-first state projection with `Resource`-bound state APIs

## Runtime Boundary

- Owns: render runtime resources, flow registry integration, and render prepare/submit scheduling.
- Non-ownership: app input mapping ownership and scene lifecycle ownership.
- Prepare/submit boundary artifact: `PreparedRenderFrame` in `engine/src/plugins/render/frame/`.
- Prepared frame packets carry main/offscreen views, flow input snapshots, prepared flow invocations, dynamic target descriptor snapshots, target alias bindings, UI surface bindings, and history signatures.
- Runtime compatibility helper: `RenderFrameDataRegistry` for projection helpers/tests only, not active submission.
- Feature/domain payloads are carried as prepared contributions (`PreparedFrameContributions`) with explicit status/fallback policy.
- Feature prepared payload handoff resources are explicit (`PreparedDrawFeatureResource`, `PreparedMaterialFeatureResource`, `PreparedDeformationFeatureResource`) and consumed only in `RenderPrepare`.
- Active runtime validation accepts typed surface-color/UI/history semantics; external imports are compatibility-only.
- Runtime-backed graphics depth uses flow-owned depth targets. Imported surface-depth declarations remain compatibility metadata until the renderer exposes a prepared surface-depth texture.
- Active execute path is still conservative for product surfaces: prepared packets can describe offscreen views and dynamic targets, but renderer-owned dynamic target cache allocation and target-alias pass execution remain render product surface bundle work.

## Inspection Boundary

Inspection APIs live under `engine::plugins::render::inspect` and expose graph/resource/texture/timing/prepared-frame diagnostics without leaking backend internals into common-path flow authoring.

## API Map

Use `public-api-reference.md` for the canonical map of:

- common path authoring APIs
- frame boundary APIs
- execution-compile metadata APIs
- runtime/debug integration surfaces
