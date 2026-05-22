---
title: WR-057 Render Flow Pass Shape And Instance Contract Guards Implementation Contract
description: Design-first implementation contract for render-flow pass-shape and instance-count guard diagnostics.
status: active
owner: engine
layer: engine-runtime / render graph validation
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-057 Render Flow Pass Shape And Instance Contract Guards Implementation Contract

## Goal

Define the bounded implementation contract for `PM-RENDER-GPU-003` and
`WR-057`. The slice adds renderer-owned pass-shape diagnostics that reject
accidental fullscreen-style work multiplied by instance count unless the render
flow carries explicit author intent and bounded evidence.

This is a design-first contract. It records the accepted design evidence and
promotion readiness only. Product code changes remain unauthorized until the
stack or single-track coordinator selects the WR for implementation.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-GPU-003`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml::WR-057`.
- Accepted GPU evidence doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`.
- Accepted render execution graph maturity design:
  `docs-site/src/content/docs/design/accepted/render-execution-graph-compiler-maturity-design.md`.
- GPU timing dependency closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md`.

`task production:plan -- --milestone PM-RENDER-GPU-003 --roadmap WR-057`
reported:

- milestone state: `designing`;
- WR state: `blocked_deferred`;
- dependency: `WR-056:completed`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-057-render-flow-pass-shape-and-instance-contract-guards/plan.md`.

## Readiness

The original blockers are cleared:

- the renderer GPU evidence doctrine is accepted;
- the render execution graph compiler maturity design is accepted;
- `WR-056` is completed with runtime GPU timestamp evidence;
- architecture-governance kickoff was run for this bounded scope.

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Plan render-flow pass-shape and instance contract guards for WR-057" --scope "engine/src/plugins/render engine/tests docs-site/src/content/docs/engine/reference/plugins/render docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md docs-site/src/content/docs/design/accepted/render-execution-graph-compiler-maturity-design.md"
```

Governance conclusion for this contract:

- bounded context owner: `engine/src/plugins/render` render graph validation and
  prepared-frame preflight;
- no ADR is required while the guard remains renderer execution validation,
  extends accepted render-flow diagnostics, and does not move product truth or
  fallback policy into the renderer;
- stop for ADR/design update if the implementation rejects a previously
  documented supported public shape, changes dependency direction, persists a
  cross-domain ABI, or makes renderer diagnostics authoritative for product
  freshness or fallback legality.

## Promotion History

`task production:plan -- --milestone PM-RENDER-GPU-003 --roadmap WR-057`
reported after the row moved to `ready_next`:

- milestone state: `active`;
- WR state: `ready_next`;
- blocker: `B2`;
- dependency: `WR-056:completed`;
- promotion preflight: `promotable`.

Promotion evidence:

```text
Accepted renderer GPU evidence design, completed WR-056 GPU timing closeout, architecture-governance kickoff, and WR-057 promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-057-render-flow-pass-shape-and-instance-contract-guards/plan.md.
```

The row was promoted with that evidence:

```text
task roadmap:promote -- --id WR-057 --state current_candidate --evidence "Accepted renderer GPU evidence design, completed WR-056 GPU timing closeout, architecture-governance kickoff, and WR-057 promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-057-render-flow-pass-shape-and-instance-contract-guards/plan.md."
```

Implementation may still start only when the stack or single-track coordinator
selects `execute_next_wr_implementation_contract`.

## Ownership And Boundaries

Renderer-owned:

- render-flow pass-shape classification;
- typed diagnostics for fullscreen-plus-instance and ambiguous procedural
  shape hazards;
- explicit advanced opt-in policy on renderer pass descriptors;
- static compiler validation, prepared-frame runtime guards, and inspection
  projection of pass-shape evidence.

Not renderer-owned:

- boids-specific shortcuts;
- product truth, freshness, authority, fallback legality, rebuild policy, or
  residency policy;
- gameplay particle/VFX semantics or product selection.

## Critical Review Decisions

Source truth:

- `RenderFlow` pass descriptors and compiled render graph plans are source truth
  for pass shape.
- Prepared-frame preflight is a runtime projection that confirms compiled flow
  shapes were not bypassed by cached validation.
- Inspection docs and diagnostics are projections, not product policy.

Source-to-runtime chain:

1. `engine/src/plugins/render/api/passes.rs` records explicit pass-shape intent
   and draw instance evidence on pass descriptors.
2. `engine/src/plugins/render/graph/validation.rs` keeps existing static shape
   legality for compute, fullscreen, graphics, copy, present, and builtin UI.
3. `engine/src/plugins/render/graph/execution_plan.rs` compiles raster draw,
   buffer, and storage-binding shape into backend-neutral execution plans.
4. A dedicated graph validation owner, expected as
   `engine/src/plugins/render/graph/pass_shape.rs`, classifies risky raster
   shapes and emits typed diagnostics through
   `RenderExecutionGraphDiagnostic`.
5. `engine/src/plugins/render/graph/prepared_validation.rs` runs a runtime
   guard so cached preflight cannot hide a hazardous pass shape.
6. `engine/src/plugins/render/inspect` exposes pass-shape evidence without
   backend handles or product-policy decisions.

Typed contracts replace ad hoc strings for:

- pass-shape intent;
- fullscreen-style draw classification;
- advanced opt-in reason and bounded instance evidence;
- guard diagnostic kinds and severities.

Forbidden fallbacks:

- Do not lower instance count, skip validation, or special-case boids.
- Do not infer advanced opt-in from shader names, topology alone, product
  fallback state, or backend capability.
- Do not treat CPU/GPU timing evidence as pass-shape safety.
- Do not make product freshness, authority, or fallback legality dependent on
  renderer guard results.

## Implementation Scope

Allowed write scopes after roadmap promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-render-flow-pass-shape-and-instance-contract-guards
docs-site/src/content/docs/reports/implementation-plans/wr-057-render-flow-pass-shape-and-instance-contract-guards/plan.md
docs-site/src/content/docs/reports/closeouts/wr-057-render-flow-pass-shape-and-instance-contract-guards/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected owning modules:

- `engine/src/plugins/render/api/passes.rs`: public builder methods for typed
  advanced pass-shape intent, if implementation needs an author opt-in surface.
- `engine/src/plugins/render/graph/diagnostics.rs`: new typed diagnostic kinds
  such as fullscreen-instanced work and ambiguous procedural shape.
- `engine/src/plugins/render/graph/pass_shape.rs`: pass-shape classification
  and guard diagnostics. Add this focused module instead of growing a catch-all.
- `engine/src/plugins/render/graph/execution_plan.rs`: compiled raster shape
  inputs consumed by the guard.
- `engine/src/plugins/render/graph/prepared_validation.rs`: runtime guard
  integration for cached/strict preflight.
- `engine/src/plugins/render/inspect/plan.rs` and
  `engine/src/plugins/render/inspect/readiness.rs`: inspection/readiness
  projection if pass-shape diagnostics need public summaries.
- `engine/tests/render_flow_v2.rs`, `engine/tests/render_cutoff_guard.rs`, and
  `engine/tests/render_runtime_inspect.rs`: focused guard, preflight, and
  inspection coverage.

## Acceptance Criteria

- A graphics pass that draws fullscreen-style generated geometry with
  `instance_count > 1` and no explicit advanced intent produces a typed
  diagnostic before submit.
- A graphics pass that combines high instance count with storage-backed
  procedural data but lacks local vertex/instance geometry produces a typed
  diagnostic unless it records explicit bounded intent.
- Valid compute, fullscreen, single-instance fullscreen-style, and graphics
  passes with local vertex/instance buffers remain valid.
- Explicit advanced opt-in records author intent, pass identity, bounded
  instance evidence, and inspection output; it is never inferred.
- Prepared-frame preflight can reject or report the hazard even when full
  validation is cached.
- The public docs teach the guard as renderer execution validation, not product
  truth or boids-specific policy.

## Required Validation

Implementation validation must include:

```text
cargo test -p engine render_flow
cargo test -p engine render_cutoff_guard
cargo test -p engine render_runtime_inspect
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

Focused tests must prove:

- default rejection of fullscreen-plus-instance hazards;
- explicit advanced opt-in acceptance with bounded evidence;
- no regression for valid compute/fullscreen/graphics paths;
- runtime preflight guard coverage for cached validation;
- inspection/readiness visibility when diagnostics are public.

## Stop Conditions

Stop before implementation if:

- the guard requires inspecting shader source or shader names to infer product
  semantics;
- a public API change would reject a currently documented supported render-flow
  shape without accepted design or ADR evidence;
- product freshness, fallback legality, or residency policy must be consulted
  to decide renderer guard behavior;
- pass-shape evidence cannot be surfaced as typed diagnostics;
- validation would rely only on the boids example instead of reusable renderer
  guard tests.

## Closeout Requirements

Closeout must:

- archive `WR-057` with completed metadata only after focused tests and
  roadmap/production/docs/planning gates pass;
- update `PM-RENDER-GPU-003` evidence gates and completion metadata;
- record exact diagnostics, public API surface, and preserved valid pass shapes;
- carry any advanced-opt-in or inspection limitations as known quality gaps;
- rerun `task ai:goal -- --track PT-RENDER-PERFECTION --stack`.

Expected completion quality is `bounded_contract` unless runtime preflight and
inspection evidence prove the guard path end to end. `perfectionist_verified`
is not available for this row.

## Perfectionist Closeout Audit

The closeout audit must reject descriptor-only or status-panel-only completion.
It needs evidence that the guard is consumed by the compile/preflight path used
before submit, not only documented on a builder or shown in a report. Any
remaining accepted advanced opt-in ambiguity must stay visible in
`known_quality_gaps` until later procedural API and boids proof rows consume the
contract.

## Implementation Closeout Log

2026-05-22 bounded slice:

- Added `RenderPassShapeIntent` to `RenderPassNode` and
  `GraphicsPassBuilder::allow_instanced_fullscreen(...)` for explicit bounded
  advanced intent.
- Added `engine/src/plugins/render/graph/pass_shape.rs` as the renderer-owned
  pass-shape classifier and wired it into `compile_flow_plan_checked(...)`,
  full prepared-frame preflight, and cached runtime guards.
- Added typed diagnostic kinds for fullscreen-instanced work, ambiguous
  procedural shape, and invalid pass-shape intent.
- Added focused render-flow tests for default rejection, explicit opt-in,
  opt-in limit enforcement, local geometry preservation, and runtime guard
  rejection before a cache hit.
- Updated render reference docs for the new public contract.

Validation:

```text
cargo test -p engine render_flow
cargo test -p engine render_cutoff_guard
cargo test -p engine render_runtime_inspect
task docs:validate
```
