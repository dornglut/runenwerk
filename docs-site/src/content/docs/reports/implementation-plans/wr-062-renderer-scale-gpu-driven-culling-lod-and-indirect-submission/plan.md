---
title: WR-062 Renderer Scale GPU Driven Culling LOD And Indirect Submission Implementation Contract
description: Design-first contract for capability-gated visible working sets, LOD selection, compaction, and indirect submission evidence.
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
  - ../../../workspace/production-tracks.yaml
---

# WR-062 Renderer Scale GPU Driven Culling LOD And Indirect Submission Implementation Contract

## Goal

Prepare the bounded implementation slice for `PM-RENDER-SCALE-003` and
`WR-062`. This row adds renderer-owned visible working-set evidence and
capability-gated submission reduction: frustum/screen-size culling, renderer LOD
band selection, visible-list compaction, and indirect draw or dispatch command
generation diagnostics.

This is a design-first implementation contract. It clears the blocked-deferred
intake questions and prepares WR-062 for roadmap application and promotion. It
does not authorize product code changes until the stack coordinator selects
WR-062 for implementation.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  visibility and LOD reduce submitted renderer work before command encoding,
  and GPU-driven paths must be capability-gated with explicit unsupported
  diagnostics.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  GPU timing and pass-shape evidence must remain explicit when runtime claims
  depend on GPU execution.
- `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`:
  WR-061 completed finite resident working-set and budget-pressure evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md`:
  WR-056 completed GPU timing DTO and capability evidence.

## Readiness

`task production:plan -- --milestone PM-RENDER-SCALE-003 --roadmap WR-062`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/plan.md`.

The accepted scale doctrine already records the ownership and dependency
direction. No ADR is required if implementation only adds renderer-owned derived
visibility buffers, LOD bands, compaction/indirect DTOs, and diagnostics. Stop
for ADR or design update before implementation if the slice moves semantic LOD,
streaming, product fallback, visibility truth, or source product authority into
renderer code.

## Promotion Readiness

After the design-first contract and intake proposal were applied,
`task production:plan -- --milestone PM-RENDER-SCALE-003 --roadmap WR-062`
reported:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- dependency evidence: `WR-061` and `WR-056` are completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`.

Accepted promotion evidence:

- Completed WR-061 working-set registry and residency budget closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`.
- Completed WR-056 GPU timing and capability evidence closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md`.
- Accepted scale doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`.
- Accepted GPU evidence doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`.
- This WR-062 design-first implementation contract.

Promotion command:

```text
task roadmap:promote -- --id WR-062 --state current_candidate --evidence "Completed WR-061 renderer scale residency budgets, completed WR-056 GPU timing evidence, accepted renderer scale/GPU doctrines, and WR-062 design-first visibility and indirect-submission implementation contract."
```

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Current Implementation Readiness

WR-062 has been promoted to `current_candidate`. The bounded implementation is
now selected by the stack coordinator, but code changes remain limited to the
implementation scope and validation in this contract.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-gpu-driven-culling-lod-an
docs-site/src/content/docs/reports/implementation-plans/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/plan.md
docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected owning modules:

- `engine/src/plugins/render/residency/resource.rs`:
  source resident working-set evidence from WR-061. Do not make it own
  visibility or LOD policy beyond passing renderer-owned record identity into
  the next stage.
- `engine/src/plugins/render/inspect/gpu_residency.rs`:
  existing residency inspection DTOs may link to visible/submitted summaries but
  must not absorb all visibility responsibilities.
- `engine/src/plugins/render/inspect`: add narrowly named visibility or
  submitted-work inspection DTOs if public evidence does not fit existing
  residency inspection names.
- `engine/src/plugins/render/renderer/render_flow` and
  `engine/src/plugins/render/graph`: use only if the implementation needs
  compiled draw/dispatch or indirect-command evidence connected to render-flow
  execution. Avoid backend-handle leakage.
- `engine/tests`: focused tests for visible count, culled count, LOD band,
  indirect command count, unsupported capability diagnostics, and no per-entity
  CPU submission path.

## Required Contracts

WR-062 must introduce explicit renderer evidence for:

- resident candidates consumed from WR-061 working-set records;
- visible candidates after renderer culling and renderer LOD band selection;
- culled counts by reason where feasible, such as outside-frustum, below
  screen-size threshold, unsupported path, or over-budget compaction;
- compacted visible-list count and byte range;
- submitted draw, dispatch, or indirect command counts;
- capability state for indirect draw/dispatch, storage buffers, readback, and
  GPU timing evidence used by the selected path;
- degraded-mode diagnostics when GPU-driven submission is unsupported.

The API must fail closed:

- unsupported indirect or storage capability must produce diagnostics instead
  of silently reverting to unbounded per-entity CPU submission;
- visible counts larger than resident counts, submitted counts larger than
  compacted visible counts, missing resident lineage, and missing capability
  status must be diagnostics;
- renderer LOD bands are execution buckets only and must not become product
  semantic LOD or asset streaming policy.

## Non-Goals

WR-062 does not implement:

- sparse SDF page residency, page tables, clipmaps, distance mips, raymarch
  candidate lists, or SDF traversal policy;
- mesh/material asset import, shader material specialization, lighting cache,
  TLAS/BLAS management, temporal reconstruction, or hardware ray queries;
- production hardware profiles, final scale benchmarks, or renderer perfection
  audit evidence;
- product-owned semantic LOD, streaming, freshness, fallback, rebuild policy,
  visibility truth, or gameplay culling authority.

## Implementation Steps

1. Inspect WR-061 residency DTOs, render-flow pass-shape guards, GPU timing
   DTOs, and existing graph/renderer capability diagnostics before adding new
   types.
2. Add narrowly named renderer scale visibility DTOs under the owning renderer
   inspection or scale subsystem. Avoid catch-all helpers or renderer-owned
   product truth.
3. Add deterministic culling/LOD evaluation helpers that consume resident
   renderer records and emit visible, culled, compacted, and submitted counts.
4. Add capability-gated indirect submission diagnostics. Unsupported paths must
   be explicit degraded-mode DTOs.
5. Add focused tests proving count invariants, unsupported diagnostics, and the
   no-unbounded-CPU-submission rule.
6. Update renderer reference docs with the visible/submitted working-set
   vocabulary and the WR-063 evidence boundary.
7. Close the row only after focused tests, docs validation, roadmap validation,
   production validation, and planning validation pass.

## Required Validation

Implementation validation must include:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
cargo test -p engine render_flow
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If no focused visible/indirect scale tests exist yet, add them under
`engine/tests` with names matching the `render_scale` filter.

## Stop Conditions

Stop before implementation if:

- WR-062 has not been applied to active roadmap state and promoted through the
  required roadmap workflow;
- the design would move product semantic LOD, streaming, fallback, freshness,
  or visibility truth into renderer code;
- unsupported GPU-driven capability would silently fall back to unbounded CPU
  draw submission;
- implementation requires a persisted cross-domain ABI without accepted
  ADR/design evidence;
- validation cannot prove visible, compacted, submitted, and measured counts
  remain separate.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md
```

The closeout must record:

- exact changed modules and functions;
- visible, culled, compacted, indirect, and capability diagnostics evidence;
- focused tests and validation output;
- docs updates;
- completion quality and known gaps.

Expected completion quality is `bounded_contract` unless runtime evidence and
benchmarks exceed this contract. `runtime_proven` remains a later
`PT-RENDER-SCALE` claim after WR-063 closes.
