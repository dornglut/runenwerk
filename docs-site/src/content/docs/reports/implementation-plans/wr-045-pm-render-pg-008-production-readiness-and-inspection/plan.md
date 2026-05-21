---
title: WR-045 PM-RENDER-PG-008 Production Readiness And Inspection Plan
description: Promotion and implementation-readiness contract for PM-RENDER-PG-008 renderer production readiness hardening.
status: active
owner: engine
layer: engine-runtime / render production readiness
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/product-surface-platform-hardening-design.md
  - ../../../design/accepted/render-fragment-data-driven-maturity-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-045 PM-RENDER-PG-008 Production Readiness And Inspection Plan

## Goal

Prepare and implement the bounded `PM-RENDER-PG-008` production-readiness slice
for renderer diagnostics, inspection, budgets, examples, capture/replay policy,
and closeout evidence.

This contract is promotion evidence first. It permits `WR-045` to become the
current candidate only if the roadmap promotion gate remains clean. It does not
authorize product code before promotion succeeds and the goal gate is rerun.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-PG-008`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml::WR-045`.
- Accepted design:
  `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`.
- Boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- Dependency closeouts:
  - `docs-site/src/content/docs/reports/closeouts/pm-render-pg-005-product-surface-platform-hardening/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-render-pg-006-multi-surface-presentation/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-render-pg-007-render-fragments-and-hot-reload/closeout.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-RENDER-PG-008 --roadmap WR-045`
reported:

- milestone state: `ready_next`;
- roadmap state: `ready_next`;
- dependencies: `WR-044:completed`, `WR-009:completed`, `WR-010:completed`;
- promotion preflight: `promotable`;
- suggested command:
  `task roadmap:promote -- --id WR-045 --state current_candidate --evidence "<accepted evidence>"`.

Promote only with evidence that names this active contract and the accepted
PM-008 design. If promotion fails, repair only the exact metadata named by the
failed command, run `task roadmap:switch-current` if instructed, or stop and
report. Do not inspect or repair adjacent completed WR evidence.

## Ownership And Boundaries

DDD bounded context owner:

- `engine/src/plugins/render` owns renderer readiness inspection,
  backend-neutral render execution diagnostics, render capture/replay policy
  over renderer-derived state, render budget reports, examples, and render
  closeout evidence.

Translation boundaries:

- Product domains and Product Jobs own product truth, dependency truth,
  freshness, authority, fallback legality, rebuild policy, residency intent,
  source lineage, and product diagnostics.
- App/product-surface producers own prepared render inputs and producer status.
- Render inspection aggregates already prepared renderer-facing DTOs and
  backend-derived execution evidence only.
- Backend handles stay inside backend/runtime code. Inspection exposes read-only
  DTOs, artifact paths, terminal reasons, and timing summaries.

Team Topologies ownership:

- Complicated-subsystem render owner enabling stream-aligned product, editor,
  drawing, and future world/product teams.

ADR requirement:

- No ADR is required while implementation remains an engine-runtime hardening
  slice over accepted render contracts.
- Create an ADR or stop if capture/replay becomes a persisted cross-domain ABI,
  source-truth ownership changes, or a reusable contract is needed outside the
  render plugin boundary.

## Implementation Scope

Allowed write scopes after promotion:

```text
engine/src/plugins/render
engine/tests
engine/examples
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/engine/plugins/render
docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md
docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md
docs-site/src/content/docs/reports/implementation-plans/wr-045-pm-render-pg-008-production-readiness-and-inspection/plan.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-21-pm-render-pg-008-production-readiness-an
docs-site/src/content/docs/reports/closeouts/pm-render-pg-008-production-readiness-and-inspection/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/roadmap-decision-register.md
docs-site/src/content/docs/workspace/design-implementation-triage.md
docs-site/src/content/docs/workspace/diagrams/current-roadmap-candidates.puml
docs-site/src/content/docs/workspace/diagrams/value-weighted-dependency-roadmap.puml
docs-site/src/content/docs/workspace/production-tracks.yaml
docs-site/src/content/docs/workspace/production-track-index.md
docs-site/src/content/docs/workspace/production-milestone-register.md
docs-site/src/content/docs/workspace/diagrams/production-track-roadmap.puml
```

Future implementation should prefer additive modules near existing inspection
contracts:

- `engine/src/plugins/render/inspect/readiness.rs` for readiness report DTOs;
- `engine/src/plugins/render/inspect/budgets.rs` for renderer budget reports;
- existing `capture.rs`, `artifacts.rs`, and `report.rs` for capture/replay
  manifest policy if those modules remain the clearest ownership location;
- existing `timings.rs`, `prepared_frame.rs`, `plan.rs`, `graph_dump.rs`, and
  `pass_provenance.rs` as source reports rather than replacement contracts.

## Non-Goals

Do not implement:

- product truth, product selection, freshness, fallback legality, authority,
  rebuild policy, dependency truth, material truth, drawing truth, or residency
  policy;
- material graph lowering, SDF/world renderer features, asset catalog
  ownership, fragment authoring UI, project save/load, or product-family
  semantics;
- new native multi-window behavior beyond readiness evidence over existing
  PM-006 contracts;
- renderer-private handle exposure in product, app, docs, examples, or
  inspection DTOs;
- a persisted capture/replay ABI without a separate ADR or accepted design
  update.

## Implementation Steps

1. Inspect existing render inspection, capture, timing, fragment, graph,
   prepared-frame, dynamic-target, and multi-surface tests before editing.
2. Add a typed renderer readiness report that aggregates existing reports by
   frame, surface, producer, flow, pass, view, invocation, target, fragment,
   capture, and budget identity.
3. Add typed readiness diagnostics for missing evidence, failed preflight,
   failed capture, failed replay/dry-run, budget overflow, incomplete fragment
   reload evidence, surface identity mismatch, and unsupported backend
   capability.
4. Add capture/replay manifest policy over renderer-derived artifacts and
   prepared renderer inputs. Replays must fail closed when inputs,
   capabilities, artifacts, formats, or target identities are missing.
5. Add renderer budget reports for prepare/submit timing, graph/preflight,
   dynamic targets/uploads, capture/readback, fragment validation/reload, and
   multi-surface evidence. Budget reports diagnose renderer execution only.
6. Add or harden examples proving the normal public contract path for product
   surfaces, compiler/preflight diagnostics, fragment inspection,
   capture/readback or capture-manifest policy, and multi-surface identity.
7. Update render public API docs, usage docs, and roadmaps so product teams can
   understand readiness inspection without reading backend internals first.
8. Run focused validation, then workflow validation.
9. Create closeout evidence only after implementation validation passes.
10. Rerun `task ai:goal -- --track PT-RENDER-PG` after closeout and metadata
    updates.

## Diagnostics And Fitness Functions

The implementation is not complete unless focused tests prove:

- readiness inspection references existing typed source reports instead of
  string-only summaries;
- capture/replay manifest validation fails closed for missing artifacts,
  unsupported formats, and missing capability profiles;
- budget reports produce typed over-budget diagnostics without deciding product
  rebuild or fallback policy;
- fragment, graph/preflight, product-surface, prepared-frame, capture, timing,
  and multi-surface evidence can be inspected from one readiness surface;
- examples use public render APIs and do not receive backend-private handles.

## Validation

Required focused commands:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_dynamic_targets
cargo test -p engine --test render_flow_v2
cargo test -p engine --test render_flow_fragments
cargo test -p engine --test render_multi_surface
cargo test -p engine --example render_fragment_compositor
cargo test -p engine --example render_readiness_inspection
```

Required workflow commands:

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

Add narrower commands if implementation introduces new test targets, examples,
or docs.

## Stop Conditions

Stop immediately if:

- promotion preflight changes from `promotable`;
- ownership requires product/domain policy in renderer code;
- a capture/replay decision becomes a durable cross-domain ABI without ADR
  coverage;
- implementation needs files outside WR-045 write scopes;
- a validation command fails and the failure is not understood;
- closeout evidence would need to claim runtime proof that did not run;
- source files changed enough that `task ai:goal` must be rerun before
  continuing.

## Closeout Requirements

Closeout must be created at:

```text
docs-site/src/content/docs/reports/closeouts/pm-render-pg-008-production-readiness-and-inspection/closeout.md
```

Closeout must include:

- implementation evidence by file/module;
- focused validation output summary;
- workflow validation summary;
- capture/replay and budget evidence;
- example evidence;
- known host/GPU/multi-surface limitations;
- honest completion quality.

Expected completion quality is `bounded_contract` unless host-backed
capture/replay, budget, and multi-surface evidence are actually proven in this
environment. Do not claim `runtime_proven` or `perfectionist_verified` without
matching evidence.

## Perfectionist Closeout Audit

`perfectionist_verified` is not expected for the first WR-045 slice.

It may only be claimed later if:

- a completed audit path exists;
- the known-quality-gap list is empty;
- host-backed GPU capture/replay and multi-surface evidence passed;
- examples prove the public production inspection path without private handles;
- roadmap archive and production-track completion metadata agree.
