---
title: WR-010 PM-RENDER-PG-007 Render Fragment And Hot Reload Plan
description: Promotion and implementation-readiness contract for the PM-RENDER-PG-007 render fragment and hot reload slice.
status: active
owner: engine
layer: engine-runtime / render composition
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/render-fragment-data-driven-maturity-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
  - ../../../design/accepted/product-surface-platform-hardening-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../../engine/plugins/render/docs/roadmap.md
  - ../../../engine/roadmaps/fully-featured-renderer-roadmap.md
---

# WR-010 PM-RENDER-PG-007 Render Fragment And Hot Reload Plan

## Goal

Promote and implement `PM-RENDER-PG-007` as one bounded render fragment and
hot reload slice.

The slice turns authored fragment descriptions into validated, mergeable
`RenderFlow` contributions without creating a second execution graph or moving
product policy into the renderer:

```text
fragment package source
  -> schema and namespace validation
  -> fragment registry revision
  -> merge plan with provenance
  -> normal RenderFlow definition
  -> normal compile_flow_plan_checked validation
  -> prepared-frame execution through existing render flow runtime
  -> fragment-aware inspection and reload diagnostics
```

The render plugin owns fragment description contracts, validation, merge,
registry state, hot reload status, last-good revision preservation, and
inspection. Product domains and product jobs still own product truth, product
selection, freshness, authority class, fallback legality, rebuild policy,
material truth, source lineage, and residency intent.

## Source Of Truth

- Production milestone: `PM-RENDER-PG-007`.
- Bounded implementation row: `WR-010`.
- Accepted fragment design:
  `docs-site/src/content/docs/design/accepted/render-fragment-data-driven-maturity-design.md`.
- Product/render boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- Compiler prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-004-render-execution-graph-compiler-maturity/closeout.md`.
- Product-surface prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-005-product-surface-platform-hardening/closeout.md`.
- `WR-003` remains support-only context. It is not the PM-007 implementation
  row.

## Readiness

`task production:plan -- --milestone PM-RENDER-PG-007 --roadmap WR-010`
reported after design acceptance and exact promotion metadata repair:

- milestone state: `ready_next`;
- WR state: `ready_next`;
- WR blocker: `B2`;
- dependency `WR-003:support_only`;
- next action: `write_promotion_contract`;
- promotion preflight: `promotable`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "PM-RENDER-PG-007 render fragments and hot reload" --scope "engine/src/plugins/render/composition, engine/src/plugins/render/graph, engine/src/plugins/render/inspect, engine/examples, docs-site render roadmap"
```

No ADR is required while fragments remain engine render descriptions that merge
into existing `RenderFlow` and do not become cross-domain product truth,
project asset truth, or backend allocation policy. Add an ADR before
implementation if fragments become a cross-domain asset/package contract,
introduce a new execution graph outside `RenderFlow`, or change reload
ownership outside the render plugin boundary.

Promotion command, after this contract is linked and validation passes:

```text
task roadmap:promote -- --id WR-010 --state current_candidate --evidence "Accepted PM-RENDER-PG-007 fragment design and active promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-010-render-fragment-and-data-driven-maturity-r10/plan.md"
```

Do not promote if validation fails, if another current candidate creates a
write-scope conflict, or if source files changed enough that
`task ai:goal -- --track PT-RENDER-PG` must be rerun first.

## Implementation Scope

Allowed write scopes:

- `engine/src/plugins/render`;
- `engine/tests`;
- `engine/examples`;
- `docs-site/src/content/docs/engine/reference/plugins/render`;
- `docs-site/src/content/docs/engine/plugins/render/docs/roadmap.md`;
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`;
- `docs-site/src/content/docs/design/accepted/render-fragment-data-driven-maturity-design.md`;
- `docs-site/src/content/docs/reports/implementation-plans/wr-010-render-fragment-and-data-driven-maturity-r10/plan.md`;
- `docs-site/src/content/docs/reports/closeouts`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- generated roadmap docs and diagrams;
- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- generated production-track docs and diagrams.

Expected implementation modules or nearby owners:

```text
engine/src/plugins/render/composition/fragments.rs
engine/src/plugins/render/composition/fragment_registry.rs
engine/src/plugins/render/composition/fragment_validation.rs
engine/src/plugins/render/composition/hot_reload.rs
engine/src/plugins/render/composition/integration.rs
engine/src/plugins/render/composition/mod.rs
engine/src/plugins/render/graph/merge.rs
engine/src/plugins/render/graph/validation.rs
engine/src/plugins/render/graph/diagnostics.rs
engine/src/plugins/render/graph/planning.rs
engine/src/plugins/render/inspect/graph_dump.rs
engine/src/plugins/render/inspect/pass_provenance.rs
engine/src/plugins/render/inspect/report.rs
engine/src/plugins/render/mod.rs
engine/tests/render_flow_fragments.rs
engine/tests/render_flow_v2.rs
engine/tests/render_runtime_inspect.rs
engine/examples/render_fragment_compositor
```

Use nearby module names if implementation shows a better local fit, but keep
the composition, graph, reload, and inspection responsibilities separated by
subdomain. Do not add catch-all helper files.

## Required Contracts

The implementation must add or refine typed, inspectable contracts for:

- `RenderFragmentId`, `RenderFragmentPackageId`, and
  `RenderFragmentNamespace`;
- fragment package metadata including package id, schema version, source path,
  declared namespace, source revision, active revision, and last-good revision;
- fragment resource, pass, dependency, alias, capability, uniform, and feature
  contribution descriptors;
- fragment diagnostics with package id, fragment id, namespace, source path,
  revision, diagnostic kind, severity, and message;
- a fragment registry that keeps accepted fragments deterministic and
  preserves last-good active revisions on invalid reloads;
- a validation pass that rejects invalid schema versions, duplicate ids,
  namespace collisions, missing resource/pass references, alias mismatches,
  unsupported capabilities, and illegal override attempts;
- a merge plan that produces normal `RenderFlow` definitions or builder patches
  and records provenance for generated resources, passes, dependencies, aliases,
  and contribution slots;
- fragment-aware inspection that reports active/stale/failed revisions, merge
  provenance, conflicts, and reload diagnostics.

Fragments are authored descriptions. They must not allocate WGPU resources,
publish prepared-frame requests, choose product selections, choose residency,
or bypass `compile_flow_plan_checked`.

## Implementation Steps

1. Add fragment identity, namespace, package, descriptor, provenance, and
   diagnostic types under `engine/src/plugins/render/composition/fragments.rs`
   and export them through `composition/mod.rs` and the render module surface.
2. Add fragment validation in
   `engine/src/plugins/render/composition/fragment_validation.rs`, using typed
   diagnostics and existing render graph vocabulary where possible.
3. Add a fragment registry in
   `engine/src/plugins/render/composition/fragment_registry.rs` that tracks
   active, stale, failed, disabled, source revision, and last-good revision
   state with deterministic iteration.
4. Add merge planning in `engine/src/plugins/render/graph/merge.rs` so accepted
   fragments merge into normal `RenderFlow` definitions and then pass existing
   compiler validation.
5. Integrate registry-to-flow compilation in
   `engine/src/plugins/render/composition/integration.rs` without replacing the
   existing `RenderFlowRegistryResource` execution path.
6. Add hot reload support in
   `engine/src/plugins/render/composition/hot_reload.rs` using existing reload
   hooks if present; if no suitable hook exists, implement a render-local
   revision application API and leave file watching to later asset/editor work.
7. Extend inspection in `graph_dump.rs`, `pass_provenance.rs`, and `report.rs`
   to show fragment ids, package ids, source paths, revisions, merge
   provenance, and conflicts.
8. Add `engine/tests/render_flow_fragments.rs` for descriptor validation,
   namespace collision rejection, merge provenance, compile validation, reload
   last-good behavior, and diagnostics.
9. Extend existing render-flow and runtime-inspection tests only where they
   prove fragment output still uses normal compiled execution and prepared
   frame inspection.
10. Add one bounded `engine/examples/render_fragment_compositor` proof that a
    fragment-driven compositor flow runs through the normal render-flow
    execution path without a custom executor.
11. Update render roadmap/reference docs to describe the accepted fragment
    contract as implemented and name any remaining non-PM-007 work honestly.
12. After validation passes, create PM-007 closeout evidence and only then
    update `PM-RENDER-PG-007` completion metadata.

## Explicit Non-Goals

Do not implement:

- PM-008 production-readiness inspection, final examples, performance budgets,
  capture/replay policy, or broad renderer hardening;
- editor asset-pipeline authoring UI, package catalog ownership, migration UI,
  or project save/load workflow;
- native multi-window or multi-surface presentation work from PM-006;
- product truth, product selection, source lineage, freshness, authority,
  fallback legality, rebuild policy, material truth, drawing truth, or
  residency policy;
- a renderer-owned product policy, product fallback, or product selection
  shortcut;
- a second execution graph or custom fragment executor outside normal
  `RenderFlow` compilation;
- direct backend resource allocation from fragment descriptions;
- broad product-surface hardening beyond already completed PM-005 contracts.

## Acceptance Criteria

- `WR-010` is the PM-007 implementation row and is not reused for PM-004 or
  PM-008 work.
- Fragment descriptions, package metadata, namespaces, diagnostics, and
  provenance are typed and inspectable.
- Invalid fragments cannot affect active rendering.
- Valid fragments merge into normal `RenderFlow` definitions and compile
  through `compile_flow_plan_checked`.
- Hot reload preserves the last-good active revision on invalid changes.
- Inspection exposes fragment ids, package ids, source paths, revisions, merge
  provenance, and conflicts.
- At least one fragment-driven compositor proof uses the normal render-flow
  runtime path.
- No product truth, product policy, backend allocation policy, or editor asset
  ownership moves into renderer fragment code.

## Validation

Required focused tests:

```text
cargo test -p engine --test render_flow_fragments
cargo test -p engine --test render_flow_v2
cargo test -p engine --test render_runtime_inspect
cargo test -p engine --example render_fragment_compositor
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

- `WR-010` cannot be promoted legally;
- production planning stops at a gate other than implementation;
- implementation requires renderer-owned product truth, selection, freshness,
  authority, fallback legality, rebuild policy, material truth, or residency
  policy;
- fragments need direct backend resource allocation to work;
- fragments need a custom executor instead of normal `RenderFlow` compilation;
- editor asset/package ownership is required to prove the bounded engine slice;
- hot reload cannot preserve last-good active revision on invalid changes;
- inspection cannot expose fragment provenance and diagnostics with stable ids;
- focused validation fails and the cause is outside the PM-007 scope.

## Closeout Requirements

After implementation and validation pass:

- create PM-007 closeout evidence under
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-007-render-fragments-and-hot-reload/closeout.md`;
- update `PM-RENDER-PG-007` evidence gates and completion audit;
- set PM-007 `completion_quality: bounded_contract` unless runtime example
  evidence honestly supports a higher tier;
- record known PM-008 quality gaps for production-readiness inspection, capture,
  performance budgets, and broad examples;
- archive or close `WR-010` only with completed evidence and matching roadmap
  validation;
- rerun `task ai:goal -- --track PT-RENDER-PG` before starting PM-008.

## Perfectionist Closeout Audit

Expected completion quality is `bounded_contract`.

`runtime_proven` is only allowed if the closeout includes source-backed runtime
evidence that a fragment-driven compositor flow executes through the normal
render-flow path. `perfectionist_verified` is out of scope for PM-007 because
PM-008 still owns production-readiness inspection, budgets, capture/replay
policy, and final renderer examples.

The closeout must explicitly list any remaining quality gaps instead of
claiming production readiness by implication. Guard tests must prevent
descriptor-only, merge-report-only, status-panel-only, fallback-only, and
unconsumed-fragment completion claims.
