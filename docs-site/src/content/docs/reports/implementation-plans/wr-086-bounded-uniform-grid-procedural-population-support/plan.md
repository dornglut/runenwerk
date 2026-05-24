---
title: WR-086 Bounded Uniform Grid Procedural Population Support Implementation Contract
description: Bounded implementation contract for reusable 2D wrapping uniform-grid population planning and validation.
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

# WR-086 Bounded Uniform Grid Procedural Population Support Implementation Contract

## Goal

Implement `PM-RENDER-POP-004` / `WR-086` as the reusable renderer procedural
population slice for bounded 2D wrapping uniform-grid populations.

The slice must make the canonical population pipeline explicit:

1. clear counts;
2. count cells;
3. scan counts;
4. reset scatter cursors;
5. scatter sorted indices;
6. simulate neighbors over adjacent wrapped cells;
7. publish/draw.

This slice targets `bounded_contract`. It must not claim boids runtime proof,
visual resize proof, production benchmarks, or full track closeout.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`:
  bounded uniform grid is the first canonical population path; spatial hash and
  chunked unbounded support are later milestones.
- `docs-site/src/content/docs/reports/closeouts/wr-085-gpu-prefix-scan-compaction-and-indirect-args-primitives/closeout.md`:
  primitive descriptors, capacity validation, and explicit primitive execution
  planning are complete.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PM-RENDER-POP-004` links to `WR-086`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  `WR-086` is the current implementation row and depends on archived completed
  `WR-085`.

## Readiness

`task production:plan -- --milestone "PM-RENDER-POP-004" --roadmap "WR-086"`
reported:

- production milestone state: `designing`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-085:completed`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-086-bounded-uniform-grid-procedural-population-support/plan.md`.

The milestone state starts as `designing`, but the planner confirmed this row
can write the implementation contract and proceed under the active doctrine.
Closeout must promote the milestone to completed only if the reusable
population support is present and validated.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/procedural/population/mod.rs`:
  population module boundary and exports.
- `engine/src/plugins/render/procedural/population/uniform_grid.rs`:
  `BoundedUniformGrid2dConfig`, wrapping cell helpers,
  `BoundedUniformGrid2dBuildPlan`, canonical stage plan, resource validation,
  and primitive-plan composition.

This slice may consume `engine/src/plugins/render/gpu_primitives` types from
`WR-085`, but it must not add boids-specific renderer graph code under the
population module.

## Required Decisions

- Use bounded 2D wrapping grid semantics first.
- Reject zero dimensions, zero agent capacity, and grid cell-count overflow.
- Use total-count-sized buffers for counts, offsets, cursors, and sorted
  indices.
- Provide a reusable canonical stage plan with stable labels and dependencies.
- Compose WR-085 primitive planning for reset and scan work where reusable.
- Do not use fixed-capacity buckets that silently drop dense cells.
- Spatial hash and chunked unbounded populations are explicitly out of scope.

## Non-Goals

WR-086 does not implement:

- boids-specific graph wiring or shader entrypoints;
- visual/pixel resize evidence;
- production benchmark updates;
- spatial hash, chunked unbounded worlds, quadtree, BVH, or density-field
  approximations;
- graph multi-step fixed-update scheduling;
- final `runtime_proven` track closeout.

## Implementation Steps

1. Tighten `BoundedUniformGrid2dConfig` validation to reject cell-count overflow
   instead of saturating silently.
2. Add wrapped cell-index and adjacent-cell helpers for 2D wrapping grids.
3. Expand `BoundedUniformGrid2dBuildPlan` so it records:
   cell resources, reset-counts primitive, scan-counts primitive, reset-cursors
   primitive, primitive execution plan, and canonical stage plan.
4. Add canonical stage descriptors for:
   clear counts, count cells, scan counts, reset cursors, scatter sorted
   indices, simulate neighbors, and publish/draw.
5. Validate total-count capacities using real `StorageArrayHandle::len`.
6. Add tests for invalid dimensions, zero capacity, overflow, total-count
   sizing, wrapping neighbors, and canonical stage order.

## Acceptance Criteria

- Reusable bounded 2D wrapping grid support exists under
  `engine/src/plugins/render/procedural/population`.
- The canonical stage order is represented by renderer-owned data, not only by
  `engine/examples/boids_render_flow/rendering/graph.rs`.
- Counts, offsets, cursors, and sorted indices validate against total-count
  capacities.
- Grid cell overflow is explicit.
- Spatial hash/chunked unbounded support remains deferred.

## Validation

Run:

- `cargo fmt --all -- --check`
- `cargo test -p engine procedural`
- `cargo test -p engine render_flow`
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

Stop before closing WR-086 if:

- boids graph wiring remains the only source of the canonical grid stage order;
- any buffer capacity can silently overflow or drift below total-count
  requirements;
- wrapping behavior is implicit shader-only behavior with no reusable renderer
  contract;
- closeout tries to claim boids runtime proof or final `runtime_proven` track
  status.

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/wr-086-bounded-uniform-grid-procedural-population-support/closeout.md`
with `status: completed`.

Move completed `WR-086` to
`docs-site/src/content/docs/workspace/roadmap-archive.yaml` with:

- `planning_state: completed`;
- `completion_quality: bounded_contract`;
- `known_quality_gaps` listing `WR-087`, `WR-088`, reusable GPU dispatch if
  still deferred, and `PT-RENDER-PERFECTION`;
- `completion_audit` pointing at the WR-086 closeout.

Update `PM-RENDER-POP-004` in
`docs-site/src/content/docs/workspace/production-tracks.yaml` with completed
state, evidence gate, completion audit, and honest known gaps.

Run the phase completion drift-check routine before starting `WR-087`.

## Perfectionist Closeout Audit

WR-086 targets `bounded_contract`, not `runtime_proven` or
`perfectionist_verified`.

Known quality gaps at closeout:

- boids runtime proof remains `WR-087`;
- evidence, benchmarks, docs, and track closeout remain `WR-088`;
- final no-gap verification remains `PT-RENDER-PERFECTION`;
- reusable primitive shader dispatch remains a visible gap unless implemented
  in a later slice.
