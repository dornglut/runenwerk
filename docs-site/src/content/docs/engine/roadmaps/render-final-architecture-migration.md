---
title: "Render Final Architecture Migration Roadmap"
description: "Documentation for Render Final Architecture Migration Roadmap."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../design/active/editor-native-multi-window-presentation-design.md
  - ../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ./fully-featured-renderer-roadmap.md
---

# Render Final Architecture Migration Roadmap

This roadmap is the implementation contract for the final render architecture:

- ECS-prepared
- compiler-shaped
- renderer-executed
- feature/plugin-extensible

It is the foundation cutover plan, not the full renderer capability roadmap.
For long-term sequencing across SDF/world rendering, editor viewport products,
materials, fragments, history, and production inspection, use
[Fully Featured Renderer Roadmap](./fully-featured-renderer-roadmap.md).

## Non-goals

- No full render-graph scheduler rewrite.
- No generalized external imports beyond typed supported categories.
- No universal simulation abstraction inside renderer.

## Current Status (May 21, 2026)

- Phases 1-7 are active on the runtime path.
- Phase 8 product-surface foundation behavior is landed:
  - `PreparedRenderFrame` carries main/offscreen views, prepared flow invocations, target alias bindings, dynamic target descriptors, and history signatures;
  - dynamic target descriptor/request validation exists;
  - renderer-owned dynamic target cache allocation and target-alias execution are implemented foundation behavior.
- Phase 9 cleanup/docs/cutoff hardening remains in progress.

The implemented product-surface foundation bundle is recorded at `docs-site/src/content/docs/design/implemented/render-product-surface-foundation-bundle-design.md`. It pulled the product-surface portion of Phase 8 forward with dynamic targets, target aliases, prepared render views, history invalidation, and inspection support.

Native OS multi-window and multi-swapchain presentation is specified separately in `docs-site/src/content/docs/design/active/editor-native-multi-window-presentation-design.md`.

## Final Target Shape

1. Authoring:
   - `engine::plugins::render::api` remains ergonomic (`RenderFlow`, pass builders, typed state projection).
2. Prepare:
   - `frame_render_prepare_system` produces one owned immutable `PreparedRenderFrame`.
3. Compile:
   - graph compile stays focused on validation/order/inspectability.
   - execution compile produces explicit execution metadata for binding/access/target/dispatch/import semantics.
4. Execute:
   - renderer consumes prepared frame + execution plan only.
   - renderer owns all `wgpu` artifacts and runtime caches.
5. Features:
   - feature descriptors and fallback policy are explicit (`Ready | Stale | Disabled | Missing` + policy).

## Ownership Cut

- ECS owns:
  - flow and feature registries
  - shader metadata/revisions
  - prepared frame state and feature contributions
  - compile metadata and debug/inspection snapshots
  - cache stats metadata resources
- Renderer owns:
  - `wgpu` pipelines/layouts/modules/bind groups/samplers
  - runtime flow resources and temporal/history allocations
  - command encoding/submission

`RenderFrameDataRegistry` remains compatibility-only for projection helpers/tests and is excluded from active submit/execute.

## Staged Plan

### Phase 1: Prepared frame boundary

- Objective:
  - move prepare/submit boundary to `PreparedRenderFrame`.
- Core files:
  - `engine/src/plugins/render/frame/*`
  - `engine/src/plugins/render/renderer/submit.rs`
- Gate:
  - submit consumes packet and does no live ECS extraction for flow state.

### Phase 2: Feature registry + fallback model

- Objective:
  - deterministic feature ordering and explicit fallback policy.
- Core files:
  - `engine/src/plugins/render/features/mod.rs`
  - `engine/src/plugins/render/frame/contributions.rs`
- Gate:
  - dependency ordering and cycle detection tests pass.

### Phase 3: Prepare/submit split hardening

- Objective:
  - move shader polling, surface/view snapshot, uniform/dispatch projection, and contribution gathering into prepare.
- Core files:
  - `engine/src/plugins/render/renderer/submit.rs`
  - `engine/src/plugins/render/plugin.rs`
- Gate:
  - cutoff guards verify submit has no extraction/projection/polling.

### Phase 4: Execution compilation and typed import model

- Objective:
  - compile execution-ready pass metadata and enforce typed external-import semantics.
- Core files:
  - `engine/src/plugins/render/graph/execution_plan.rs`
  - `engine/src/plugins/render/resource/descriptors.rs`
  - `engine/src/plugins/render/resource/import.rs`
  - `engine/src/plugins/render/graph/validation.rs`
- Gate:
  - unit tests cover binding order/access, target/depth plans, typed imports.

### Phase 5: Renderer executor path

- Objective:
  - runtime executes compiled execution plans only.
- Core files:
  - `engine/src/plugins/render/renderer/render_flow.rs`
  - `engine/src/plugins/render/renderer/prepare.rs`
- Gate:
  - no active-path ad hoc binding/target inference from raw graph arrays.

### Phase 6: Cache convergence

- Objective:
  - keep real backend caches renderer-owned and consolidate ECS stats authority.
- Core files:
  - `engine/src/plugins/render/renderer/pipeline_cache.rs`
  - `engine/src/plugins/render/pipelines/cache.rs`
  - `engine/src/plugins/render/backend/pipeline_cache.rs`
  - `engine/src/plugins/render/pipelines/flow_keys.rs`
- Gate:
  - stable flow frames hit cache for pipeline artifacts; stats observable via ECS.

### Phase 7: Material/draw/deformation contracts

- Objective:
  - establish prepared contribution contracts for world draw, materials, and deformation streams.
- Core files:
  - `engine/src/plugins/render/frame/contributions.rs`
  - `engine/src/plugins/render/features/mod.rs`
  - renderer integration paths in `engine/src/plugins/render/renderer/*`
- Policy:
  - compile-time specialization is restricted to pipeline-shaping state; runtime parameter data stays bind/update payload.
  - material specialization keys are core-render-owned key types with a material-feature-owned specialization fragment hashed into them.
- Gate:
  - contract and parity tests for CPU/GPU deformation paths.

### Phase 8: Multi-view + history management

- Objective:
  - make view identity explicit in execution/runtime contracts and history invalidation.
  - support product-surface prepared render views through the render product surface foundation bundle.
- Core files:
  - `engine/src/plugins/render/frame/view.rs`
  - `engine/src/plugins/render/graph/execution_plan.rs`
  - `engine/src/plugins/render/renderer/render_flow.rs`
  - runtime resource allocation paths
  - `engine/src/plugins/render/renderer/dynamic_targets.rs`
- Policy:
  - multi-view execution is per-flow-per-view, with execution-compiled view masks enabling view-scoped pass subsets when needed.
  - product-surface prepared render views are the first implementation target; broader OS/window multi-swapchain presentation remains a later concrete app requirement.
- Gate:
  - resize/view-signature changes deterministically invalidate view-local history resources.
  - one compiled flow can execute against multiple prepared offscreen product views without cloning the flow registry.

### Phase 9: Cutover cleanup and docs finalization

- Objective:
  - isolate/remove deprecated or misleading active-path scaffolding and finalize docs.
- Core files:
  - `engine/src/plugins/render/renderer/frame_bindings.rs`
  - `engine/tests/render_cutoff_guard.rs`
  - render docs in `docs-site/src/content/docs/engine/reference/plugins/render/`
- Gate:
  - compatibility coverage stays green (`render_flow_v2`), cutoff guards prevent reintroduction.

## Feature Fallback Contract

Fallback is resolved in prepare and carried in packet contribution status:

- `Ready`: execute normally.
- `Stale`: allowed policy-based degradation (`ReuseLastGood` or equivalent).
- `Disabled`: execute fallback policy without live extraction.
- `Missing`: execute fallback policy without submit-time world queries.

Default behavior is `SkipFeaturePasses`; higher strictness is opt-in (`FailFrame`).

## Runtime Acceptance Criteria

- submit performs no live ECS extraction for flow-declared data.
- prepared frame is owned, immutable, and published once per renderable frame.
- compiled execution plans carry explicit binding/access/target/dispatch semantics.
- stable flows avoid per-frame pipeline/layout/module/bind-group churn.
- renderer cache stats are visible through ECS metadata resources.
- UI composite semantics are explicit feature execution behavior, not fake generic imports.
