---
title: "Render Target Architecture"
description: "Documentation for Render Target Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
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
   - Product-surface requests are frozen here: prepared views, flow invocations, target alias bindings, dynamic target descriptors, and history signatures.
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

Active runtime accepts typed import semantics for executable surface-color/UI/history paths:

- surface color
- builtin UI frame contribution
- history categories (typed texture/buffer)

Surface-depth import remains typed declaration compatibility, but graphics depth attachments are currently runtime-backed through flow-owned depth targets.

Generic external imports remain compatibility constructors and are rejected in active runtime validation.

## Multi-view and Feature Policy

- Prepared frame carries a view container (`PreparedViewFrame`) even for single-view operation.
- Execution plans carry view masks (`CompiledViewMask`) to support view-scoped pass subsets.
- Prepared frame requests can describe offscreen product views and per-flow invocations.
- Active renderer execution for dynamic target aliases and dynamic target cache allocation remains in the render product surface foundation bundle; callers must not emulate product surfaces by cloning flows or suffixing static target labels.
- Feature contribution status/fallback policy is resolved in prepare and executed without submit-time world extraction.

## Product Surface Target Model

Target identity is explicit:

- flow-owned surface-format target: declared directly on a `RenderFlow` with `with_color_target(...)` and resolved to the selected surface format;
- flow-owned exact-format target: declared with `with_color_target_exact(...)` when byte truth requires a specific color format independent of the selected surface format;
- target alias: static authoring placeholder resolved by a prepared flow invocation;
- dynamic target key: backend-neutral runtime address requested through `RenderDynamicTextureTargetRequestRegistryResource`;
- surface target: the main swapchain surface import.

History retention is explicit:

- flow-owned history textures are declared with `with_history_texture(...)` and usually updated through `copy_pass(...)`;
- dynamic targets carry `RenderDynamicTextureRetention`;
- prepared views and invocations carry history signatures used by future cache invalidation and inspection.

`copy_pass(...)` is a raw texture transfer contract, not a color conversion contract. It may copy between color formats that are identical after stripping the sRGB suffix, but unrelated color formats and depth/stencil formats must be rejected. Shader blit/convert behavior belongs in an explicit future pass family.

## Non-goals

- no full render-graph scheduler rewrite
- no generalized external imports beyond typed supported categories
- no universal simulation abstraction inside renderer

## Migration Reference

- [Final architecture migration roadmap](../../../roadmaps/render-final-architecture-migration.md)
