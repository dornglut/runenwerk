---
title: "Render Plugin Architecture"
description: "Documentation for Render Plugin Architecture."
---

# Render Plugin Architecture

## Primary Architecture Docs

- Target architecture summary:
  - `engine/docs/reference/plugins/render/render-target-architecture.md`
- Final migration roadmap:
  - `engine/docs/roadmaps/render-final-architecture-migration.md`
- Internal roadmap/backlog:
  - `engine/src/plugins/render/docs/roadmap.md`

## Current Public Surface

- `RenderFlow` v2 builder surface
- pass builders: `compute_pass`, `fullscreen_pass`, `builtin_ui_composite_pass`
- typed handles: storage arrays, uniforms, ping-pong handles
- GPU params derives: `GpuUniform`, `GpuStorage`, `GpuParams`, `ToGpuValue`
- ECS-first state projection with `Resource`-bound state APIs

## Runtime Boundary

- Owns: render runtime resources, flow registry integration, and render prepare/submit scheduling.
- Non-ownership: app input mapping ownership and scene lifecycle ownership.
- Prepare/submit boundary artifact: `PreparedRenderFrame` in `engine/src/plugins/render/frame/`.
- Runtime compatibility helper: `RenderFrameDataRegistry` for projection helpers/tests only, not active submission.
- Feature/domain payloads are carried as prepared contributions (`PreparedFrameContributions`) with explicit status/fallback policy.
- Feature prepared payload handoff resources are explicit (`PreparedDrawFeatureResource`, `PreparedMaterialFeatureResource`, `PreparedDeformationFeatureResource`) and consumed only in `RenderPrepare`.
- Active runtime validation only accepts typed import semantics (surface/UI/history categories); external imports are compatibility-only.
- Active execute path is single-view only; multi-view packet execution is deferred and guarded by fail-fast runtime checks.

## Inspection Boundary

Inspection APIs live under `engine::plugins::render::inspect` and expose graph/resource/texture/timing diagnostics without leaking backend internals into common-path flow authoring.

## API Map

Use `public-api-reference.md` for the canonical map of:

- common path authoring APIs
- frame boundary APIs
- execution-compile metadata APIs
- runtime/debug integration surfaces
