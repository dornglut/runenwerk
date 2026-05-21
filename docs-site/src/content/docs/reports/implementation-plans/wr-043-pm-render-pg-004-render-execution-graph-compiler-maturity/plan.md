---
title: WR-043 PM-RENDER-PG-004 Render Execution Graph Compiler Maturity Plan
description: Promotion and implementation-readiness contract for the bounded PM-RENDER-PG-004 render execution graph compiler maturity slice.
status: active
owner: engine
layer: engine-runtime / render graph compiler
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/feature-owned-render-contributions-design.md
  - ../../../design/accepted/render-contract-ergonomics-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-043 PM-RENDER-PG-004 Render Execution Graph Compiler Maturity Plan

## Goal

Promote and implement `PM-RENDER-PG-004` as one bounded compiler maturity
slice.

The slice moves render execution graph failures that can be known before backend
command encoding into typed compiler/preflight diagnostics. It matures static
`RenderFlow` validation and prepared-frame execution preflight for resources,
pass order, target aliases, dynamic targets, history scopes, resource lifetime
windows, backend capability profiles, diagnostics, and inspection.

The renderer remains an execution compiler and backend runtime. It must not own
product truth, product freshness, product dependency truth, authority, fallback
legality, rebuild policy, or residency policy.

## Source Of Truth

- Production milestone: `PM-RENDER-PG-004`.
- Bounded implementation row: `WR-043`.
- Accepted PM-004 design:
  `docs-site/src/content/docs/design/accepted/render-execution-graph-compiler-maturity-design.md`.
- Boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- PM-003 prerequisite closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-003-feature-owned-render-contributions/closeout.md`.
- `WR-010` remains the later render fragment and hot-reload row for
  `PM-RENDER-PG-007`; it is not the PM-004 implementation row.

## Readiness

`task production:plan -- --milestone PM-RENDER-PG-004 --roadmap WR-043`
reported:

- milestone state: `ready_next`;
- WR state: `ready_next`;
- dependency `WR-042:completed`;
- next action: `write_promotion_contract`;
- promotion preflight: `promotable`.

Promotion command, after this contract is linked and validation passes:

```text
task roadmap:promote -- --id WR-043 --state current_candidate --evidence "Accepted PM-RENDER-PG-004 render execution graph compiler maturity design and active promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-043-pm-render-pg-004-render-execution-graph-compiler-maturity/plan.md"
```

Do not promote if validation fails, if source files changed enough that
`task ai:goal -- --track PT-RENDER-PG` must be rerun, or if another current
candidate blocks promotion and the workflow requires `task roadmap:switch-current`.

## Implementation Scope

Owned area:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md
```

Expected implementation modules:

```text
engine/src/plugins/render/graph/diagnostics.rs
engine/src/plugins/render/graph/capabilities.rs
engine/src/plugins/render/graph/prepared_validation.rs
engine/src/plugins/render/graph/resource_lifetimes.rs
engine/src/plugins/render/graph/planning.rs
engine/src/plugins/render/graph/validation.rs
engine/src/plugins/render/graph/execution_plan.rs
engine/src/plugins/render/inspect/graph_dump.rs
engine/src/plugins/render/inspect/plan.rs
engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs
```

Use nearby module names if implementation shows a better local fit, but keep
the boundaries explicit. Do not create catch-all helper files.

## Required Contracts

The implementation must add or refine typed, inspectable contracts for:

- static render execution graph diagnostics;
- prepared-frame execution preflight diagnostics;
- backend-neutral capability profiles;
- resource lifetime/read-write windows;
- target alias requirements and prepared binding compatibility;
- dynamic target descriptor compatibility with compiled alias usage;
- history scope validation by view id, invocation id, resource id, and history
  signature;
- compiled/preflight inspection data.

Runtime submit may keep defensive guards, but known preflight failures should be
reported through typed compiler diagnostics before command encoding where
possible.

## Implementation Steps

1. Add typed diagnostic DTOs and error/report types under the render graph
   subsystem.
2. Extend static validation or planning so resource/pass diagnostics can be
   surfaced as compiler diagnostics without losing existing
   `RenderFlowValidationIssue` coverage.
3. Add resource lifetime window computation from the compiled pass order and
   resource graph.
4. Add backend-neutral capability profile types and default runtime profile
   validation for the active supported pass/resource surface.
5. Add prepared-frame execution preflight over compiled flows, prepared views,
   prepared flow invocations, target alias bindings, dynamic target descriptors,
   invocation uniforms, dispatch preparation, feature-gated pass status, and
   history signatures.
6. Expose compiler/preflight diagnostics and lifetime/capability summaries
   through render inspection.
7. Convert duplicated late runtime errors to preflight diagnostics only after
   focused tests prove equivalent or stronger coverage. Keep backend runtime
   fail-closed guards.
8. Update render reference docs and roadmap notes to describe the compiler
   boundary and the non-ownership of product policy.

## Explicit Non-Goals

Do not implement:

- render fragments, fragment asset schemas, fragment registry, merge
  provenance, hot reload, or last-good fragment promotion;
- broad product-surface producer hardening across viewport, material preview,
  field/debug, drawing, or future preview producers;
- native multi-window or multi-surface presentation;
- material graph lowering, shader graph truth, product selection, product
  freshness, authority, fallback legality, rebuild policy, product dependency
  truth, or residency policy;
- production-readiness inspection, capture/replay policy, or performance budget
  closeout beyond compiler diagnostics needed here;
- renderer-private backend handles in app, domain, fragment, or product
  producer contracts.

## Acceptance Criteria

- Static flow validation and compiled planning expose typed diagnostics for
  invalid resources, pass order, target aliases, resource roles, lifetimes, and
  capability mismatches.
- Prepared-frame preflight validates target alias bindings, dynamic targets,
  history scopes, invocation uniforms, dispatch preparation, feature-gated pass
  status, and view masks before backend command encoding.
- Diagnostics include flow, pass, resource, invocation, prepared view, alias,
  dynamic target, history, capability, severity, kind, and message fields where
  relevant.
- Resource lifetime windows are derived from compiled pass order and are
  inspectable.
- Backend capability checks are backend-neutral and do not expose WGPU handles.
- Submit still consumes prepared frames and compiled/preflight output only; it
  does not perform live ECS extraction for product or policy decisions.
- `WR-010` remains available for PM-RENDER-PG-007 fragment and hot-reload work.

## Validation

Focused implementation validation must include:

```text
cargo test -p engine render_flow
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
```

Add or extend tests for:

- static compiler diagnostics;
- prepared-frame preflight diagnostics;
- target alias missing/mismatched prepared bindings;
- dynamic target usage and sampleability compatibility;
- history signature and view/invocation scope conflicts;
- resource lifetime windows;
- backend capability mismatch diagnostics;
- inspection of compiler/preflight reports;
- submit-boundary guards proving no live ECS extraction for graph/product
  decisions.

Workflow validation:

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

Stop and report instead of coding if:

- `task ai:goal -- --track PT-RENDER-PG` no longer selects PM-004/WR-043;
- `task production:plan -- --milestone PM-RENDER-PG-004 --roadmap WR-043`
  no longer reports a promotable or actionable row;
- promotion fails for anything other than exact metadata repair or a required
  current-candidate switch;
- implementation needs fragment assets, hot reload, product-surface producer
  migration, native multi-window, material lowering, or product policy;
- compiler diagnostics would need product truth or backend handle leakage;
- validation fails and cannot be repaired inside the bounded WR-043 scope.

## Closeout Requirements

Closeout evidence must be created only after implementation and validation
pass.

Closeout path:

```text
docs-site/src/content/docs/reports/closeouts/pm-render-pg-004-render-execution-graph-compiler-maturity/closeout.md
```

After closeout:

- archive `WR-043` with completed evidence;
- add the closeout path to WR-043 write scopes before archival;
- update `PM-RENDER-PG-004` evidence gates, completion audit, and completion
  quality;
- rerun roadmap, production, docs, planning, and goal validation.

## Perfectionist Closeout Audit

Expected completion quality is `bounded_contract`.

This slice can prove compiler contract maturity, typed diagnostics, prepared
preflight, lifetime/capability inspection, and submit-boundary preservation. It
should not claim `runtime_proven` unless closeout includes runtime evidence
that exercises the accepted compiler/preflight path through backend execution.
It must not claim `perfectionist_verified` while PM-005 through PM-008 remain
incomplete.

Known quality gaps expected at closeout unless later evidence proves otherwise:

- PM-RENDER-PG-005 still owns broad product-surface platform hardening.
- PM-RENDER-PG-006 still owns multi-surface presentation.
- PM-RENDER-PG-007 still owns render fragments and hot reload.
- PM-RENDER-PG-008 still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
- PM-004 does not move product truth or product policy into the renderer.
