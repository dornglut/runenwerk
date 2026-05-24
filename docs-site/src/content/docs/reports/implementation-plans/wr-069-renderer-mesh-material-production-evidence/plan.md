---
title: WR-069 Renderer Mesh Material Production Evidence Implementation Contract
description: Design-first contract for renderer mesh/material runtime evidence, examples, benchmark artifacts, docs, and production closeout.
status: active
owner: engine
layer: engine-runtime / renderer mesh material production evidence
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-069 Renderer Mesh Material Production Evidence Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-MESH-MATERIAL-004` and
`WR-069`. This row must close the mesh/material/lighting renderer handoff with
runtime evidence, examples, focused tests, benchmark or replay artifacts, and
docs that prove the chain built by WR-067 and WR-068 without moving material,
asset, model, scene, product, shader source, or fallback authority into the
renderer.

This contract began as design-first planning to clear the deferred intake
questions and prepare WR-069 for roadmap application and promotion. After
promotion, it is the bounded implementation contract for the stack-selected
WR-069 work. Product code changes remain limited to the write scopes and stop
conditions below.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md`:
  accepted doctrine for renderer mesh/material/lighting, shader, pipeline
  cache, fallback, asset handoff, and production evidence boundaries.
- `docs-site/src/content/docs/reports/closeouts/wr-067-renderer-mesh-material-shader-asset-handoff/closeout.md`:
  completed prepared material handoff inspection and fail-closed material pass
  evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-068-renderer-lighting-pipeline-cache-and-last-good-fallback/closeout.md`:
  completed pipeline/fallback inspection and prior-valid shader failure
  evidence.
- `engine/src/plugins/render/inspect/material_handoff.rs` and
  `engine/src/plugins/render/inspect/pipeline_fallback.rs`: renderer-owned
  inspection contracts that WR-069 must consume for production evidence.
- `engine/examples`, `engine/benches`, and renderer artifact folders: expected
  owners for executable runtime evidence, benchmark runners, raw artifacts, and
  human-readable production reports.

## Readiness

`task production:plan -- --milestone PM-RENDER-MESH-MATERIAL-004 --roadmap WR-069`
now reports:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-069-renderer-mesh-material-production-evidence/plan.md`.

Governance decisions from PM-RENDER-MESH-MATERIAL-001 still apply:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer runtime
  evidence, prepared-frame consumption, pass provenance, inspection reports,
  benchmark runners, and replay/visual artifact references.
- Source owners retain material authoring, asset catalog, model/mesh source,
  scene assignment, product freshness, fallback legality, and rebuild policy.
- ADR requirement: no ADR is required for consuming existing renderer-owned
  inspection and runtime evidence. Stop for ADR if implementation adds a
  persisted cross-domain ABI, changes fallback authority, or makes renderer
  artifacts source truth.
- Team Topologies ownership: complicated-subsystem renderer platform consuming
  stream-aligned product/material/asset/model producers.

## Promotion And Implementation Readiness

WR-069 was promoted after the intake proposal recorded:

- dependencies on completed `WR-067` and `WR-068`;
- accepted renderer handoff doctrine gate;
- completed PM-RENDER-MESH-MATERIAL-001 doctrine closeout;
- completed WR-067 and WR-068 closeout evidence;
- this active implementation contract as a design gate;
- focused write scopes for renderer examples, benchmarks, inspection docs,
  runtime evidence artifacts, roadmap metadata, production metadata, and
  closeout evidence;
- validation commands that prove runtime mesh/material evidence, inspection
  consumption, benchmark/report generation, docs, roadmap, production, and
  planning gates.

Implementation is authorized only when the stack and single-track coordinators
select WR-069, `task production:plan` reports the row as `current_candidate`,
and this contract remains accurate for the owning modules.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/examples
engine/benches
engine/benchmark-artifacts/render-mesh-material-production-evidence
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/benchmarks/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-23-renderer-mesh-material-production-evidence
docs-site/src/content/docs/reports/implementation-plans/wr-069-renderer-mesh-material-production-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-069-renderer-mesh-material-production-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/examples`: executable mesh/material production evidence that exercises
  prepared material handoff, pipeline/fallback inspection, and visible renderer
  product proof.
- `engine/benches`: benchmark runner for the same renderer path, with raw
  outputs kept in dedicated artifact folders and prose reports in docs.
- `engine/src/plugins/render/inspect`: consume existing WR-067/WR-068
  inspection contracts; add only missing aggregation needed for production
  evidence.
- `engine/tests`: anti-drift guards that prevent descriptor-only,
  status-panel-only, fallback-only, unconsumed-inspection, or benchmark-only
  completion claims.
- `docs-site/src/content/docs/engine/reference/plugins/render`: public usage
  and evidence documentation for the preferred mesh/material production path.

## Required Contracts

WR-069 must provide or verify explicit typed evidence for:

- runtime consumption of prepared material handoff and pipeline/fallback
  inspection reports;
- a visible mesh/material or material-scene renderer product evidence reference
  with deterministic artifact identity;
- CPU timing and, where available, GPU timing or explicit unsupported/readback
  diagnostics;
- benchmark command, raw artifact path, and human-readable report path;
- fail-closed diagnostics when evidence is descriptor-only, unconsumed,
  fallback-only, missing material handoff inspection, missing pipeline/fallback
  inspection, or missing artifact references.

## Critical Review Decisions

- Source truth remains with material, asset, model, scene, product, shader
  package, and fallback policy owners. WR-069 evidence is runtime product
  evidence, inspection aggregation, benchmark/report metadata, and artifact
  references only.
- The complete source-to-runtime chain is prepared material handoff -> material
  pass provenance -> pipeline/fallback inspection -> runtime visual evidence ->
  timing and benchmark/report evidence -> closeout. WR-069 must fail closed if
  the chain stops at descriptors, status panels, fallback-only shader use,
  benchmark-only numbers, or unconsumed inspection reports.
- Owners are `engine/src/plugins/render/inspect` for report aggregation,
  `engine/examples` for executable runtime evidence, `engine/benches` for
  benchmark runners, `engine/tests` for anti-drift guards, and docs closeout
  folders for artifact/report references.
- Typed contracts must name material handoff readiness, pipeline/fallback
  readiness, runtime visual artifact references, timing evidence, benchmark
  command, raw artifact path, and human-readable report path.
- Forbidden fallbacks include material-pass shader fallback, missing generated
  shader revision, missing material specialization evidence, descriptor-only
  visual proof, and evidence that never consumes WR-067 or WR-068 inspection
  reports.
- Guard tests must prevent descriptor-only, prepared-data-only,
  status-panel-only, fallback-only, benchmark-only, and unconsumed-contract
  completion claims.
- Completion quality target is `runtime_proven`. `perfectionist_verified`
  remains reserved for `PT-RENDER-PERFECTION`.

## Non Goals

- No new material authoring, asset catalog, model, mesh, scene, or product
  source contracts.
- No new fallback legality policy or product freshness decision in renderer
  code.
- No pipeline cache redesign beyond consuming existing diagnostics.
- No claim of `perfectionist_verified`; final no-gap audit belongs to
  `PT-RENDER-PERFECTION`.

## Implementation Steps

1. Inspect the WR-067 and WR-068 inspection APIs, existing render runtime
   evidence examples, benchmark conventions, and renderer benchmark reports.
2. Define the minimal production evidence DTO or example output that aggregates
   material handoff inspection, pipeline/fallback inspection, timing evidence,
   visual artifact references, benchmark command, and artifact/report paths.
3. Add executable example and focused tests proving the production evidence
   consumes renderer inspection reports instead of duplicating source truth.
4. Add or extend benchmark/report generation with stable artifact locations.
5. Update renderer public docs with the preferred production evidence path.
6. Close out WR-069 only after focused runtime tests, examples, benchmark or
   artifact generation, docs, roadmap, production, and planning validators pass.

## Acceptance Criteria

- Runtime evidence proves mesh/material renderer handoff consumption through
  the same inspection contracts exposed by WR-067 and WR-068.
- Visual proof, timing evidence, benchmark command, artifact paths, and report
  paths are explicit and fail closed when missing.
- Production evidence does not move product, material, asset, model, scene,
  shader source, fallback, or rebuild authority into renderer code.
- Known gaps after WR-069 are limited to cross-track perfectionist audit work.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_mesh_material
cargo test -p engine render_pipeline
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow_fragments
cargo test -p engine --example render_mesh_material_production_evidence
cargo bench -p engine --bench render_flow_planning
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If exact test, example, or benchmark names differ after inspection, the
closeout must name the actual focused commands and explain why they cover the
same contracts.

## Stop Conditions

Stop before implementation if:

- WR-069 remains `blocked_deferred` or lacks accepted design, dependency,
  write-scope, validation, or contract gates;
- implementation would make renderer evidence source truth for material,
  asset, model, scene, product, shader, fallback, freshness, or rebuild policy;
- a new persisted cross-domain ABI is required without ADR review;
- visual proof, timing proof, benchmark evidence, or inspection consumption can
  only be satisfied through descriptor-only, status-panel-only, fallback-only,
  or unconsumed-contract paths;
- source files drift enough that this contract no longer describes the owning
  modules.

## Closeout Requirements

The closeout must include:

- exact changed files and owning modules;
- governance evidence and ADR decision;
- focused tests, examples, benchmark commands, and command output summaries;
- docs, roadmap, production, and planning validation results;
- artifact paths for visual evidence, raw benchmark output, and human-readable
  reports;
- evidence that material handoff and pipeline/fallback diagnostics are consumed
  through renderer inspection without moving fallback authority into renderer
  code;
- explicit known quality gaps for final perfectionist verification.

## Perfectionist Closeout Audit

Expected completion quality for WR-069: `runtime_proven`.

WR-069 should prove mesh/material runtime production evidence with visible
artifact references, timing evidence, tests, examples, benchmark/report
evidence, and docs. It must not claim `perfectionist_verified` until
`PT-RENDER-PERFECTION` audits the completed renderer stack and confirms no
known quality gaps remain.
