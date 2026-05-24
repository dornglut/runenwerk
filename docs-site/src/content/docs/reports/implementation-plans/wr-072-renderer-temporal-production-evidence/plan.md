---
title: WR-072 Renderer Temporal Production Evidence Implementation Contract
description: Design-first contract for temporal runtime production evidence, examples, benchmarks, artifacts, docs, and closeout.
status: active
owner: engine
layer: engine-runtime / renderer temporal production evidence
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-072 Renderer Temporal Production Evidence Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-TEMPORAL-004` and `WR-072`.
This row must prove the renderer temporal reconstruction platform through
runtime-facing evidence: examples, benchmark/report artifacts, public docs,
timing, quality, fallback, history, temporal-input, and adapter/ray-input
inspection evidence.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-072` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md`:
  accepted doctrine for portable temporal reconstruction, dynamic resolution,
  optional adapters, ray reconstruction inputs, diagnostics, runtime evidence,
  and fallback.
- `docs-site/src/content/docs/reports/closeouts/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/closeout.md`:
  completed temporal input, history, jitter, and dynamic-resolution evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-071-renderer-upscaling-adapters-and-ray-reconstruction-inputs/closeout.md`:
  completed optional adapter and ray reconstruction input inspection evidence.
- `engine/src/plugins/render`: owning renderer inspection and production
  evidence boundary.
- `engine/examples`, `engine/benches`, `engine/benchmark-artifacts`, and
  `docs-site/src/content/docs/reports/benchmarks/render`: owning example,
  benchmark, raw artifact, and human report locations for runtime evidence.

## Readiness

`task production:plan -- --milestone PM-RENDER-TEMPORAL-004 --roadmap WR-072`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-072-renderer-temporal-production-evidence/plan.md`.

This contract clears the design-first gap by making completed WR-070 and WR-071
dependencies, runtime evidence scope, write scopes, validation, stop
conditions, and closeout quality explicit before implementation.

After applying the intake proposal, WR-072 may be promoted only when:

- `WR-070` and `WR-071` remain completed with valid closeout evidence;
- `PM-RENDER-TEMPORAL-004` is active and still selected by the stack
  coordinator;
- `task production:plan -- --milestone PM-RENDER-TEMPORAL-004 --roadmap WR-072`
  reports a promotable or promotion-contract action rather than `design_first`;
- roadmap, production, docs, and planning validators pass.

The promotion preflight now reports:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependency: `WR-071:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-072 --state current_candidate --evidence "Accepted renderer temporal reconstruction doctrine, completed WR-070 temporal inputs/history/dynamic-resolution closeout, completed WR-071 upscaling adapter/ray reconstruction input closeout, and active WR-072 temporal production evidence contract."
```

WR-072 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-TEMPORAL-004`.

## Governance Decisions

- DDD bounded context owner: `engine/src/plugins/render`.
- Renderer-owned vocabulary: temporal production evidence report, temporal
  hardware profile, runtime visual evidence reference, timing evidence,
  benchmark command, raw artifact path, human report path, temporal readiness
  summary, adapter fallback summary, and production diagnostics.
- Source-owner vocabulary: camera truth, scene/product generation, SDF and
  ray-query authority, material reactivity semantics, exposure meaning, product
  freshness, and fallback legality.
- Translation boundary: WR-072 may aggregate renderer-owned WR-070 and WR-071
  inspection reports plus runtime evidence references. It must not make docs,
  artifacts, examples, or benchmark summaries authoritative product state.
- Clean Architecture direction: examples, benchmarks, and docs may consume
  renderer public APIs. The renderer must not depend on docs, app/editor state,
  benchmark harness state, or artifact files for runtime correctness.
- ADR requirement: no ADR is required for production evidence DTOs, examples,
  benchmark wiring, or reports. Stop for ADR if implementation introduces a
  durable external evidence ABI, changes runtime ownership, or requires a
  vendor SDK or hardware feature for the baseline temporal claim.
- Team Topologies ownership: complicated-subsystem renderer platform work with
  stream-aligned producer evidence and enabling workspace documentation.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
engine/examples
engine/benches
engine/benchmark-artifacts
engine/Cargo.toml
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/benchmarks/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-temporal-production-evidence
docs-site/src/content/docs/reports/implementation-plans/wr-072-renderer-temporal-production-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-072-renderer-temporal-production-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/inspect/temporal_production.rs`: aggregate
  temporal input inspection, temporal upscaling inspection, runtime visual
  evidence, timing, hardware profile, benchmark command, artifact path, and
  human report evidence into a fail-closed temporal production report.
- `engine/src/plugins/render/inspect/mod.rs`: export the temporal production
  evidence API.
- `engine/tests/render_temporal_production_evidence.rs`: guard runtime
  evidence readiness, missing visual evidence, missing benchmark/artifact
  paths, unconsumed WR-070/WR-071 inspections, unsupported timing diagnostics,
  and fallback-only claims.
- `engine/examples/render_temporal_production_evidence.rs`: print the canonical
  temporal production evidence summary.
- `engine/benches/render_flow_planning.rs`: add a direct temporal production
  evidence benchmark case.
- `engine/benchmark-artifacts/render-temporal-production-evidence/`: store raw
  benchmark artifact placeholders and summaries.
- `docs-site/src/content/docs/reports/benchmarks/render/temporal-production-evidence.md`:
  store the human-readable benchmark/evidence report.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the public API and evidence contract.

## Non-Goals

- Do not implement FSR, DLSS, XeSS, frame generation, vendor SDK calls, or
  hardware ray-query execution.
- Do not claim `perfectionist_verified`; final renderer audit remains
  `PT-RENDER-PERFECTION` scope.
- Do not make artifact files, docs, or benchmark summaries runtime
  dependencies.
- Do not move camera, scene, product, material, exposure, SDF, ray-query, or
  fallback authority into renderer production evidence code.
- Do not broaden app/editor UI.

## Required Implementation Shape

WR-072 must provide typed renderer evidence for:

1. Consumption of WR-070 temporal input/history/dynamic-resolution inspection.
2. Consumption of WR-071 adapter/ray reconstruction input inspection.
3. Runtime visual evidence references with nonzero rendered-pixel or coverage
   evidence.
4. CPU timing evidence and explicit GPU timing support/unsupported diagnostics.
5. Hardware or backend capability profile identity.
6. Benchmark commands and raw/human artifact paths.
7. Fail-closed diagnostics for missing evidence, unconsumed inspections,
   fallback-only claims, missing timing diagnostics, and broken count
   invariants.

The closeout target is `runtime_proven` for the temporal track. Any remaining
gap must stay visible and prevent the track from claiming
`perfectionist_verified`.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_temporal
cargo test -p engine render_temporal_upscaling
cargo test -p engine render_temporal_production
cargo test -p engine render_runtime_inspect
cargo test -p engine --example render_temporal_production_evidence
cargo run -p engine --example render_temporal_production_evidence
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

If implementation touches shared render-flow benchmark setup, add the smallest
focused existing render-flow benchmark or test filter before closeout.

## Stop Conditions

Stop before product code if:

- `WR-072` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-TEMPORAL-004 --roadmap WR-072`
  still reports `design_first`, a promotion blocker, or a metadata blocker;
- WR-070 or WR-071 closeout evidence is missing or invalid;
- runtime evidence cannot consume both WR-070 and WR-071 inspection reports;
- a vendor adapter, hardware ray query, or GPU timing support is required to
  make the portable path pass;
- required validation cannot run or fails;
- closeout evidence cannot honestly claim `runtime_proven`.

## Closeout Requirements

The closeout must include:

- exact changed modules, functions, examples, benchmarks, and artifact paths;
- architecture evidence showing reports are renderer evidence, not product
  truth;
- validation commands and results;
- public API/doc updates;
- roadmap and production metadata updates;
- completion quality `runtime_proven`;
- remaining gaps, if any, that prevent `perfectionist_verified`.
