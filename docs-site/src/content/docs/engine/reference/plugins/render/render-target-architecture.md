---
title: "Render Target Architecture"
description: "Documentation for Render Target Architecture."
---

# Render Target Architecture

This is the target steady-state architecture for render runtime evolution.

## Architecture Rule

Render stays:

- ECS-prepared
- compiler-shaped
- renderer-executed
- feature/plugin-extensible

## Layered Shape

1. Authoring layer:
   - `engine::plugins::render::api` remains ergonomic and declarative.
2. Prepare layer:
   - `frame_render_prepare_system` extracts render-relevant ECS state and publishes `PreparedRenderFrame`.
3. Compile layer:
   - graph compile: validation/order/inspectability.
   - execution compile: explicit pass execution metadata.
4. Execute layer:
   - renderer consumes prepared packet + compiled execution plans and owns backend/runtime artifacts.
5. Feature layer:
   - feature descriptors and prepared contributions integrate through typed contracts.

## Ownership Boundary

- ECS/domain side owns:
  - flow and feature registries
  - shader metadata/revisions
  - prepared frame and contribution payloads
  - compile metadata and inspection snapshots
  - cache stats metadata
- renderer/backend side owns:
  - all `wgpu` runtime objects
  - runtime flow resources and temporal/history allocations
  - command encoding and submission

`RenderFrameDataRegistry` remains compatibility-only (projection helpers/tests), not active runtime submission.

## Typed Import Boundary

Active runtime accepts typed import semantics only:

- surface color
- surface depth
- builtin UI draw list
- history categories (typed texture/buffer)

Generic external imports remain compatibility constructors and are rejected in active runtime validation.

## Multi-view and Feature Policy

- Prepared frame carries a view container (`PreparedViewFrame`) even for single-view operation.
- Execution plans carry view masks (`CompiledViewMask`) to support view-scoped pass subsets.
- Active runtime execution is currently single-view only; multi-view packets fail fast to avoid misleading partial support.
- Feature contribution status/fallback policy is resolved in prepare and executed without submit-time world extraction.

## Non-goals

- no full render-graph scheduler rewrite
- no generalized external imports beyond typed supported categories
- no universal simulation abstraction inside renderer

## Migration Reference

- [Final architecture migration roadmap](../../../roadmaps/render-final-architecture-migration.md)
