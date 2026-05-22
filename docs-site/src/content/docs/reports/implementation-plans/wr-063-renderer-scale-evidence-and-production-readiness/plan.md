---
title: WR-063 Renderer Scale Evidence And Production Readiness Implementation Contract
description: Design-first contract for runtime scale evidence, benchmarks, hardware profiles, documentation, and closeout.
status: active
owner: engine
layer: engine-runtime / renderer scale
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-063 Renderer Scale Evidence And Production Readiness Implementation Contract

## Goal

Prepare the production-readiness slice for `PM-RENDER-SCALE-004` and `WR-063`.
This row turns the completed renderer scale working-set, residency-budget,
visibility, LOD, compaction, and indirect-submission contracts into durable
runtime evidence, benchmarks, hardware-profile reports, public docs, and
closeout evidence.

This is a design-first implementation contract. It clears the deferred intake
questions and prepares WR-063 for roadmap application and promotion. It does
not authorize product code changes until the stack coordinator selects WR-063
for implementation after roadmap gates are satisfied.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  renderer scale claims must distinguish addressable, resident, visible,
  submitted, and measured frame cost. Runtime evidence must expose memory
  pressure, upload pressure, visible counts, culled counts, LOD bands, indirect
  command counts, GPU timings, and unsupported or degraded-mode diagnostics.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  hardware-dependent renderer claims require explicit GPU timing capability,
  unsupported diagnostics, pass-shape evidence, and runtime evidence.
- `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`:
  production readiness is based on fail-closed inspection DTOs, evidence
  budgets, replay/readiness validation, closeout records, and visible known
  gaps.
- `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`:
  WR-061 completed finite resident working-set and residency budget evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`:
  WR-062 completed renderer visibility, LOD, compaction, and indirect
  submission inspection evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-060-renderer-procedural-visuals-production-evidence/closeout.md`:
  WR-060 completed the renderer GPU/procedural production-evidence pattern for
  runtime evidence, benchmarks, and public docs.

## Readiness

`task production:plan -- --milestone PM-RENDER-SCALE-004 --roadmap WR-063`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-063-renderer-scale-evidence-and-production-readiness/plan.md`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "WR-063 Renderer Scale Evidence And Production Readiness" --scope "PM-RENDER-SCALE-004 renderer scale runtime evidence, benchmarks, hardware profiles, and production readiness"
```

No ADR is required for WR-063 if implementation only records renderer-owned
derived evidence, benchmark reports, hardware capability profiles, and docs.
Stop for ADR or accepted design update before implementation if the slice
introduces a persisted cross-domain ABI, moves product truth, semantic LOD,
streaming, fallback, freshness, rebuild policy, or visibility authority into
renderer code, or changes dependency direction between product domains and
renderer execution.

## Governance Decisions

DDD bounded context owner:

- `engine/src/plugins/render` owns renderer execution evidence, renderer
  inspection DTOs, runtime diagnostics, benchmark runners, and renderer
  production docs.
- Product, scene, gameplay, SDF, mesh/material, and editor domains remain the
  owners of authored/source truth and product policy. Renderer evidence can
  reference product lineage but must not become product truth.

Vocabulary:

- source or product truth: selected products, product lineage, residency intent,
  freshness, fallback legality, semantic LOD, streaming policy;
- renderer-derived evidence: addressable count, selected count, resident count,
  resident bytes, upload bytes, visible count, culled count, LOD execution
  band, compacted count, submitted command count, timing source, capability
  profile, hardware profile, and degraded-mode diagnostic;
- measured cost: CPU timing, GPU timing when supported, unsupported GPU timing
  diagnostics when unavailable, benchmark timing, and evidence-reporting cost.

Invariants:

- submitted work must be bounded by compacted visible work;
- visible work must be bounded by resident work;
- resident work and bytes must be bounded by selected/addressable product
  lineage and configured renderer budgets;
- unsupported timestamp, indirect, storage, readback, or runtime adapter
  capabilities must be explicit diagnostics, not silent success;
- no runtime evidence may collapse addressable, resident, visible, submitted,
  and measured cost into a single "count" or status.

Team Topologies ownership label: complicated-subsystem renderer platform team
with stream-aligned product producers consuming explicit renderer evidence.

ATAM-lite tradeoff: WR-063 favors portable, repeatable evidence and explicit
degraded-mode reporting over a single hardware-specific FPS promise. Hardware
profiles may record local adapter evidence, but completion must remain honest
when an adapter lacks timestamp queries or indirect submission support.

## Promotion Readiness

After the design-first contract and intake proposal were applied,
`task production:plan -- --milestone PM-RENDER-SCALE-004 --roadmap WR-063`
reported:

- production milestone state: `designing`;
- roadmap state: `ready_next`;
- dependency evidence: `WR-061`, `WR-062`, and `WR-060` are completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`.

The WR-063 intake proposal and active roadmap row now record:

- dependencies:
  - `WR-061` for resident working-set and budget evidence;
  - `WR-062` for visible, compacted, and submitted working-set evidence;
  - `WR-060` for the completed renderer runtime-evidence and benchmark
    closeout pattern;
- decision gates:
  - accepted renderer scale doctrine;
  - accepted renderer GPU evidence doctrine;
  - accepted render production readiness and inspection doctrine;
  - this WR-063 implementation contract;
- DDD owner: `engine/src/plugins/render`;
- ownership mode: complicated-subsystem renderer production evidence with
  stream-aligned product producers;
- ADR requirement: ADR only for persisted cross-domain ABI, dependency
  direction changes, or renderer-owned product policy;
- fitness functions: runtime evidence tests, benchmark commands, hardware
  profile validation, unsupported capability diagnostics, docs validation,
  roadmap validation, production validation, and planning validation.

Accepted promotion evidence:

- Completed WR-061 working-set registry and residency budget closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`.
- Completed WR-062 visibility, LOD, compaction, and indirect-submission
  closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`.
- Completed WR-060 renderer runtime evidence, benchmark, and docs closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-060-renderer-procedural-visuals-production-evidence/closeout.md`.
- Accepted scale, GPU evidence, and production readiness doctrines:
  `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`,
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`,
  and
  `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`.
- This WR-063 design-first runtime scale evidence implementation contract.

Promotion command:

```text
task roadmap:promote -- --id WR-063 --state current_candidate --evidence "Completed WR-061 renderer scale residency budget closeout, completed WR-062 visibility and indirect-submission closeout, completed WR-060 renderer runtime evidence closeout, accepted renderer scale/GPU/readiness doctrines, and WR-063 design-first runtime scale evidence implementation contract."
```

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Current Implementation Readiness

WR-063 has been promoted to `current_candidate`. The bounded implementation is
eligible after the stack coordinator and production planner select it, but code
changes remain limited to the implementation scope, validation, and stop
conditions in this contract.

`task production:plan -- --milestone PM-RENDER-SCALE-004 --roadmap WR-063`
reported after promotion:

- production milestone state: `designing`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-061`, `WR-062`, and `WR-060` are completed;
- next action: `write_implementation_contract`.

The implementation pass must rerun the stack and single-track coordinators
after this contract update. It may start product code only after the production
milestone is active and the coordinator selects the WR-063 implementation
slice.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/benches
engine/examples
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/benchmarks
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-evidence-and-production-r
docs-site/src/content/docs/reports/implementation-plans/wr-063-renderer-scale-evidence-and-production-readiness/plan.md
docs-site/src/content/docs/reports/closeouts/wr-063-renderer-scale-evidence-and-production-readiness/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Raw benchmark or profile artifacts must live near the owning crate under a
dedicated artifact folder, such as:

```text
engine/benchmark-artifacts/render-scale-evidence
```

Human-readable reports belong under:

```text
docs-site/src/content/docs/engine/benchmarks
docs-site/src/content/docs/reports/closeouts/wr-063-renderer-scale-evidence-and-production-readiness
```

## Exact Implementation Owners

- `engine/src/plugins/render/residency/resource.rs`:
  `RenderGpuResidencyResource::derive_from_selections`,
  `RenderGpuResidencySummary`, `RenderGpuResidencyEntry`, and
  `RenderGpuResidencyBudgetResource` are the source resident-working-set and
  residency-budget evidence for scale reports.
- `engine/src/plugins/render/inspect/gpu_residency.rs`:
  `inspect_render_gpu_residency`, `RenderGpuResidencyInspection`, and
  `RenderGpuResidencyInspectionEntry` are the public residency evidence DTO
  boundary.
- `engine/src/plugins/render/inspect/scale_visibility.rs`:
  `inspect_render_scale_visibility`, `RenderScaleVisibilityInspection`,
  `RenderScaleVisibilityRecord`, `RenderScaleVisibilityCapabilities`, and
  `RenderScaleVisibilityDiagnostic` are the visible, compacted, and submitted
  working-set evidence boundary.
- `engine/src/plugins/render/inspect/timings.rs`:
  `RenderDebugTimingsState`, `summarize_pass_timings`, and
  `summarize_gpu_pass_timing_evidence` provide timing source and GPU timing
  capability evidence.
- `engine/src/plugins/render/inspect/readiness.rs`:
  `RenderReadinessBudgetMeasurements::from_reports`,
  `evaluate_render_readiness_budgets`, and `validate_render_replay_manifest`
  are the existing fail-closed readiness and evidence-budget patterns.
- `engine/benches/render_flow_planning.rs`:
  add benchmark cases for scale residency reporting, visibility compaction,
  indirect submitted-work reporting, and evidence aggregation if no narrower
  benchmark owner exists.
- `engine/examples/render_scale_evidence.rs`:
  create the canonical finite scale-evidence example if an existing example
  cannot host it without mixing product semantics into renderer evidence.
- `engine/tests/render_scale_production_evidence.rs`:
  add focused integration tests proving runtime-evidence DTO shape, count
  invariants, unsupported capability diagnostics, and no descriptor-only
  completion claim.

## Required Contracts

WR-063 must produce explicit renderer scale evidence for:

- addressable records, selected records, accepted residency requests, resident
  records, resident bytes, and upload bytes;
- visible candidates, culled candidates by reason, LOD execution bands,
  compacted visible records, submitted draw/dispatch work, and indirect command
  counts;
- CPU timing, GPU timing when supported, GPU timing unsupported/readback
  diagnostics when not supported, and benchmark timing for the reporting path;
- hardware profile identity, adapter/capability profile, timestamp-query
  support, indirect/storage/readback support, and degraded-mode diagnostics;
- benchmark commands and outputs for registry planning, visibility compaction,
  indirect command reporting, and evidence aggregation;
- public docs that explain how to read the evidence and which claims remain
  outside renderer ownership.

The API and reports must fail closed:

- a missing hardware profile, capability profile, prepared-frame digest,
  addressable/resident/visible/submitted count, benchmark command, or closeout
  artifact is a diagnostic or validation failure;
- unsupported capabilities must produce typed degraded-mode diagnostics;
- benchmark output must not be treated as a universal FPS claim;
- local hardware evidence must record enough context to be reproducible or
  explicitly marked degraded/unsupported.

## Non-Goals

WR-063 does not implement:

- product semantic LOD, product streaming, fallback legality, freshness,
  residency intent, gameplay visibility truth, or source product authority;
- sparse SDF page tables, clipmaps, distance mips, raymarch acceleration, or
  SDF runtime proof beyond using existing renderer scale evidence vocabulary;
- mesh/material import, material specialization, lighting cache, TLAS/BLAS
  management, temporal upscaling, ray queries, product visual producers, or the
  final renderer perfectionist audit;
- a universal FPS threshold or cross-machine performance guarantee;
- public API widening solely for tests.

## Implementation Steps

1. Inspect the WR-061 residency DTOs, WR-062 visibility DTOs, WR-060 runtime
   evidence path, GPU timing DTOs, and readiness-budget helpers before adding
   new types.
2. Add a narrowly named scale production-evidence DTO or report builder under
   `engine/src/plugins/render/inspect` only if existing DTOs cannot aggregate
   the required evidence without ad hoc strings.
3. Add a canonical finite scale-evidence example under
   `engine/examples/render_scale_evidence.rs` or reuse an existing renderer
   example if it can show addressable, resident, visible, submitted, and
   measured cost without product-policy leakage.
4. Extend `engine/benches/render_flow_planning.rs` or a more specific renderer
   benchmark with scale registry, visibility compaction, indirect reporting,
   and evidence aggregation benchmarks.
5. Add focused tests in `engine/tests/render_scale_production_evidence.rs` for
   evidence shape, count invariants, missing hardware/capability profile
   failures, unsupported diagnostics, and benchmark/report command coverage.
6. Add human-readable benchmark and hardware profile documentation under
   `docs-site/src/content/docs/engine/benchmarks`, keeping raw outputs in the
   dedicated engine artifact folder.
7. Update renderer reference docs so users can discover the normal scale
   evidence workflow from `public-api-reference.md` and
   `render-flow-usage-guide.md`.
8. Close WR-063 only after focused tests, benchmark commands or documented
   unsupported runtime evidence, docs validation, roadmap validation,
   production validation, and planning validation pass.

## Required Validation

Implementation validation must include:

```text
cargo fmt
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
cargo test -p engine render_flow
cargo run -p engine --example render_scale_evidence -- --evidence
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

If the local adapter cannot produce GPU timestamp or indirect-submission
runtime evidence, the implementation must record typed unsupported diagnostics
and keep that limitation visible in the closeout and production metadata.

## Stop Conditions

Stop before implementation if:

- WR-063 has not been applied to active roadmap state and promoted through the
  required roadmap workflow;
- the implementation would move product semantic LOD, streaming, fallback,
  freshness, rebuild policy, residency intent, or visibility truth into
  renderer code;
- a runtime profile or benchmark report would silently succeed without
  capability, timing-source, or unsupported-diagnostic evidence;
- count invariants cannot prove addressable, resident, visible, submitted, and
  measured costs remain separate;
- the change requires a persisted cross-domain evidence schema or dependency
  direction change without ADR/design acceptance;
- raw benchmark artifacts would be mixed into prose docs instead of a dedicated
  artifact folder near `engine`.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-063-renderer-scale-evidence-and-production-readiness/closeout.md
```

The closeout must record:

- exact changed files, functions, modules, examples, benchmarks, and docs;
- runtime evidence command output, hardware/capability profile context, timing
  source, unsupported diagnostics if any, and benchmark command results;
- artifact paths for raw outputs and human-readable reports;
- count-invariant tests for addressable, resident, visible, submitted, and
  measured cost;
- docs, roadmap, production, and planning validation output;
- completion quality and known quality gaps.

Expected completion quality is `runtime_proven` only if runtime evidence,
hardware profile context, benchmark evidence, public docs, and closeout
validation all pass. `perfectionist_verified` remains blocked until
`PT-RENDER-PERFECTION` completes the final no-gap renderer audit.

## Perfectionist Closeout Audit

WR-063 must preserve visible gaps for the final stack audit instead of claiming
perfection:

- final cross-track renderer audit, consistency matrix, and no-gap verification
  remain `PT-RENDER-PERFECTION` scope;
- hardware evidence is local/profiled, not a universal performance guarantee;
- product semantic ownership remains outside renderer evidence.

Anti-drift guards must prevent:

- descriptor-only or docs-only completion;
- prepared-data-only evidence that is never consumed by runtime examples or
  benchmarks;
- fallback-only evidence that hides unsupported GPU capabilities;
- unbounded per-entity CPU submission in the claimed scale path;
- collapsed count reporting that hides resident, visible, submitted, or
  measured-cost differences.
