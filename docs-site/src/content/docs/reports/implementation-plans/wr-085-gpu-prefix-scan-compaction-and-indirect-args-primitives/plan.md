---
title: WR-085 GPU Prefix Scan Compaction And Indirect Args Primitives Implementation Contract
description: Bounded implementation contract for reusable renderer GPU primitive descriptors, capacity validation, and explicit primitive execution planning.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-085 GPU Prefix Scan Compaction And Indirect Args Primitives Implementation Contract

## Goal

Implement `PM-RENDER-POP-003` / `WR-085` as the reusable renderer GPU
primitive contract slice for procedural populations:

- u32 prefix scan descriptors;
- scatter/compaction descriptors;
- counter reset descriptors;
- indirect draw args generation descriptors;
- real storage-length capacity validation;
- an explicit primitive execution-plan object that later population support can
  compose without making boids-local graph wiring the canonical platform.

This slice targets `bounded_contract`. It must not claim full runtime GPU
execution if the implementation remains descriptors plus execution planning.
Runtime population behavior and boids proof remain `WR-086` and `WR-087`.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`:
  primitive support must be reusable renderer infrastructure, total-count-sized,
  and explicit about unsupported or invalid capacity states.
- `docs-site/src/content/docs/reports/closeouts/wr-084-procedural-builder-and-first-class-draw-sources/closeout.md`:
  first-class draw-source semantics and typed indirect draw args ABI are
  complete.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PM-RENDER-POP-003` is active and links to `WR-085`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  `WR-085` is the current implementation row and depends on archived completed
  `WR-084`.

## Readiness

`task production:plan -- --milestone "PM-RENDER-POP-003" --roadmap "WR-085"`
reported:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-084:completed`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-085-gpu-prefix-scan-compaction-and-indirect-args-primitives/plan.md`.

No ADR is required while primitives stay under
`engine/src/plugins/render/gpu_primitives` and consume renderer-owned resource
handles.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/gpu_primitives/mod.rs`:
  primitive module boundary and exports.
- `engine/src/plugins/render/gpu_primitives/scan.rs`:
  `U32PrefixScanDescriptor`, `PrefixScanMode`, `U32ScanElement`, shared
  validation errors, and capacity helper.
- `engine/src/plugins/render/gpu_primitives/compaction.rs`:
  `U32ScatterDescriptor`.
- `engine/src/plugins/render/gpu_primitives/counters.rs`:
  `CounterResetDescriptor` and `U32Counter`.
- `engine/src/plugins/render/gpu_primitives/draw_args.rs`:
  `IndirectDrawArgsGenerationDescriptor` and generation descriptor validation.
- `engine/src/plugins/render/gpu_primitives/plan.rs`:
  primitive execution-plan steps and resource-access summaries.
- `engine/src/plugins/render/api/handles.rs`:
  `StorageArrayHandle::len` and `StorageArrayHandle::is_empty`.

`DrawIndirectArgs`, `DrawIndexedIndirectArgs`, and
`IndirectDrawArgsBuffer` are now graph draw-source ABI from `WR-084`; this slice
may re-export or consume them but must not move draw-source ownership back into
primitive generation.

## Required Decisions

- Primitive contracts validate against `StorageArrayHandle::len`; duplicated
  caller-supplied capacity must not be trusted when a real typed handle exists.
- Canonical contracts are total-count-sized. Fixed bucket overflow is not a
  production primitive policy.
- Primitive descriptors must reject empty labels, zero work counts, insufficient
  capacities, and invalid aliasing where output and input buffers overlap.
- Primitive planning must produce explicit steps that list resource reads and
  writes for later graph lowering. A vector of ad hoc shader labels is not
  enough.
- This slice is not runtime-proven unless actual reusable GPU primitive
  dispatch is implemented and validated. If dispatch remains deferred, close as
  `bounded_contract` with that gap visible.

## Non-Goals

WR-085 does not implement:

- bounded uniform-grid population flow helpers;
- boids-specific grid shaders or evidence;
- visual/pixel resize evidence;
- production benchmarks or docs closeout;
- unbounded spatial hash or chunked population support;
- final `runtime_proven` track closeout.

## Implementation Steps

1. Keep primitive descriptors in the `gpu_primitives` module with explicit
   labels and typed renderer resource ids.
2. Ensure all descriptors validate real handle lengths through
   `StorageArrayHandle::len`.
3. Add alias validation for scatter/compaction outputs and scan input/output.
4. Keep indirect draw args generation typed against the graph-owned indirect
   draw args ABI from `WR-084`.
5. Add `GpuPrimitiveExecutionPlan` and `GpuPrimitiveStep` so later population
   code can compose reset, scan, scatter, and draw-args generation as a
   reusable renderer primitive plan.
6. Add unit tests for capacity drift, zero work, aliasing, draw args sizes,
   output index validation, and execution-plan resource access.

## Acceptance Criteria

- `StorageArrayHandle` exposes stable length metadata for validation.
- Scan, counter reset, scatter/compaction, and indirect args generation reject
  invalid labels, zero work, insufficient capacity, and invalid aliases.
- Indirect draw argument generation uses typed renderer draw-args structs.
- Primitive execution planning is explicit and reusable by `WR-086`; it is not
  buried inside the boids example.
- Closeout is honest: descriptors and execution planning are
  `bounded_contract` unless reusable GPU dispatch is also implemented.

## Validation

Run:

- `cargo fmt --all -- --check`
- `cargo test -p engine gpu_primitives`
- `cargo test -p engine render_scale`
- `task docs:validate`

Closeout metadata must then pass:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task planning:validate`

## Stop Conditions

Stop before closing WR-085 if:

- primitive descriptors rely on duplicated caller capacity while typed handles
  have real lengths;
- any total-count primitive can silently accept insufficient output capacity;
- aliasing can corrupt input/output buffers without validation;
- primitive planning remains boids-local instead of reusable renderer-owned
  data;
- closeout tries to claim `runtime_proven` without runtime dispatch evidence.

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/wr-085-gpu-prefix-scan-compaction-and-indirect-args-primitives/closeout.md`
with `status: completed`.

Move completed `WR-085` to
`docs-site/src/content/docs/workspace/roadmap-archive.yaml` with:

- `planning_state: completed`;
- `completion_quality: bounded_contract`;
- `known_quality_gaps` listing `WR-086` through `WR-088`, deferred reusable GPU
  dispatch if still deferred, and `PT-RENDER-PERFECTION`;
- `completion_audit` pointing at the WR-085 closeout.

Update `PM-RENDER-POP-003` in
`docs-site/src/content/docs/workspace/production-tracks.yaml` with completed
state, evidence gate, completion audit, and honest known gaps.

Run the phase completion drift-check routine before starting `WR-086`.

## Perfectionist Closeout Audit

WR-085 targets `bounded_contract`, not `runtime_proven` or
`perfectionist_verified`.

Known quality gaps at closeout:

- bounded uniform-grid population support remains `WR-086`;
- boids runtime proof remains `WR-087`;
- evidence, benchmarks, docs, and track closeout remain `WR-088`;
- final no-gap verification remains `PT-RENDER-PERFECTION`;
- if reusable GPU dispatch remains deferred, that must be listed explicitly in
  the closeout and production milestone gaps.
