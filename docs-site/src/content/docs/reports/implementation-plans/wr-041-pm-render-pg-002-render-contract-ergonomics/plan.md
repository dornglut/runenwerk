---
title: WR-041 PM-RENDER-PG-002 Render Contract Ergonomics Plan
description: Implementation contract for the bounded PM-RENDER-PG-002 render contract ergonomics slice.
status: active
owner: engine
layer: engine-runtime / render public API
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/render-contract-ergonomics-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-041 PM-RENDER-PG-002 Render Contract Ergonomics Plan

## Goal

Implement `PM-RENDER-PG-002` as one bounded ergonomics slice for shared,
return-only render product-surface request construction. The slice improves
request authoring for editor viewport and material preview producers while
preserving product truth, product selection, freshness, fallback legality,
authority, rebuild policy, and residency policy outside the renderer.

## Source Of Truth

- Production milestone: `PM-RENDER-PG-002`.
- Roadmap item: `WR-041`.
- Accepted design:
  `docs-site/src/content/docs/design/accepted/render-contract-ergonomics-design.md`.
- Boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- Support context only: `WR-003`.

## Readiness

- `PM-RENDER-PG-001` is completed with doctrine closeout evidence.
- The render contract ergonomics design is accepted.
- `WR-041` is the bounded implementation WR row for PM-002.
- Architecture governance kickoff was run for the bounded scope.
- No ADR is required while helper APIs stay return-only and do not change
  ownership, dependency direction, product truth, residency policy, or fallback
  legality.

## Implementation Scope

Allowed write scopes:

- `engine/src/plugins/render`;
- `engine/tests`;
- `apps/runenwerk_editor/src/runtime/viewport`;
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs`;
- `apps/runenwerk_editor/tests`;
- `docs-site/src/content/docs/engine/reference/plugins/render`;
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`;
- `docs-site/src/content/docs/design/accepted/render-contract-ergonomics-design.md`;
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-21-pm-render-pg-002-render-contract-ergonom`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- generated roadmap docs and diagrams;
- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- generated production-track docs and diagrams;
- PM-002 closeout evidence.

The implementation owns these concrete modules:

- `engine/src/plugins/render/frame/product_surface.rs` for return-only request
  batch helpers.
- `engine/src/plugins/render/frame/packet.rs` for prepared-frame request
  constructors, typed errors, and typed diagnostics.
- `engine/src/plugins/render/frame/view.rs` for prepared-view history
  signatures.
- `engine/src/plugins/render/resource/dynamic_target.rs` for dynamic target
  descriptor constructors.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs` for the editor
  viewport producer migration.
- `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs` for viewport
  dynamic descriptor constructor usage.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` for the
  material preview producer migration.

## Public API Contract

Additive engine render APIs:

- `PreparedViewFrame::with_history_signature(...)`;
- `PreparedFlowInvocationRequest::new(...)`;
- alias, history, and uniform helpers on `PreparedFlowInvocationRequest`;
- `RenderDynamicTextureTargetDescriptor::{color_sampled, color_attachment_only,
  storage_sampled, depth_sampled}`;
- return-only product-surface request and batch helpers under
  `engine/src/plugins/render/frame/product_surface.rs`.

The helper may assemble dynamic target descriptors, prepared views, and
prepared flow invocation requests. It must not publish into ECS resources, hide
publication errors, infer product semantics, choose selected products, decide
freshness, own fallback legality, own authority, own rebuild policy, or own
residency policy.

Producer systems must still explicitly publish into:

- `RenderDynamicTextureTargetRequestRegistryResource`;
- `PreparedRenderFrameRequestResource`.

## Diagnostics Contract

Replace unstructured prepared-frame request validation with typed diagnostics:

- `PreparedRenderFrameRequestError`;
- `PreparedRenderFrameRequestDiagnostic`;
- `PreparedRenderFrameRequestResource::diagnostics()`.

Diagnostics must include producer id, existing producer id for cross-producer
conflicts, view id or invocation id, request kind, and a human-readable
message. The bounded diagnostic cases are:

- duplicate view within producer;
- duplicate view across producers;
- duplicate invocation within producer;
- duplicate invocation across producers.

Unknown view references remain a `RenderPrepare` concern because they require
flow registry state.

## Implementation Steps

1. Add dynamic target descriptor constructors and prepared-view/request fluent
   helpers.
2. Add typed prepared-frame request errors and producer-scoped diagnostics.
3. Add the return-only product-surface request helper module.
4. Migrate viewport render request construction to the shared helper path while
   keeping explicit ECS publication.
5. Migrate material preview render request construction to the same helper path
   while keeping explicit ECS publication.
6. Update render API reference and usage docs.
7. Add focused tests for helpers, descriptors, diagnostics, inspection, and both
   migrated producers.

## Non-Goals

Do not implement:

- native multi-window or multi-swapchain ownership;
- render fragments or hot reload;
- feature-owned contribution registries;
- render execution graph compiler maturity;
- production-readiness inspection, budgets, capture/replay, or final examples;
- material truth or material lowering;
- broad product-surface hardening beyond request ergonomics;
- renderer-owned product policy of any kind.

## Acceptance Criteria

- The accepted ergonomics design exists and remains indexed.
- `PM-RENDER-PG-002` points to the accepted ergonomics design and `WR-041`.
- `WR-003` is not the PM-002 implementation row.
- Shared return-only helpers exist and are used by viewport and material
  preview producers.
- Both producers still publish explicitly into ECS resources.
- `PreparedRenderFrameRequestResource` exposes typed errors and
  producer-scoped diagnostics.
- Dynamic target, view, invocation, alias, and history request construction is
  easier and covered by focused tests.
- No out-of-scope renderer ownership or policy changes are implemented.

## Validation

Required focused tests:

```text
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
cargo test -p runenwerk_editor viewport::render_jobs
cargo test -p runenwerk_editor material_preview
```

Required workflow validation:

```text
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

## Stop Conditions

Stop immediately if:

- `WR-041` cannot be promoted legally;
- production planning stops at a gate other than implementation;
- ownership of product truth or residency would need to move into renderer code;
- a helper would need to publish into ECS to work;
- another producer beyond viewport and material preview must be migrated;
- focused validation fails and the cause is outside the PM-002 scope;
- closeout evidence would need to claim `runtime_proven` or
  `perfectionist_verified`.

## Closeout Requirements

After implementation and validation pass:

- create PM-002 closeout evidence under
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-002-render-contract-ergonomics/closeout.md`;
- update `PM-RENDER-PG-002` evidence gates and completion audit;
- set PM-002 `completion_quality: bounded_contract`;
- record known quality gaps for later PM-003 through PM-008 work;
- close or archive `WR-041` only with completed evidence and matching roadmap
  validation.

## Perfectionist Closeout Audit

Expected completion quality is `bounded_contract`.

`PM-002` must not claim `runtime_proven` because it does not add a new runtime
product chain or GPU pixel proof. It must not claim `perfectionist_verified`
because later production milestones still own contribution collectors, compiler
maturity, multi-surface presentation, render fragments, broad product-surface
hardening, and final production inspection.
