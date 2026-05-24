---
title: WR-083 Renderer Procedural Population Doctrine And Track Activation Contract
description: Workflow recovery, governance summary, draft-branch audit, and WR-sliced implementation plan for the renderer procedural population production track.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports: []
---

# WR-083 Renderer Procedural Population Doctrine And Track Activation Contract

## Status

This contract recovers the renderer procedural population draft branch into the
proper production-track workflow.

`WR-083` is doctrine and track activation only. It must not be used as a
monolithic implementation row for procedural authoring, GPU primitives,
population grids, boids upgrades, evidence, benchmarks, or closeout.

The existing branch
`codex/boids-procedural-population-production` is implementation draft evidence
only. It is not accepted production evidence and must not be merged as one
broad change.

## Workflow Commands Run

- `task ai:architecture-governance -- --task "Renderer procedural population platform production track recovery and implementation planning" --scope "engine/src/plugins/render, engine/examples/boids_render_flow, assets/shaders/boids_*, docs-site/src/content/docs/workspace, docs-site/src/content/docs/design"`
- `task production:plan -- --milestone "PM-RENDER-POP-001" --roadmap "WR-083"`
- `task production:plan -- --milestone "PM-RENDER-POP-002" --roadmap "WR-084"`
- `task production:plan -- --milestone "PM-RENDER-POP-003" --roadmap "WR-085"`
- `task production:plan -- --milestone "PM-RENDER-POP-004" --roadmap "WR-086"`
- `task production:plan -- --milestone "PM-RENDER-POP-005" --roadmap "WR-087"`
- `task production:plan -- --milestone "PM-RENDER-POP-006" --roadmap "WR-088"`

Architecture governance classified the recovery as review-only until the
decision contract is written. `PM-RENDER-POP-001` / `WR-083` returned
`write_promotion_contract`; the remaining linked rows returned
`write_implementation_contract`.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`
  owns the active doctrine.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-RENDER-PROCEDURAL-POPULATION` and milestones
  `PM-RENDER-POP-001` through `PM-RENDER-POP-006`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-083`
  through `WR-088`.
- Existing accepted renderer designs remain binding:
  `renderer-gpu-evidence-and-procedural-visuals-design.md`,
  `renderer-scale-residency-and-gpu-driven-visibility-design.md`,
  `render-product-graph-platform-design.md`, and
  `render-production-readiness-and-inspection-design.md`.

## Bounded Context And Ownership

Bounded context owner: engine render plugin.

Owning module root:
`engine/src/plugins/render/mod.rs`.

Vocabulary:

- procedural pass authoring;
- direct draw source;
- indirect draw source;
- GPU primitive;
- bounded uniform grid;
- population build plan;
- fixed-step boids evidence;
- explicit unsupported capability diagnostic.

Translation boundaries:

- Renderer owns derived GPU execution data, render-flow graph semantics,
  primitive buffers, diagnostics, and evidence.
- Domain, gameplay, product, world, asset, editor, and streaming owners retain
  source truth, authored semantic identity, authority, fallback legality,
  freshness, and residency intent.

Dependency direction is acceptable only while procedural population support
stays inside `engine/src/plugins/render` and examples consume renderer APIs.
Moving population source truth into renderer, or moving renderer primitive
contracts into foundation/domain without a broader design, would require a new
ADR or accepted design update.

Team Topologies label: complicated-subsystem renderer owner, with a
stream-aligned renderer example for boids.

## Draft Branch Audit

### Valid Work To Salvage

- `engine/src/plugins/render/procedural/authoring.rs::ProceduralPassBuilder`
  is the right procedural-owned extension surface. It does not expose
  `GraphicsPassBuilder` to procedural users.
- `engine/src/plugins/render/procedural/lowering.rs::lower_procedural_pass`
  moves procedural lowering out of the old builder file and keeps graphics
  lowering internal.
- `engine/src/plugins/render/api/flow.rs::RenderFlow::procedural_pass_builder`
  preserves the existing simple `procedural_pass` path while adding advanced
  authoring.
- `engine/src/plugins/render/graph/pass_graph.rs::RenderDrawSource` correctly
  separates direct and indirect draw intent.
- `engine/src/plugins/render/api/passes.rs::GraphicsPassBuilder::draw_indirect`
  and
  `engine/src/plugins/render/renderer/render_flow/execute_passes.rs::encode_graphics_pass`
  make indirect submission inspectable instead of only a sidecar buffer.
- `engine/src/plugins/render/api/handles.rs::StorageArrayHandle` now carries
  length, which lets primitive contracts validate real buffer capacity instead
  of trusting duplicated capacity arguments.
- `engine/examples/boids_render_flow/rendering/state.rs::BoidAgent` and
  `engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState`
  express fixed-step state and smoothed visual heading separately from
  simulation velocity.
- `assets/shaders/boids_compose.wgsl::vs_main` uses surface-aware pixel-to-clip
  offsets, which is the right long-term direction for resize correctness.

### Workflow Violations

- Product code for `WR-084` through `WR-088` was implemented before the
  `WR-083` promotion/readiness contract existed.
- The draft branch combines doctrine, graph API, primitive contracts,
  population support, boids shader work, docs, benchmarks, and generated
  roadmap artifacts in one review unit.
- `WR-083` remains `ready_next` while dependent `WR-084` through `WR-088` are
  already `current_candidate` in the draft roadmap state. Do not close or stage
  implementation slices before `WR-083` is promoted and closed as
  `bounded_contract`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml::WR-088` currently
  scopes docs to `docs-site/src/content/docs/engine/plugins/render`, but the
  draft changes docs under
  `docs-site/src/content/docs/engine/reference/plugins/render`. The write scope
  must be corrected before any WR-088 closeout.
- `PM-RENDER-POP-004` and `PM-RENDER-POP-006` are still `designing` while their
  linked WR rows are `current_candidate`. This may be intentional, but the
  implementation contracts must explicitly reconcile the milestone state before
  code is accepted.

### Prototype-Level Work

- `engine/src/plugins/render/gpu_primitives/*` provides typed descriptors and
  validation, but reusable runtime primitive execution is not complete.
  `WR-085` must not claim the primitive platform is complete while scan and
  scatter remain boids-local shader logic.
- `assets/shaders/boids_compute.wgsl::scan_counts` performs a single-invocation
  linear scan over cells. That is acceptable as draft evidence for a small
  bounded grid, but it is not the long-term scalable prefix-scan primitive.
- `engine/src/plugins/render/procedural/population/uniform_grid.rs::BoundedUniformGrid2dConfig`
  and related plan structs validate contracts, but
  `engine/examples/boids_render_flow/rendering/graph.rs::build_render_flow`
  still manually wires the concrete grid passes. `WR-086` must move reusable
  population flow authoring out of the example.
- `assets/shaders/boids_compute.wgsl::cs_main` guards
  `sorted_index < boid_count`, but there is no invariant-mismatch counter or
  diagnostic path if scatter capacity assumptions fail. The canonical path is
  total-count-sized, but fail-closed evidence is still missing.
- The direct/indirect draw contract is implemented, but the boids production
  path still uses direct draw. `WR-084` can close graph semantics; indirect
  runtime proof remains an explicit test/evidence requirement, not implied by
  boids.
- `engine/benches/render_flow_planning.rs::build_procedural_boids_flow` does
  not benchmark the actual production grid flow. `WR-088` must add scan, grid
  build, boids production flow, and evidence-report cases that match the final
  runtime shape.
- Resize/aspect proof is currently shader and unit-test level. There is no
  automated screenshot or pixel evidence proving no aspect skew across resized
  surfaces.

### Missing Evidence

- No `WR-083` closeout exists.
- No implementation contracts exist for `WR-084` through `WR-088`.
- No reusable GPU primitive runtime execution closeout exists.
- No visual resize screenshot/pixel evidence exists for boids.
- No invariant mismatch diagnostic evidence exists for grid scatter overflow or
  capacity drift.
- No closeout report under `docs-site/src/content/docs/reports/closeouts` ties
  the track to `runtime_proven`.
- No final phase completion drift check has been run for any slice.

## Recovery Decision

Salvage and split the existing branch. Do not discard it, and do not merge it
as a single branch.

Use `codex/boids-procedural-population-production` as an integration reference
and patch source. Create reviewable WR-sized branches or commits from it in
dependency order. A slice may only be staged when its implementation contract,
scope, evidence, and validation are complete.

If a slice cannot be made honest without broadening its scope, stop and create a
new WR row or split the row before implementation continues.

## WR-Sliced Plan

### WR-083 - Doctrine And Track Activation

Owning files/modules:

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`
- `docs-site/src/content/docs/design/active/README.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- generated production and roadmap docs under
  `docs-site/src/content/docs/workspace`
- this contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-083-renderer-procedural-population-doctrine-and-track-activation/plan.md`

Required action:

1. Promote `WR-083` only after this contract is accepted:
   `task roadmap:promote -- --id WR-083 --state current_candidate --evidence "<accepted evidence>"`.
2. Keep `WR-083` code-free.
3. Close `WR-083` as `bounded_contract` only after generated roadmap and
   production docs are current and validation passes.

Validation:

- `task production:render`
- `task production:validate`
- `task production:check`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task planning:validate`

### WR-084 - Procedural Builder And Draw Sources

Owning files/modules:

- `engine/src/plugins/render/procedural/authoring.rs::ProceduralPassBuilder`
- `engine/src/plugins/render/procedural/lowering.rs::lower_procedural_pass`
- `engine/src/plugins/render/procedural/mod.rs`
- `engine/src/plugins/render/api/flow.rs::RenderFlow::procedural_pass_builder`
- `engine/src/plugins/render/api/passes.rs::GraphicsPassBuilder::draw_indirect`
- `engine/src/plugins/render/graph/pass_graph.rs::RenderDrawSource`
- `engine/src/plugins/render/graph/execution_plan.rs`
- `engine/src/plugins/render/graph/validation.rs`
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs::encode_graphics_pass`

Required action:

1. Write the implementation contract at
   `docs-site/src/content/docs/reports/implementation-plans/wr-084-procedural-builder-and-first-class-draw-sources/plan.md`.
2. Split only the procedural authoring and draw-source graph changes.
3. Preserve `.draw(...)` as the direct path.
4. Require validation failures for undeclared indirect args buffers and invalid
   indirect offsets.
5. Do not include boids grid or shader changes.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine procedural`
- `cargo test -p engine render_flow`

### WR-085 - GPU Primitive Contracts And Runtime Execution Plan

Owning files/modules:

- `engine/src/plugins/render/gpu_primitives/mod.rs`
- `engine/src/plugins/render/gpu_primitives/scan.rs::U32PrefixScanDescriptor`
- `engine/src/plugins/render/gpu_primitives/compaction.rs::U32ScatterDescriptor`
- `engine/src/plugins/render/gpu_primitives/counters.rs::CounterResetDescriptor`
- `engine/src/plugins/render/gpu_primitives/draw_args.rs::IndirectDrawArgsGenerationDescriptor`
- `engine/src/plugins/render/api/handles.rs::StorageArrayHandle`

Required action:

1. Write the implementation contract at
   `docs-site/src/content/docs/reports/implementation-plans/wr-085-gpu-prefix-scan-compaction-and-indirect-args-primitives/plan.md`.
2. Decide in the contract whether `WR-085` closes only typed primitive
   contracts or also runtime execution. Long-term acceptance should include a
   reusable primitive execution plan before boids relies on grid behavior.
3. Keep primitive contracts renderer-owned unless a later design moves them to
   a cross-domain layer.
4. Add fail-closed diagnostics for capacity drift and unsupported primitive
   execution/readback/indirect support.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine gpu_primitives`
- `cargo test -p engine render_scale`
- targeted benchmark for scan and primitive planning after runtime execution
  exists

### WR-086 - Bounded Uniform Grid Population Support

Owning files/modules:

- `engine/src/plugins/render/procedural/population/mod.rs`
- `engine/src/plugins/render/procedural/population/uniform_grid.rs::BoundedUniformGrid2dConfig`
- future renderer-owned population flow authoring under
  `engine/src/plugins/render/procedural/population`

Required action:

1. Write the implementation contract at
   `docs-site/src/content/docs/reports/implementation-plans/wr-086-bounded-uniform-grid-procedural-population-support/plan.md`.
2. Build reusable population support that composes the primitive contracts:
   clear counts, count cells, scan counts, reset cursors, scatter sorted
   indices, adjacent-cell neighbor iteration, publish/draw.
3. Do not leave the canonical population path as manual boids graph wiring.
4. Keep unbounded spatial hash/chunked populations out of scope.
5. Add explicit invariant diagnostics for capacity mismatch and impossible
   grid dimensions.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine procedural`
- `cargo test -p engine render_flow`
- population-specific tests for total-count capacity and no silent overflow

### WR-087 - Boids Production Upgrade

Owning files/modules:

- `engine/examples/boids_render_flow/rendering/state.rs::BoidAgent`
- `engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState`
- `engine/examples/boids_render_flow/rendering/graph.rs::build_render_flow`
- `engine/examples/boids_render_flow/rendering/evidence.rs::production_evidence_report`
- `assets/shaders/boids_compute.wgsl::cs_main`
- `assets/shaders/boids_compose.wgsl::vs_main`

Required action:

1. Write the implementation contract at
   `docs-site/src/content/docs/reports/implementation-plans/wr-087-boids-render-flow-production-upgrade/plan.md`.
2. Make boids consume the reusable WR-086 population support, not duplicate it.
3. Keep fixed-step limitations explicit.
4. Prove no render-stage storage loop over all boids and no production
   O(n^2) neighbor loop.
5. Add visual evidence for resize/aspect and heading stability.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine --example boids_render_flow`
- `cargo run -p engine --example boids_render_flow -- --evidence`
- visual resize/pixel evidence command once the harness exists

### WR-088 - Evidence, Benchmarks, Docs, And Closeout

Owning files/modules:

- `engine/benches/render_flow_planning.rs::build_procedural_boids_flow`
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
- `docs-site/src/content/docs/reports/closeouts`
- generated production and roadmap docs under
  `docs-site/src/content/docs/workspace`

Required action:

1. Write the implementation contract at
   `docs-site/src/content/docs/reports/implementation-plans/wr-088-procedural-population-evidence-benchmarks-docs-and-closeout/plan.md`.
2. Fix the WR-088 docs write scope before closeout so it includes
   `docs-site/src/content/docs/engine/reference/plugins/render`.
3. Add benchmark cases that match the real final runtime shape: scan, grid
   build, boids production flow, and evidence reporting.
4. Add closeout evidence proving:
   no render-stage storage loop, no production O(n^2) neighbor loop, no aspect
   skew, no silent grid overflow, explicit unsupported diagnostics, bounded
   submitted work, and stable production evidence.
5. Close at `runtime_proven` only. Do not claim `perfectionist_verified`.

Validation:

- `cargo fmt --all -- --check`
- `cargo bench -p engine --bench render_flow_planning`
- `task docs:validate`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task planning:validate`

## Stop Conditions

Stop before product code if:

- `WR-083` is not promoted and closed as doctrine/track activation;
- an implementation slice requires write scopes outside its WR row;
- primitive execution remains boids-local but a slice attempts to claim
  reusable GPU primitive completion;
- capacity drift or unsupported capability states would be silent;
- visual correctness depends on pixels but only shader-string tests exist;
- a slice would need unbounded spatial hash/chunked population behavior;
- a slice attempts to claim `perfectionist_verified`.

## Closeout Requirements

Closeout for the recovery action must report:

- governance and production-plan commands run;
- the decision to salvage and split the draft branch;
- the list of WR implementation contracts still required;
- validation commands and results for this planning artifact;
- remaining blockers before any implementation slice can be staged.

`WR-083` completion quality can only be `bounded_contract`. Runtime evidence
belongs to later rows, and final no-gap renderer proof remains
`PT-RENDER-PERFECTION`.
