---
title: PM-RENDER-PG-007 Render Fragments And Hot Reload Closeout
description: Closeout evidence for the bounded render fragment, validation, merge provenance, and last-good reload contract slice.
status: completed
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

# PM-RENDER-PG-007 Render Fragments And Hot Reload Closeout

## Result

`PM-RENDER-PG-007` completed as a bounded render-fragment contract slice.

The implementation adds typed fragment package, fragment id, namespace,
descriptor, diagnostic, registry, reload, merge provenance, and inspection
contracts. Valid fragment packages merge into normal `RenderFlow` definitions
and pass `compile_flow_plan_checked(...)`; invalid packages produce typed
diagnostics before they can affect active rendering. Reload state preserves the
last-good active package revision when a later revision fails validation.

## Evidence

- `engine/src/plugins/render/composition/fragments.rs` owns fragment package,
  id, namespace, resource/pass descriptor, diagnostic, revision, and provenance
  types.
- `engine/src/plugins/render/composition/fragment_validation.rs` validates
  schema version, package/fragment ids, namespace consistency, duplicate local
  labels, missing local resources, missing local pass dependencies, and bounded
  pass shape before merge.
- `engine/src/plugins/render/composition/fragment_registry.rs` owns the
  render-fragment registry resource, active/failed/disabled package records,
  revision tracking, diagnostics, and last-good active package preservation.
- `engine/src/plugins/render/composition/hot_reload.rs` provides the render
  fragment reload request/apply path used by registry-backed reload tests.
- `engine/src/plugins/render/graph/merge.rs` namespace-qualifies local
  fragment labels, merges descriptors into normal `RenderFlow`, validates the
  merged flow with `compile_flow_plan_checked(...)`, and returns typed merge
  diagnostics instead of backend/runtime shortcuts.
- `engine/src/plugins/render/inspect/graph_dump.rs` exposes fragment merge
  report inspection with package id, source path, source revision, generated
  flow id, diagnostics, and provenance lines.
- `engine/src/plugins/render/inspect/pass_provenance.rs` exposes
  fragment-pass provenance records for tooling.
- `engine/src/plugins/render/plugin.rs` initializes
  `RenderFragmentRegistryResource` as an engine render resource.
- `engine/examples/render_fragment_compositor.rs` proves a fragment-driven
  compositor flow builds through the normal render-flow path.

## Validation

Passed:

```text
cargo test -p engine --test render_flow_fragments
cargo test -p engine --test render_flow_v2
cargo test -p engine --test render_runtime_inspect
cargo test -p engine --example render_fragment_compositor
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

`completion_quality: bounded_contract`

This closeout does not claim `runtime_proven` or `perfectionist_verified`.
The deterministic fragment contract, merge path, inspection path, and example
compile proof are validated, but PM-008 still owns production-readiness
inspection, capture/replay policy, performance budgets, and final renderer
example coverage.

## Known Gaps

- The hot reload contract preserves last-good revisions through an explicit
  render-fragment apply path; project asset watching, package catalog ownership,
  migration UI, and save/load workflow remain asset/editor-owned follow-up work.
- The fragment compositor example proves normal `RenderFlow` compilation, not a
  host-backed GPU pixel proof.
- PM-RENDER-PG-008 still owns production readiness, capture/replay policy,
  performance budgets, broad examples, and final inspection hardening.
- Product truth, product selection, freshness, authority, fallback legality,
  rebuild policy, material truth, drawing truth, and residency policy remain
  outside renderer fragment ownership.
