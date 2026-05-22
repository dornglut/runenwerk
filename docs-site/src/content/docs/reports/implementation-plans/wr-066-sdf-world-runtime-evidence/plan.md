---
title: WR-066 SDF World Runtime Evidence Implementation Contract
description: Design-first contract for sparse SDF runtime examples, benchmark evidence, visual proof, and production readiness.
status: active
owner: engine
layer: engine-runtime / renderer sdf runtime evidence
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-066 SDF World Runtime Evidence Implementation Contract

## Goal

Prepare the runtime-proven implementation slice for `PM-RENDER-SDF-004` and
`WR-066`. This row must prove the sparse SDF renderer path through executable
examples, benchmark commands, visual/runtime evidence, SDF residency
diagnostics, raymarch acceleration diagnostics, GPU/CPU timing evidence, and a
closeout-ready production report.

This is a design-first contract. It clears the deferred intake questions and
prepares WR-066 for roadmap application and promotion. It does not authorize
product code changes until the stack coordinator selects WR-066 for
implementation after roadmap gates are satisfied.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md`:
  production SDF evidence must include GPU timing, ray step statistics, page
  and brick residency state, clipmap coverage, candidate-list sizes, cache
  rebuild pressure, memory pressure, and visual runtime proof for near, mid,
  far, and summary behavior.
- `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`:
  runtime evidence must be closeout-ready, budgeted, inspectable, and honest
  about host-backed GPU/capture limits.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  GPU timing evidence must distinguish supported timings from unsupported
  diagnostics, and examples must prove bounded work instead of multiplied
  fullscreen work.
- `docs-site/src/content/docs/reports/closeouts/wr-064-sparse-sdf-brick-page-and-clipmap-residency/closeout.md`:
  WR-064 completed SDF page, brick, clipmap, invalidation, byte, upload, and
  budget-pressure residency evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-065-sdf-raymarch-acceleration-and-candidate-lists/closeout.md`:
  WR-065 completed conservative SDF distance mip, safe-step, tile/depth
  candidate-list, rejected-candidate, and diagnostic evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-063-renderer-scale-evidence-and-production-readiness/closeout.md`:
  WR-063 completed the renderer production-evidence pattern for hardware
  profiles, benchmark commands, timing evidence, artifact paths, and
  fail-closed count invariants.
- `engine/examples/sdf_render_flow`:
  current 3D SDF example with compute prepare, fullscreen compose, history
  copy, present pass, view modes, and state-projected uniforms.
- `engine/examples/procedural_sky_sdf_terrain`:
  current navigable SDF terrain example with free-flight camera and lit,
  height, normal, and step views.
- `engine/benches/render_flow_planning.rs`:
  current benchmark owner for render-flow planning and SDF-like compute/compose
  cases.
- `engine/src/plugins/render/features/world/sdf_residency.rs` and
  `engine/src/plugins/render/features/world/sdf_raymarch.rs`:
  completed renderer-owned SDF residency and raymarch acceleration evidence
  sources that WR-066 must consume, not duplicate.

## Readiness

`task production:plan -- --milestone PM-RENDER-SDF-004 --roadmap WR-066`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-066-sdf-world-runtime-evidence/plan.md`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "WR-066 SDF World Runtime Evidence" --scope "PM-RENDER-SDF-004 engine/examples engine/benches engine/src/plugins/render SDF runtime examples visual proof benchmarks GPU timing residency raymarch diagnostics production evidence"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer
  execution evidence, SDF residency/raymarch inspection DTOs, timing evidence,
  production reports, benchmark harnesses, and example rendering flows.
- Runtime example owner: `engine/examples/sdf_render_flow` and
  `engine/examples/procedural_sky_sdf_terrain` own executable example wiring,
  CLI evidence output, view modes, camera controls, screenshots or artifact
  references, and user-visible proof surfaces.
- Source truth owner: `domain/world_sdf`, SDF products, and upstream product
  publication own authored SDF payload truth, query policy, fallback legality,
  collision semantics, generation authority, and source freshness.
- Translation boundary: WR-066 may synthesize or load example SDF payloads to
  feed renderer evidence, but the closeout must identify them as example
  evidence inputs. Renderer evidence cannot become canonical world SDF product
  truth.
- ADR requirement: no ADR is required if WR-066 only adds runtime examples,
  renderer-owned evidence aggregation, benchmark/artifact docs, and tests over
  accepted renderer SDF contracts. Stop for ADR or accepted design update if
  implementation introduces persisted cross-domain SDF runtime ABI, changes
  dependency direction, or makes renderer code authoritative for product query,
  collision, fallback, or rebuild policy.
- Team Topologies ownership: complicated-subsystem renderer platform work with
  stream-aligned SDF/domain product producers.

## Promotion Readiness

WR-066 can become promotable only after the intake proposal records:

- dependencies on completed `WR-064`, `WR-065`, and `WR-063`;
- accepted SDF, renderer production readiness, GPU evidence, and scale
  residency design gates;
- completed closeout gates for `WR-064`, `WR-065`, and `WR-063`;
- this active implementation contract as a design gate;
- implementation write scopes for examples, benchmarks, renderer evidence,
  tests, benchmark artifacts, renderer docs, intake, roadmap metadata, and
  production metadata.

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/examples/sdf_render_flow
engine/examples/procedural_sky_sdf_terrain
engine/examples/render_sdf_runtime_evidence.rs
engine/benches/render_flow_planning.rs
engine/benchmark-artifacts/render-sdf-runtime-evidence
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/benchmarks
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-sdf-world-runtime-evidence
docs-site/src/content/docs/reports/implementation-plans/wr-066-sdf-world-runtime-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-066-sdf-world-runtime-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/inspect/sdf_production.rs`:
  preferred new module for `RenderSdfProductionEvidenceRequest`,
  `RenderSdfProductionEvidenceReport`, hardware/profile evidence, count
  invariants, timing summaries, visual artifact references, benchmark commands,
  and fail-closed diagnostics.
- `engine/src/plugins/render/features/world/sdf_residency.rs`:
  input owner for SDF page, brick, clipmap, byte, upload, invalidation, and
  budget-pressure evidence. Extend only for narrow runtime evidence fields that
  cannot be derived externally.
- `engine/src/plugins/render/features/world/sdf_raymarch.rs`:
  input owner for distance mips, safe steps, candidate lists, rejected
  candidates, and acceleration diagnostics. Extend only for runtime step
  statistics that belong to renderer acceleration evidence.
- `engine/examples/sdf_render_flow/rendering/evidence.rs`:
  preferred evidence module for the canonical 3D SDF example. It should build
  the render flow, compile/preflight it, derive SDF residency and raymarch
  evidence over near/mid/far example payloads, and format `--evidence` output.
- `engine/examples/sdf_render_flow/main.rs`:
  add the `--evidence` CLI path, mirroring the boids evidence pattern.
- `engine/examples/procedural_sky_sdf_terrain/rendering/evidence.rs`:
  optional supporting evidence if the terrain example is used for free-flight
  visual proof. Keep the canonical runtime-proven gate on
  `sdf_render_flow` unless implementation explicitly chooses terrain as the
  canonical example.
- `engine/examples/render_sdf_runtime_evidence.rs`:
  optional non-windowed evidence command if a standalone report builder is
  cleaner than extending the runtime examples.
- `engine/benches/render_flow_planning.rs`:
  add or harden benchmark cases for cold/warm SDF residency, ray step budgets,
  and candidate-list behavior.
- `engine/tests/render_sdf_runtime_evidence.rs`:
  focused tests for ready reports, missing visual/timing/artifact evidence,
  count-invariant drift, missing residency/raymarch inputs, and unsupported GPU
  timing diagnostics.
- `docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md`:
  human-readable benchmark report. Raw artifacts belong under
  `engine/benchmark-artifacts/render-sdf-runtime-evidence`.

## Required Contracts

WR-066 must introduce explicit runtime evidence for:

- visible near, mid, far, and summary SDF behavior through example view modes,
  screenshot/artifact references, or deterministic evidence output;
- SDF residency evidence: selected products, resident products, page counts,
  brick counts, clipmap windows, resident bytes, upload bytes, invalidation,
  and budget pressure;
- SDF raymarch acceleration evidence: distance mip levels, safe-step bounds,
  candidate-list counts, rejected candidate counts, max steps per ray, and
  overstep/candidate diagnostics;
- timing evidence: CPU preflight/flow/submit fields where available, GPU pass
  timings when supported, and explicit unsupported/readback-pending diagnostics
  when GPU timing is unavailable;
- benchmark evidence for cold and warm residency, ray step budgets, and
  candidate-list behavior;
- visual proof evidence that points to generated artifacts or explains why a
  host-backed capture could not run while still preserving diagnostics.

The report must fail closed:

- missing residency evidence, missing raymarch evidence, missing benchmark
  command, missing visual artifact reference, missing hardware/capability
  profile, broken count invariants, unsupported timing without diagnostics, and
  fullscreen-per-entity multiplication block runtime-ready claims;
- unsupported host GPU capture can be a diagnostic only when the report still
  contains deterministic example, timing, residency, raymarch, and artifact
  evidence;
- runtime reports must not synthesize product success from fallback paths.

## Non-Goals

WR-066 does not implement:

- new SDF authoring, product formation, physics/collision truth, gameplay query
  authority, fallback legality, or product rebuild policy;
- SDF brick/page/clipmap residency, which is completed WR-064 scope;
- SDF raymarch acceleration and candidate lists, which are completed WR-065
  scope;
- mesh/material lighting, temporal reconstruction, hardware ray queries, product
  visual producers, or final perfectionist verification.

## Implementation Steps

1. Re-read WR-064, WR-065, WR-063 closeouts and the accepted SDF/production/GPU
   evidence designs before code changes.
2. Add the SDF production evidence report in the renderer inspection domain,
   consuming existing SDF residency and raymarch evidence rather than
   duplicating their logic.
3. Extend the canonical SDF example with `--evidence` output and deterministic
   near/mid/far example payloads or artifact references.
4. Add focused tests that fail closed when evidence is descriptor-only,
   unconsumed, missing visual proof, missing benchmark evidence, or missing
   timing diagnostics.
5. Add or harden benchmark cases for cold/warm residency and candidate-list
   behavior, then record raw artifacts and a benchmark report in the approved
   locations.
6. Update renderer public API docs, usage guide, and benchmark docs so product
   teams can run the SDF runtime evidence command and understand remaining
   host-dependent diagnostics.
7. Close WR-066 only after focused validation, example evidence command,
   benchmark command, docs validation, roadmap validation, production
   validation, and planning validation pass.

## Required Validation

Implementation validation must include:

```text
cargo fmt
cargo test -p engine render_sdf
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
cargo test -p engine --example sdf_render_flow
cargo test -p engine --example render_sdf_runtime_evidence
cargo run -p engine --example sdf_render_flow -- --evidence
cargo run -p engine --example render_sdf_runtime_evidence -- --evidence
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

If implementation chooses only one evidence command, remove the unused example
command from the active WR row before promotion instead of leaving a dead
validation requirement.

## Acceptance Criteria

- The canonical SDF example exposes a deterministic evidence command with
  renderer flow/preflight, SDF residency, SDF raymarch, timing, benchmark, and
  visual artifact evidence.
- Runtime evidence proves near, mid, far, and summary SDF behavior without
  multiplying fullscreen work by entity count.
- Benchmarks cover cold and warm residency, ray step budgets, and
  candidate-list behavior.
- Missing evidence fails closed and prevents `runtime_proven` claims.
- Renderer evidence remains derived execution evidence and does not become SDF
  product truth.

## Stop Conditions

Stop before implementation if:

- WR-066 has not been applied to active roadmap state and promoted through the
  required roadmap workflow;
- implementation would move SDF product truth, collision semantics, fallback
  legality, query authority, or rebuild policy into renderer code;
- runtime evidence cannot consume the completed WR-064 and WR-065 contracts;
- the example can only provide descriptor/status evidence without executable
  flow/preflight/timing/residency/raymarch consumption;
- benchmark or visual artifact evidence cannot be recorded honestly.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-066-sdf-world-runtime-evidence/closeout.md
```

The closeout must record exact changed modules/functions, example commands,
benchmark commands, artifact paths, hardware/capability profile, residency
evidence, raymarch evidence, timing evidence, visual proof evidence, validation
output, docs updates, roadmap/production metadata updates, completion quality,
and remaining quality gaps.

Expected completion quality is `runtime_proven`. `perfectionist_verified`
remains blocked until `PT-RENDER-PERFECTION` completes the final no-gap renderer
audit.

## Perfectionist Closeout Audit

WR-066 must preserve visible gaps for later tracks instead of claiming final
perfection:

- host-dependent GPU capture or timestamp support must be classified as
  supported, unsupported, readback-pending, or unavailable with diagnostics;
- any benchmark portability limits must be recorded as known quality gaps;
- `PT-RENDER-PERFECTION` owns the final no-gap audit after all renderer
  dependency tracks close.

Anti-drift guards must prevent descriptor-only evidence, prepared-data-only
evidence, status-panel-only evidence, fallback-only success, unconsumed
residency/raymarch DTOs, missing benchmark artifacts, missing visual proof, and
fullscreen-per-entity multiplication.
