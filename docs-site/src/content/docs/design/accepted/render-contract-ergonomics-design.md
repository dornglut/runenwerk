---
title: Render Contract Ergonomics Design
description: Accepted design for PM-RENDER-PG-002 render-flow and product-surface authoring ergonomics without renderer-owned product truth.
status: accepted
owner: engine
layer: engine-runtime / render public API
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ./render-product-graph-platform-design.md
  - ./render-execution-graph-compiler-maturity-design.md
  - ./product-surface-platform-hardening-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
related_roadmaps:
  - ../../engine/roadmaps/fully-featured-renderer-roadmap.md
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# Render Contract Ergonomics Design

## Status

This is the accepted design contract for `PM-RENDER-PG-002`.

It accepts a bounded ergonomics slice for shared render product-surface request
construction. It does not mark `PM-RENDER-PG-002` complete, does not assign
`completion_quality`, and does not authorize implementation until the milestone
is linked to a legal bounded WR row and the normal production workflow gates
allow code changes.

## Goal

Make common product-surface and render-flow authoring paths easier to use while
preserving the accepted ownership boundary:

```text
owning app/domain selects products and targets
  -> engine render helper builds backend-neutral request data
  -> producer explicitly publishes dynamic targets and prepared-frame requests
  -> RenderPrepare validates and snapshots prepared views, invocations, aliases, and dynamic targets
  -> RenderSubmit consumes the prepared frame only
```

The renderer remains an execution and presentation layer. It must not own
product truth, product selection, freshness, fallback legality, authority,
rebuild policy, or residency policy.

## Current Friction

The current architecture has the right ownership split, but the public
authoring surface makes product-surface producers assemble too much repeated
low-level prepared-frame data directly.

Observed touchpoints:

- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::build_viewport_render_job`
  manually builds `PreparedViewFrame`, `PreparedFlowInvocationRequest`, alias
  maps, uniform overrides, and product target bindings.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs::material_preview_scene_surface_flow_request`
  and
  `apps/runenwerk_editor/src/runtime/systems/material_preview.rs::material_preview_flow_requests`
  repeat the same prepared-view and alias-binding pattern.
- `engine/src/plugins/render/frame/packet.rs::PreparedRenderFrameRequestResource::replace_contribution`
  validates duplicate view and invocation ownership but returns unstructured
  `anyhow::Error` and does not keep producer-scoped request diagnostics.
- `engine/src/plugins/render/frame/view.rs::PreparedViewFrame` has useful
  constructors, but no ergonomic history-signature builder.
- `engine/src/plugins/render/frame/packet.rs::PreparedFlowInvocationRequest`
  is a public field bag with one uniform helper, so common alias and history
  patterns require repeated `BTreeMap` assembly.
- `engine/src/plugins/render/resource/dynamic_target.rs::RenderDynamicTextureTargetDescriptor`
  has a correct explicit constructor, but common color/depth/sampleable target
  shapes need repeated parameter plumbing.

These are ergonomics defects, not ownership defects. The renderer should make
the correct contract easy; it must not infer selected products, source truth,
freshness, authority, fallback legality, or rebuild behavior.

## Locked Decisions

`PM-RENDER-PG-002` has these accepted decisions:

- Implementation must migrate exactly two producers in this slice:
  editor viewport render request construction and material preview render
  request construction.
- Shared helpers must be return-only. They may build request batches for dynamic
  texture target descriptors, prepared views, and prepared flow invocation
  requests. They must not publish into ECS resources.
- Producer systems keep explicit publication into
  `RenderDynamicTextureTargetRequestRegistryResource` and
  `PreparedRenderFrameRequestResource`.
- `WR-003` remains contextual support evidence only. A new bounded implementation
  WR row must be created for `PM-RENDER-PG-002` before code changes are made.
- `PreparedRenderFrameRequestResource` must return typed request errors and
  expose producer-scoped typed diagnostics instead of only returning
  unstructured `anyhow::Error`.
- Expected closeout quality is `bounded_contract` after implementation and
  validation pass. This milestone must not claim `runtime_proven` or
  `perfectionist_verified`.

## Owning Modules

- `engine/src/plugins/render/frame/product_surface.rs` owns return-only
  product-surface request helpers that combine prepared views, prepared flow
  invocations, target alias bindings, history signatures, and dynamic target
  descriptors into backend-neutral request batches.
- `engine/src/plugins/render/frame/packet.rs` keeps `PreparedRenderFrame`,
  `PreparedRenderFrameRequestResource`, and request validation. It gains typed
  request errors and producer-scoped diagnostics while preserving the prepared
  frame packet as the submit boundary.
- `engine/src/plugins/render/frame/view.rs` owns prepared-view constructors and
  history-signature helpers.
- `engine/src/plugins/render/resource/dynamic_target.rs` owns backend-neutral
  dynamic target descriptor constructors and validation.
- `engine/src/plugins/render/inspect/prepared_frame.rs` owns inspection DTOs for
  successful prepared-frame snapshots.
- `docs-site/src/content/docs/engine/reference/plugins/render/` owns usage docs
  for common authoring paths.

No new `domain/render_contracts` crate is required for PM-002. Create one only
if a later accepted design needs engine-agnostic render request contracts shared
outside `engine/src/plugins/render`.

## Public API Shape

The implementation must be additive and migration-safe:

- Add `PreparedViewFrame::with_history_signature(...)`.
- Add `PreparedFlowInvocationRequest::new(...)`.
- Add fluent `PreparedFlowInvocationRequest` helpers for:
  - target alias bindings;
  - surface color aliases;
  - surface depth aliases;
  - dynamic texture aliases;
  - flow-owned aliases;
  - history signatures;
  - uniform overrides.
- Add `RenderDynamicTextureTargetDescriptor` constructors for:
  - `color_sampled(...)`;
  - `color_attachment_only(...)`;
  - `storage_sampled(...)`;
  - `depth_sampled(...)`.
- Add return-only product-surface request batch helpers that give callers the
  dynamic descriptors, prepared views, and prepared invocations to publish
  explicitly.

Dynamic target constructors must remain compatible with existing descriptor
validation. They may reduce repeated parameter plumbing, but must not infer
product semantics, selected products, retention policy defaults, freshness,
authority, fallback legality, or rebuild behavior.

Do not add APIs such as `publish_viewport_request(world, ...)`, and do not pass
renderer-private handles into editor producers.

## Diagnostics Contract

`PreparedRenderFrameRequestResource` diagnostics must be first-class typed DTOs,
not console text or ad hoc strings.

PM-002 introduces:

- `PreparedRenderFrameRequestError`;
- `PreparedRenderFrameRequestDiagnostic`;
- `PreparedRenderFrameRequestResource::diagnostics()`.

Each diagnostic must include:

- producer id;
- existing producer id when the conflict is cross-producer;
- view id or invocation id;
- request kind;
- message.

The bounded PM-002 diagnostics cover:

- duplicate view within one producer;
- duplicate view across producers;
- duplicate invocation within one producer;
- duplicate invocation across producers.

Unknown view references remain caught during `RenderPrepare`, where flow
registry state is available. Dynamic target, product selection, history, and
residency diagnostics continue to live in their owning resources and inspection
surfaces; PM-002 may use them in tests and docs but must not centralize their
policy into the request helper.

## Implementation Sequence

Implementation must follow this order:

1. Accept this design and update indexes, related links, and production-track
   gate paths.
2. Create a new bounded implementation WR row through roadmap intake/apply and
   link `PM-RENDER-PG-002` to that row. Do not repurpose `WR-003`.
3. Run the relevant production planning gate for the milestone and WR row before
   code changes.
4. Add additive constructors, fluent helpers, and typed request diagnostics.
5. Add `engine/src/plugins/render/frame/product_surface.rs` with return-only
   request batch helpers.
6. Migrate editor viewport request construction.
7. Migrate material preview request construction.
8. Update render usage docs and public API reference with the common path.
9. Add focused tests.
10. After validation passes, create closeout evidence and only then update
    `PM-RENDER-PG-002` completion evidence.

## Required Tests And Validation

Focused tests:

- `engine/tests/render_dynamic_targets.rs` proves helper-built descriptors,
  prepared views, prepared flow invocation requests, alias bindings, history
  signatures, typed duplicate diagnostics, and prepared-frame inspection after
  helper-built requests.
- `engine/tests/render_runtime_inspect.rs` proves inspection exposes
  helper-built prepared frame state without backend handles.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs` tests prove
  viewport jobs use the shared helper path and still bind the correct scene,
  picking, and overlay targets.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` tests prove
  material preview requests use the shared helper path and do not retarget the
  viewport scene alias.

Validation commands:

```text
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
cargo test -p runenwerk_editor viewport::render_jobs
cargo test -p runenwerk_editor material_preview
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Write Scope

The implementation WR row for PM-002 must keep write scope bounded to:

- `engine/src/plugins/render`;
- `apps/runenwerk_editor/src/runtime/viewport`;
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs`;
- relevant render/product-track docs and focused tests.

## Non-Goals

PM-002 must not:

- move Product Graph, Product Jobs, freshness, authority, fallback legality, or
  rebuild policy into `engine/src/plugins/render`;
- implement feature-owned render contribution collectors from PM-RENDER-PG-003;
- mature whole-frame execution graph compiler diagnostics from PM-RENDER-PG-004;
- claim broad product-surface hardening for every producer from
  PM-RENDER-PG-005;
- add native multi-window or multi-swapchain ownership from PM-RENDER-PG-006;
- implement data-driven render fragments or hot reload from PM-RENDER-PG-007;
- implement material truth or material lowering;
- claim production-readiness inspection, budgets, capture/replay, final
  examples, or completion-quality beyond `bounded_contract` from
  PM-RENDER-PG-008.

## Acceptance Bar

This design is accepted when:

- the decisions above are explicit in this document;
- the API shape is additive and migration-safe for existing render-flow callers;
- diagnostics are named as typed DTOs rather than only log strings;
- docs and tests prove the common product-surface path without cloned flows,
  suffixed static labels, backend handles, or renderer-owned product truth;
- architecture governance confirms no ADR is needed beyond the accepted Render
  Product Graph Platform boundary.
