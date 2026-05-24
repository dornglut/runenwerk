---
title: WR-078 Renderer Product Visuals Animation Deformation And Evidence Implementation Contract
description: Design-first contract for animation/deformation producer integration and cross-family product visual evidence.
status: active
owner: engine
layer: engine-runtime / product-render-integration
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/renderer-product-visual-producers-platform-design.md
  - ../../../design/accepted/renderer-mesh-material-shader-asset-handoff-design.md
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-078 Renderer Product Visuals Animation Deformation And Evidence Implementation Contract

## Goal

Prepare the hardening slice for `PM-RENDER-PRODUCT-VISUALS-004` and `WR-078`.

This row must close the product visual producer track by integrating the
existing animation/deformation renderer handoff with the particle/VFX and world
visual producer contracts, then producing representative evidence that the
families can be discovered, diagnosed, documented, and validated together.

This is a design-first, promotion-readiness, and current-candidate
implementation contract. Product code changes are authorized only after
`WR-078` is applied to the active roadmap, promoted by the roadmap workflow,
selected by the stack coordinator, `PM-RENDER-PRODUCT-VISUALS-004` is moved to
`active`, and `task production:plan` reports an implementation action for the
active milestone.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-product-visual-producers-platform-design.md`:
  accepted doctrine for product-owned visual producers and renderer execution
  APIs.
- `docs-site/src/content/docs/reports/closeouts/wr-076-renderer-product-visuals-particles-vfx-trails-and-decals/closeout.md`:
  completed particle/VFX/trail/decal producer contract evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-077-renderer-product-visuals-vegetation-water-atmosphere-weather-and-field-visuals/closeout.md`:
  completed world visual producer contract evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-067-renderer-mesh-material-shader-asset-handoff/closeout.md`:
  completed mesh/material handoff evidence for deformation-adjacent renderer
  inputs.
- `docs-site/src/content/docs/reports/closeouts/wr-065-sdf-raymarch-acceleration-and-candidate-lists/closeout.md`:
  completed SDF raymarch acceleration evidence for field visual consumption.
- `engine/src/plugins/render/frame/contributions.rs`: existing
  `PreparedDeformationFeatureContribution` and `PreparedDeformationStream`
  handoff types.
- `engine/src/plugins/render/runtime/frame_prepare.rs`: existing
  `PreparedDeformationFeatureResource` ingestion path.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  public renderer product visual API reference surface.

## Readiness

Initial `task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-004
--roadmap WR-078` reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- roadmap dependencies: none;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-078-renderer-product-visuals-animation-deformation-and-evidence/plan.md`.

This contract clears the design-first gap by making dependencies, ownership,
evidence scope, write scopes, validation, stop conditions, and closeout quality
explicit before implementation.

After applying the intake proposal, `task production:plan -- --milestone
PM-RENDER-PRODUCT-VISUALS-004 --roadmap WR-078` reported:

- production milestone state: `designing`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-065`, `WR-067`, `WR-076`, and `WR-077`, all
  completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested promotion command:
  `task roadmap:promote -- --id WR-078 --state current_candidate --evidence "<accepted evidence>"`.

WR-078 may be promoted only when:

- `WR-076` and `WR-077` remain completed with valid closeout evidence;
- `PM-RENDER-MESH-MATERIAL-002` and `PM-RENDER-SDF-003` remain completed with
  valid evidence;
- `PM-RENDER-PRODUCT-VISUALS-004` is active and still selected by the stack
  coordinator;
- `task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-004 --roadmap
  WR-078` reports a promotable or promotion-contract action rather than
  `design_first`;
- roadmap, production, docs, and planning validators pass.

Accepted promotion evidence should cite the accepted product visual producer
doctrine, active WR-078 implementation contract, completed WR-076
particle/VFX closeout, completed WR-077 world visual closeout, completed WR-067
mesh/material handoff closeout, and completed WR-065 SDF acceleration closeout.

After promotion, `task production:plan -- --milestone
PM-RENDER-PRODUCT-VISUALS-004 --roadmap WR-078` reported:

- production milestone state: `designing`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-065`, `WR-067`, `WR-076`, and `WR-077`, all
  completed;
- next action: `write_implementation_contract`;
- readiness note: this WR row is current-candidate eligible and the contract
  may plan implementation and closeout.

The next metadata action is to move `PM-RENDER-PRODUCT-VISUALS-004` from
`designing` to `active` only after this implementation contract validates.

## Architecture Governance

- Recommendation: use this contract as the design gate and expand WR-078 beyond
  docs-only scope. A docs-only slice cannot honestly close the
  `runtime_proven` product visual evidence outcome.
- DDD bounded context owner: `engine/src/plugins/render` owns renderer evidence,
  prepared deformation handoff inspection, product visual public docs, examples,
  benchmark/report artifacts, and closeout diagnostics.
- Product-owner vocabulary: animation graph truth, skeleton/pose authority,
  deformation source meaning, product freshness, fallback legality, material
  binding semantics, and field/visual authoring workflows.
- Renderer-owned vocabulary: prepared deformation stream, prepared feature
  contribution, product visual evidence report, public API example, benchmark
  command, artifact path, diagnostic state, and completion-quality claim.
- Translation boundary: WR-078 may aggregate renderer-owned inspection and
  example evidence for particle/VFX, world visual, and deformation handoff
  families. It must not make documentation, artifacts, or examples authoritative
  product truth.
- Clean Architecture check: examples, tests, benchmark runners, and docs may
  consume renderer public APIs. Domain and foundation crates must not depend on
  engine renderer internals, docs, benchmarks, or artifact files.
- ADR requirement: no ADR is required for renderer evidence aggregation,
  examples, docs, or closeout metadata. Stop for ADR if implementation changes
  animation/deformation ownership, creates a durable cross-domain animation ABI,
  or moves fallback authority into renderer policy.
- ATAM-lite: the quality tension is runtime evidence versus scope size. The
  decision is to add focused renderer evidence and examples, not a broad
  animation system or product-domain simulation implementation.
- Strangler Fig: not applicable unless the existing deformation handoff path is
  replaced. If replacement is needed, keep old and new paths side by side and
  stop for migration design.
- Fitness functions: focused tests and examples must prove particle/VFX, world
  visual, and deformation evidence is present, missing evidence fails closed,
  public docs describe ownership boundaries, and roadmap/production validators
  keep known gaps visible.
- Team Topologies ownership: complicated-subsystem renderer platform work with
  stream-aligned product teams as evidence consumers.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
engine/examples
engine/benches
engine/benchmark-artifacts
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/benchmarks/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-product-visuals-animation-defor
docs-site/src/content/docs/reports/implementation-plans/wr-078-renderer-product-visuals-animation-deformation-and-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-078-renderer-product-visuals-animation-deformation-and-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/inspect/product_visual_evidence.rs`: aggregate
  particle/VFX, world visual, and deformation handoff evidence into a
  fail-closed product visual evidence report.
- `engine/src/plugins/render/inspect/mod.rs`: export the product visual evidence
  API.
- `engine/tests/render_product_visual_evidence.rs`: guard ready evidence,
  missing family evidence, fallback-only claims, unconsumed deformation handoff,
  missing benchmark/artifact paths, and ownership-boundary diagnostics.
- `engine/examples/render_product_visual_evidence.rs`: print the canonical
  product visual evidence summary.
- `engine/benches/render_flow_planning.rs`: add a focused product visual
  evidence benchmark case only if the benchmark runner already supports the
  needed renderer inspection path.
- `engine/benchmark-artifacts/render-product-visual-evidence/`: store raw
  benchmark/evidence artifact placeholders.
- `docs-site/src/content/docs/reports/benchmarks/render/product-visual-evidence.md`:
  store the human-readable evidence report.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the final product visual evidence and deformation handoff contract.

## Non-Goals

- Do not implement animation graphs, skinning solvers, cloth/physics
  deformation, gameplay animation authority, product-domain VFX/world truth, or
  editor authoring workflows.
- Do not make benchmark artifacts, examples, docs, or reports runtime
  dependencies.
- Do not move product truth, fallback legality, freshness, rebuild policy, or
  user-facing meaning into renderer code.
- Do not claim `perfectionist_verified`; final no-gap renderer audit remains
  `PT-RENDER-PERFECTION` scope.

## Required Implementation Shape

WR-078 must provide typed renderer evidence for:

1. Consumption of WR-076 particle/VFX producer closeout and current public API
   evidence.
2. Consumption of WR-077 world visual producer closeout and current public API
   evidence.
3. Existing deformation handoff evidence through
   `PreparedDeformationFeatureContribution` and `PreparedDeformationStream`.
4. Runtime/example evidence references, benchmark commands, raw artifact paths,
   and human report paths.
5. Fail-closed diagnostics for missing family evidence, missing deformation
   evidence, fallback-only claims, missing benchmark/artifact paths, and any
   attempt to treat renderer evidence as product truth.
6. Public docs explaining product-domain ownership and renderer execution
   boundaries across particle/VFX, world visuals, and deformation.

The closeout target is `runtime_proven` for the product visual track only if
examples, docs, benchmark/report artifacts, and diagnostics are all present. If
that cannot be proven, the closeout must remain `bounded_contract` and the
production track must stay active with visible gaps.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_product_visual_particles
cargo test -p engine render_product_visual_world
cargo test -p engine render_product_visual_evidence
cargo test -p engine render_runtime_inspect
cargo test -p engine --example render_product_visual_evidence
cargo run -p engine --example render_product_visual_evidence
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If implementation adds or changes benchmark coverage, also run:

```text
cargo bench -p engine --bench render_flow_planning
```

## Stop Conditions

Stop before product code if:

- `WR-078` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-004 --roadmap
  WR-078` still reports `design_first`, a promotion blocker, or a metadata
  blocker;
- WR-076 or WR-077 closeout evidence is missing or invalid;
- mesh/material or SDF prerequisite evidence is missing or invalid;
- the implementation would require renderer code to own animation graph,
  deformation, VFX, water/weather, field, or product semantic truth;
- runtime/evidence requirements cannot be validated without broad unrelated
  renderer or product-domain implementation.

## Closeout Requirements

Closeout must update:

- `docs-site/src/content/docs/reports/closeouts/wr-078-renderer-product-visuals-animation-deformation-and-evidence/closeout.md`;
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` and
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` as needed;
- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- renderer public docs under
  `docs-site/src/content/docs/engine/reference/plugins/render`.

Required validation before closeout:

```text
cargo fmt
cargo test -p engine render_product_visual_particles
cargo test -p engine render_product_visual_world
cargo test -p engine render_product_visual_evidence
cargo test -p engine render_runtime_inspect
cargo test -p engine --example render_product_visual_evidence
cargo run -p engine --example render_product_visual_evidence
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Perfectionist Closeout Audit

WR-078 should close the product visual track as `runtime_proven` only if the
evidence chain covers representative particle/VFX, world visual, and
deformation families with examples, docs, benchmark/report artifacts, and
diagnostics.

Known gaps that may remain visible:

- final no-gap stack verification remains `PT-RENDER-PERFECTION` scope;
- product-domain animation, water/weather, vegetation, and VFX authoring truth
  remains outside renderer scope;
- backend-specific visual quality, vendor certification, and hardware-specific
  product visual claims remain outside this slice unless explicitly proven.

The closeout must not claim `perfectionist_verified` unless the final renderer
audit has completed with no known quality gaps.
