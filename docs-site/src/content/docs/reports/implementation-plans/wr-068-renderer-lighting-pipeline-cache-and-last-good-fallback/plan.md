---
title: WR-068 Renderer Lighting Pipeline Cache And Last Good Fallback Implementation Contract
description: Design-first contract for renderer lighting inputs, pipeline specialization diagnostics, cache evidence, and last-good shader fallback.
status: active
owner: engine
layer: engine-runtime / renderer lighting pipeline cache
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-068 Renderer Lighting Pipeline Cache And Last Good Fallback Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-MESH-MATERIAL-003` and
`WR-068`. This row must add renderer-owned evidence for lighting inputs,
shader/pipeline specialization, pipeline cache diagnostics, shader failure
diagnostics, and last-good shader fallback behavior without moving product,
material, asset, model, scene, or fallback authority into the renderer.

This contract began as design-first planning to clear the deferred intake
questions and prepare WR-068 for roadmap application and promotion. After
promotion, it is the bounded implementation contract for the stack-selected
WR-068 work. Product code changes remain limited to the write scopes and stop
conditions below.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md`:
  shader specialization, pipeline cache policy, lighting inputs, and last-good
  shader fallback are renderer execution diagnostics, not material truth.
- `docs-site/src/content/docs/reports/closeouts/wr-067-renderer-mesh-material-shader-asset-handoff/closeout.md`:
  prepared material handoff inspection and pass-consumption evidence are
  complete and must be consumed rather than duplicated.
- `engine/src/plugins/render/renderer/render_flow/provenance.rs`:
  current owner for resolved shader material, `fallback_used`,
  `pipeline_identity`, and pass provenance material binding evidence.
- `engine/src/plugins/render/inspect/pass_provenance.rs`:
  exposes `fallback_used`, `pipeline_stats_key`, specialization hashes, and
  material binding evidence for runtime inspection.
- `engine/src/plugins/render/shader/registry.rs` and
  `engine/src/plugins/render/shader/types.rs`: current shader registry,
  hot-reload poll report, failure events, and source/revision lookup owner.
- `engine/src/plugins/render/pipelines/cache.rs`: current pipeline cache stats
  resource. Existing cutoff tests require it to remain stats-only and reject a
  legacy mutable key/cache authority path.
- `engine/src/plugins/render/inspect/material_handoff.rs`: WR-067 handoff
  inspection that WR-068 should extend or compose with shader/pipeline fallback
  diagnostics.

## Readiness

`task production:plan -- --milestone PM-RENDER-MESH-MATERIAL-003 --roadmap WR-068`
now reports:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-068-renderer-lighting-pipeline-cache-and-last-good-fallback/plan.md`.

Governance decision from PM-RENDER-MESH-MATERIAL-001 still applies:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer
  lighting execution inputs, shader/pipeline specialization diagnostics,
  pipeline cache statistics, shader registry events, and inspection DTOs.
- Source owners retain material, asset, model, scene, product freshness,
  product authority, rebuild policy, and fallback legality.
- ADR requirement: no ADR is required for renderer-owned diagnostic hardening.
  Stop for ADR if implementation adds persisted cross-domain ABI, changes
  fallback authority, or makes renderer pipeline cache state canonical source
  truth.
- Team Topologies ownership: complicated-subsystem renderer platform consuming
  stream-aligned product/material/asset/model producers.

## Promotion And Implementation Readiness

WR-068 was promoted after the intake proposal recorded:

- dependency on completed `WR-067`;
- accepted renderer handoff doctrine gate;
- completed PM-RENDER-MESH-MATERIAL-001 doctrine closeout;
- this active implementation contract as a design gate;
- focused write scopes for renderer shader registry, render-flow provenance,
  inspection, pipeline cache stats, tests, docs, intake, roadmap metadata, and
  production metadata;
- validation commands that prove shader failure diagnostics, fallback evidence,
  pipeline cache stats boundaries, and no submit-time fallback extraction.

Implementation is authorized only when the stack and single-track coordinators
select WR-068, `task production:plan` reports the row as `current_candidate`,
and this contract remains accurate for the owning modules.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render/shader
engine/src/plugins/render/pipelines
engine/src/plugins/render/renderer/render_flow
engine/src/plugins/render/inspect
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-lighting-pipeline-cache-and-las
docs-site/src/content/docs/reports/implementation-plans/wr-068-renderer-lighting-pipeline-cache-and-last-good-fallback/plan.md
docs-site/src/content/docs/reports/closeouts/wr-068-renderer-lighting-pipeline-cache-and-last-good-fallback/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/renderer/render_flow/provenance.rs`:
  preserve and harden shader material resolution, `fallback_used`, pipeline
  identity, shader revision, material scene bundle fallback detection, and pass
  provenance integration.
- `engine/src/plugins/render/inspect/pass_provenance.rs`:
  expose lighting/pipeline/fallback evidence needed by tools and closeout.
- `engine/src/plugins/render/shader/registry.rs` and
  `engine/src/plugins/render/shader/types.rs`: expose shader load/reload
  failure events and prior valid revision evidence without becoming material
  source truth.
- `engine/src/plugins/render/pipelines/cache.rs`: retain stats-only pipeline
  cache evidence; do not reintroduce mutable pipeline-key authority rejected by
  cutoff tests.
- `engine/src/plugins/render/inspect/material_handoff.rs`: compose with
  WR-067 handoff inspection only when shader fallback diagnostics require
  material scene bundle context.
- `engine/tests/render_flow_fragments.rs`,
  `engine/tests/render_runtime_inspect.rs`, `engine/tests/render_cutoff_guard.rs`,
  or a new `engine/tests/render_pipeline_fallback.rs`: focused tests for
  shader failure, prior-valid preservation, fallback visibility, and pipeline
  cache stats boundaries.

## Required Contracts

WR-068 must provide or verify explicit typed evidence for:

- lighting inputs and debug labels as renderer execution inputs, not scene or
  material source truth;
- shader specialization keys and material/pipeline identities that name source
  lineage without becoming edit authority;
- pipeline cache hit/miss/failure statistics as inspection evidence only;
- shader registry failure diagnostics that preserve the last-good active shader
  or package where policy allows reuse;
- pass provenance that reports `fallback_used` and pipeline identity when a
  material scene bundle falls back to an asset shader;
- fail-closed diagnostics when a material feature pass requires a generated
  scene shader but only fallback data is available.

## Critical Review Decisions

- Source truth remains with material, asset, model, scene, product, and shader
  package owners. Renderer data added by WR-068 is only projection,
  specialization identity, cache statistics, pass provenance, or prior-valid
  runtime evidence.
- The source-to-runtime chain is material/scene bundle -> resolved shader
  material -> render-flow provenance -> pass provenance inspection -> renderer
  diagnostics. WR-068 must fail closed if that chain stops at a descriptor,
  status panel, fallback-only path, or unconsumed prepared contract.
- Owners are `renderer/render_flow/provenance.rs` for pass evidence,
  `inspect/pass_provenance.rs` and WR-068 inspection DTOs for public
  diagnostics, `shader/registry.rs` and `shader/types.rs` for shader
  load/reload event evidence, and `pipelines/cache.rs` for stats-only cache
  reporting.
- Typed contracts must replace ad hoc string evidence for fallback severity,
  pipeline cache counts, shader prior-valid state, and material pass fallback
  requirements.
- A material pass that requires generated scene shader evidence must reject an
  asset-shader fallback. Prior-valid reuse can be reported only as renderer
  execution evidence; it cannot decide product freshness or fallback legality.
- Guard tests must cover accepted fallback reporting, fail-closed generated
  material fallback, missing pipeline statistics, shader reload failure
  diagnostics, and the existing cutoff rule that keeps pipeline cache state
  stats-only.
- WR-068 closeout is `bounded_contract`; runtime visual proof, benchmarks, and
  mesh/material production evidence remain WR-069 scope.

## Non Goals

- No runtime production evidence, benchmark report, or `runtime_proven` claim;
  WR-069 owns production evidence.
- No general renderer-owned material fallback policy.
- No pipeline cache redesign that stores source truth or replaces shader
  registry/package ownership.
- No material graph, asset catalog, scene assignment, or model source edits.
- No live ECS extraction during submit.

## Implementation Steps

1. Inspect current shader registry events, render-flow provenance, pass
   provenance inspection, pipeline cache stats, material handoff inspection, and
   cutoff tests before editing.
2. Define the minimal inspection DTO or extension that reports shader fallback,
   prior-valid availability, pipeline cache stats, specialization identity, and
   lighting input evidence.
3. Preserve the existing stats-only pipeline cache boundary. Add diagnostics or
   inspection fields around it rather than reintroducing key ownership.
4. Add fail-closed tests for generated material scene bundle fallback, missing
   shader revision, shader registry failure events, and pipeline cache stats
   reporting.
5. Update renderer public docs with the preferred inspection path and explicit
   non-ownership of product/material fallback policy.
6. Close out WR-068 only after focused tests and planning validators pass.

## Acceptance Criteria

- Shader and pipeline fallback evidence is visible through renderer inspection.
- Last-good reuse is reported as prior-valid renderer execution evidence, not
  as product freshness or fallback legality.
- Pipeline cache stats remain diagnostics-only and do not become a canonical
  source of shader/material/pipeline truth.
- Material scene bundle fallback cannot silently satisfy a generated-material
  pass that requires exact source-backed shader evidence.
- Tests cover both accepted fallback reporting and fail-closed missing evidence.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_pipeline
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow_fragments
cargo test -p engine render_cutoff_guard
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If exact test filters differ after inspection, the closeout must name the
actual focused tests and explain why they cover the same contracts.

## Stop Conditions

Stop before implementation if:

- WR-068 remains `blocked_deferred` or lacks accepted design, dependency,
  write-scope, validation, or contract gates;
- implementation would make renderer pipeline cache state material, asset,
  model, scene, product, or fallback truth;
- a new persisted cross-domain ABI is required without ADR review;
- fallback evidence can only be proven through descriptor-only,
  status-panel-only, fallback-only, or unconsumed-contract paths;
- source files drift enough that this contract no longer describes the owning
  modules.

## Closeout Requirements

The closeout must include:

- exact changed files and owning modules;
- governance evidence and ADR decision;
- focused tests and command output summaries;
- docs, roadmap, production, and planning validation results;
- evidence that shader fallback and pipeline cache diagnostics are visible
  through renderer inspection without moving fallback authority into renderer
  code;
- explicit known quality gaps for WR-069 and final perfectionist verification.

## Perfectionist Closeout Audit

Expected completion quality for WR-068: `bounded_contract`.

WR-068 can prove shader/pipeline/fallback diagnostics and lighting input
inspection, but it must not claim `runtime_proven` until WR-069 supplies
runtime examples, benchmark evidence, production reports, and visible
mesh/material proof.

Known quality gaps that must remain visible at WR-068 closeout:

- mesh/material runtime production evidence remains WR-069 scope;
- final `perfectionist_verified` remains blocked until
  `PT-RENDER-PERFECTION` audits the completed renderer stack.
