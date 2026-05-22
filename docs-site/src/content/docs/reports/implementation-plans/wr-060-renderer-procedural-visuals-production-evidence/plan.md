---
title: WR-060 Renderer Procedural Visuals Production Evidence Implementation Contract
description: Design-first contract for closing the renderer GPU/procedural visuals track with runtime evidence, benchmarks, docs, and closeout.
status: active
owner: engine
layer: engine-runtime / renderer evidence
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-060 Renderer Procedural Visuals Production Evidence Implementation Contract

## Goal

Define the bounded production-evidence slice for `PM-RENDER-GPU-006` and
`WR-060`. This row closes the renderer GPU/procedural visuals platform by
turning the completed timing, pass-shape, procedural API, and boids proof rows
into durable runtime evidence, benchmark evidence, public docs, and a closeout.

This contract is design-first. It clears the blocked-deferred planning state and
prepares the row for roadmap promotion. It does not authorize product code
changes until the stack coordinator selects the row for promotion and then
implementation.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  renderer owns GPU timing evidence, pass-shape guards, procedural visual APIs,
  examples, benchmarks, runtime evidence, and inspection DTOs.
- `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`:
  completion quality starts at `bounded_contract`; `runtime_proven` requires
  runtime evidence and passing production checks; `perfectionist_verified`
  requires a completed audit and no known quality gaps.
- `docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md`:
  GPU timing source DTOs, capability diagnostics, and readiness-budget
  integration are complete.
- `docs-site/src/content/docs/reports/closeouts/wr-057-render-flow-pass-shape-and-instance-contract-guards/closeout.md`:
  renderer validation/preflight guards diagnose hazardous pass shapes before
  submit.
- `docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md`:
  public procedural descriptors can build mesh sprite, generated quad sprite,
  and local 2D SDF impostor passes.
- `docs-site/src/content/docs/reports/closeouts/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/closeout.md`:
  boids consumes the procedural API and no longer uses a fullscreen fragment
  loop over all boids, but finite runtime timing artifacts remain for `WR-060`.

## Readiness

`task production:plan -- --milestone PM-RENDER-GPU-006 --roadmap WR-060`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- dependencies: `WR-056`, `WR-057`, `WR-058`, and `WR-059` are completed;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-060-renderer-procedural-visuals-production-evidence/plan.md`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "Plan renderer procedural visuals production evidence for WR-060" --scope "engine/benches engine/examples engine/tests docs-site/src/content/docs/engine/reference/plugins/render docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-procedural-visuals-production-evidence"
```

No ADR is required for this slice if implementation only records and verifies
renderer-owned evidence. Stop for ADR or design update before implementation if
the evidence path changes renderer/product dependency direction, makes renderer
state authoritative for product policy, introduces persisted cross-domain ABI,
or makes GPU timing/procedural visual APIs mandatory outside renderer-owned
execution.

## Promotion Readiness

After the design-first contract was applied, `task production:plan -- --milestone
PM-RENDER-GPU-006 --roadmap WR-060` reported:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- dependencies: `WR-056`, `WR-057`, `WR-058`, and `WR-059` are completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`.

Accepted promotion evidence:

- `WR-056` completed GPU pass timing and workload evidence closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md`.
- `WR-057` completed render-flow pass-shape and instance guard closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-057-render-flow-pass-shape-and-instance-contract-guards/closeout.md`.
- `WR-058` completed hybrid procedural instance API closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md`.
- `WR-059` completed canonical boids procedural rewrite closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/closeout.md`.
- This `WR-060` design-first production evidence contract.

Promotion command:

```text
task roadmap:promote -- --id WR-060 --state current_candidate --evidence "Completed WR-056 GPU timing closeout, WR-057 pass-shape guard closeout, WR-058 procedural API closeout, WR-059 boids procedural rewrite closeout, and WR-060 design-first production evidence contract at docs-site/src/content/docs/reports/implementation-plans/wr-060-renderer-procedural-visuals-production-evidence/plan.md."
```

Promotion does not authorize product code by itself. After promotion, rerun the
stack and single-track coordinators and write the implementation-readiness
contract before code changes.

## Resolved Design Decisions

Runtime machines and GPU capability profiles:

- Evidence must run on the local available adapter and record backend,
  adapter/capability profile where available, timestamp-query support, timing
  source, scene size, pass shape, and instance count.
- Timestamp-supported adapters must produce GPU pass timing evidence for the
  relevant compute and graphics passes.
- Backends without timestamp support can still support `runtime_proven` only
  when they emit typed unsupported GPU timing diagnostics plus CPU
  encode/submit timing and pass-shape evidence.
- Universal frame-rate thresholds are out of scope. The evidence must prove
  bounded work and timing visibility, not a portable FPS promise.

Mandatory examples and benchmarks:

- `engine/examples/boids_render_flow` is the mandatory canonical runtime proof.
- `engine/benches/render_flow_planning.rs` is the mandatory planning benchmark
  command for this row unless the implementation proves a more specific
  existing benchmark is already the canonical renderer planning benchmark.
- `engine/tests/render_runtime_inspect.rs`, `engine/tests/render_flow_v2.rs`,
  `engine/tests/render_gpu_timing.rs`, and
  `engine/tests/procedural_instance.rs` are the expected focused test families.
- Public renderer docs under
  `docs-site/src/content/docs/engine/reference/plugins/render` must describe
  how to read GPU timing, unsupported diagnostics, pass-shape evidence,
  procedural visual evidence, and the boids proof.

Quality-gate policy:

- `PM-RENDER-GPU-006` targets `runtime_proven`.
- `perfectionist_verified` remains unavailable for this track because the final
  cross-track perfection audit belongs to `PT-RENDER-PERFECTION`.
- Any remaining gaps must be listed in the WR closeout and production track
  metadata instead of being hidden behind a successful benchmark.

## Ownership And Boundaries

Renderer owns:

- benchmark runners and benchmark reports for renderer execution contracts;
- example runtime evidence for boids procedural visuals;
- renderer inspection DTO tests and docs;
- closeout evidence for GPU timing, unsupported diagnostics, pass shape,
  procedural API consumption, and bounded work.

Renderer does not own:

- product truth, product selection, freshness, authority, fallback legality,
  rebuild policy, or residency policy;
- gameplay particle, field, VFX, material, model, editor, or authored-content
  semantics;
- final cross-track perfectionist audit acceptance.

Team Topologies ownership label: complicated-subsystem renderer platform team,
with stream-aligned product/editor producers consuming explicit renderer
execution evidence.

## Implementation Scope

Allowed write scopes after roadmap promotion:

```text
engine/benches
engine/examples
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-procedural-visuals-production-evidence
docs-site/src/content/docs/reports/implementation-plans/wr-060-renderer-procedural-visuals-production-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-060-renderer-procedural-visuals-production-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected owning files or modules:

- `engine/examples/boids_render_flow`: finite runtime evidence hook or bounded
  smoke path for the canonical procedural visual example.
- `engine/benches/render_flow_planning.rs`: benchmark command and raw output
  source for render-flow planning evidence.
- `engine/tests/render_runtime_inspect.rs`: inspection DTO and timing evidence
  regression coverage.
- `engine/tests/render_flow_v2.rs` and `engine/tests/procedural_instance.rs`:
  pass-shape/procedural API guard coverage if implementation exposes new
  evidence helpers.
- `docs-site/src/content/docs/engine/reference/plugins/render`: public evidence
  usage documentation.

## Implementation Steps

1. Add a finite boids production-evidence path under
   `engine/examples/boids_render_flow`. Prefer a small example-local evidence
   module, such as `engine/examples/boids_render_flow/runtime/evidence.rs` or
   `engine/examples/boids_render_flow/rendering/evidence.rs`, that can be
   called from tests and, if practical, from a command-line evidence mode. The
   report must include flow id, pass labels/kinds, pass order, instance count,
   local geometry/pass-shape proof, scene size, timing source, GPU capability,
   and any unsupported diagnostics.
2. Use existing renderer inspection DTOs instead of ad hoc strings. The
   implementation should consume `RenderDebugTimingsState`,
   `RenderPassTimingEvidence`, compiled flow inspection, preflight inspection,
   and readiness-budget DTOs where they already represent the needed evidence.
3. Add or extend tests in `engine/tests/render_runtime_inspect.rs` and the
   boids example tests so the report proves compute, publish, draw, and present
   pass evidence without backend handles. Tests must fail if the boids draw path
   returns to fullscreen-per-boid rendering or a fragment loop over all boids.
4. Run the existing runtime GPU timing ignored test as evidence when a local
   adapter is available:

   ```text
   cargo test -p engine render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported -- --ignored --nocapture
   ```

   The closeout must capture whether this produced measured GPU timing, typed
   unsupported diagnostics, or no-adapter diagnostics.
5. Run `cargo bench -p engine --bench render_flow_planning` and record the
   benchmark command plus Criterion output location. If the existing benchmark
   lacks a procedural-boids case after `WR-059`, add one in
   `engine/benches/render_flow_planning.rs` rather than creating a parallel
   benchmark suite.
6. Update renderer public docs under
   `docs-site/src/content/docs/engine/reference/plugins/render` so normal users
   can find the evidence path from the render-flow guide and public API
   reference.
7. Create the WR-060 closeout only after the focused tests, benchmark command,
   docs validation, roadmap validation, production validation, and planning
   validation pass.

## Evidence Artifacts

Required closeout evidence:

- Runtime evidence: command output from the finite boids evidence path or
  bounded smoke, including capability/timing status and pass evidence.
- GPU timing evidence: output from the ignored runtime timestamp-query test,
  including measured, unsupported, or no-adapter diagnostics.
- Benchmark evidence: `cargo bench -p engine --bench render_flow_planning`
  command and Criterion output location under `target/criterion`.
- Documentation evidence: changed renderer public docs and `task docs:validate`.
- Governance evidence: roadmap, production, and planning validations after
  closeout metadata updates.

## Acceptance Criteria

- Runtime evidence records boids scene size, pass order, pass kind, instance
  count, pass shape, CPU timing, GPU timing when supported, or typed unsupported
  GPU timing diagnostics when unsupported.
- Benchmark evidence records the exact command, benchmark identity, raw artifact
  location, and a human-readable summary in renderer-owned docs or closeout.
- Public docs explain how to interpret GPU timing evidence, unsupported
  diagnostics, pass-shape guards, procedural local SDF impostor evidence, and
  the canonical boids proof.
- The closeout proves that `WR-056` through `WR-059` evidence is consumed
  together instead of remaining isolated implementation rows.
- No product truth, product fallback, product freshness, product residency, or
  gameplay/VFX semantics move into renderer-owned evidence helpers.

## Required Validation

Implementation validation must include:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
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

If the implementation adds a finite boids runtime smoke command, the closeout
must record the exact command and whether GPU timing was measured, unsupported,
or unavailable this frame.

## Stop Conditions

Stop before implementation if:

- roadmap promotion preflight reports a hard blocker or unresolved current
  candidate conflict;
- the row cannot produce runtime timing or explicit unsupported timing
  diagnostics;
- the benchmark path would require product-domain policy or renderer-owned
  product truth;
- the boids evidence can only be produced by lowering count, disabling
  validation, disabling diagnostics, or bypassing the public procedural API;
- a closeout would need to claim `perfectionist_verified` before the final
  renderer perfection track runs.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-060-renderer-procedural-visuals-production-evidence/closeout.md
```

The closeout must record:

- changed files with exact modules/functions;
- runtime evidence command, environment/capability profile, timing source, and
  pass evidence;
- benchmark command, artifact location, and human-readable report;
- docs updates;
- validation output;
- completion quality and any remaining quality gaps.

Roadmap closeout must archive `WR-060` with the closeout path in
`write_scopes`, `completion_quality`, `known_quality_gaps`, and
`completion_audit`. Production closeout must update `PM-RENDER-GPU-006` with
evidence gates and completion quality only after the closeout exists.

## Perfectionist Closeout Audit

Expected completion quality for this row is `runtime_proven` if runtime
evidence and benchmark evidence pass and the closeout lists any remaining gaps.

`perfectionist_verified` is explicitly unavailable for `WR-060` and
`PM-RENDER-GPU-006`; the final no-gap audit belongs to
`PT-RENDER-PERFECTION` after the full renderer dependency stack is complete.
