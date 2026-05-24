---
title: WR-074 Renderer Hybrid Ray SDF Raster Runtime Proof Implementation Contract
description: Design-first contract for a portable hybrid raster, SDF raymarch, temporal, and optional ray-query runtime proof example.
status: active
owner: engine
layer: engine-runtime / renderer hybrid runtime proof
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-074 Renderer Hybrid Ray SDF Raster Runtime Proof Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-RT-003` and `WR-074`.
This row must prove a portable hybrid renderer path by composing existing
renderer evidence for raster/material work, SDF raymarch acceleration,
temporal reconstruction inputs, and optional ray-query capability diagnostics.
The proof must show supported and unsupported ray-query states without making
RT hardware a baseline requirement.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-074` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`:
  accepted doctrine for optional ray-query capability, hybrid paths, and
  mandatory non-RT fallback.
- `docs-site/src/content/docs/reports/closeouts/wr-073-renderer-ray-query-capability-and-acceleration-resources/closeout.md`:
  completed optional ray-query capability and acceleration-resource inspection
  evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-065-sdf-raymarch-acceleration-and-candidate-lists/closeout.md`:
  completed conservative SDF raymarch acceleration evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/closeout.md`:
  completed temporal input, history, and dynamic-resolution evidence.
- `engine/examples`: owning example location for the runtime proof.

## Readiness

`task production:plan -- --milestone PM-RENDER-RT-003 --roadmap WR-074`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-074-renderer-hybrid-ray-sdf-raster-runtime-proof/plan.md`.

This contract clears the design-first gap by making completed dependencies,
runtime-proof scope, write scopes, validation, stop conditions, and closeout
quality explicit before implementation.

After applying the intake proposal, WR-074 may be promoted only when:

- `WR-073`, `WR-065`, and `WR-070` remain completed with valid closeout
  evidence;
- `PM-RENDER-RT-003` is active and still selected by the stack coordinator;
- `task production:plan -- --milestone PM-RENDER-RT-003 --roadmap WR-074`
  reports a promotable or promotion-contract action rather than `design_first`;
- roadmap, production, docs, and planning validators pass.

The promotion preflight now reports:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-073:completed`, `WR-065:completed`, and
  `WR-070:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-074 --state current_candidate --evidence "Accepted renderer hardware ray-query doctrine, completed WR-073 ray-query capability and acceleration-resource closeout, completed WR-065 SDF raymarch acceleration closeout, completed WR-070 temporal inputs/history/dynamic-resolution closeout, and active WR-074 hybrid runtime proof implementation contract."
```

WR-074 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-RT-003`.

After promotion, `task production:plan -- --milestone PM-RENDER-RT-003 --roadmap
WR-074` reports:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-073:completed`, `WR-065:completed`, and
  `WR-070:completed`;
- next action: `write_implementation_contract`;
- implementation contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-074-renderer-hybrid-ray-sdf-raster-runtime-proof/plan.md`.

WR-074 implementation may start only after this current-candidate evidence
remains true and the stack coordinator advances from contract writing to
contract execution.

## Governance Decisions

- DDD bounded context owner: `engine/examples` consuming public
  `engine/src/plugins/render` inspection APIs.
- Renderer-owned vocabulary: hybrid runtime proof, raster pass label, SDF
  raymarch evidence, temporal reconstruction evidence, ray-query capability
  evidence, fallback evidence, and timing evidence.
- Source-owner vocabulary: scene semantics, mesh/material truth, SDF product
  truth, camera/exposure truth, product freshness, and fallback legality.
- Translation boundary: WR-074 may compose existing renderer inspection DTOs
  into a finite example output. It must not add backend RT execution, make
  example artifacts runtime dependencies, or move producer truth into the
  example.
- ADR requirement: no ADR is required for a renderer example that consumes
  existing public inspection APIs. Stop for ADR if implementation adds a
  durable hybrid render ABI, changes runtime ownership, or requires RT hardware
  for baseline correctness.
- Team Topologies ownership: enabling example evidence for the complicated
  subsystem renderer platform.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/examples
engine/Cargo.toml
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-hybrid-ray-sdf-raster-runtime-p
docs-site/src/content/docs/reports/implementation-plans/wr-074-renderer-hybrid-ray-sdf-raster-runtime-proof/plan.md
docs-site/src/content/docs/reports/closeouts/wr-074-renderer-hybrid-ray-sdf-raster-runtime-proof/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/examples/render_hybrid_ray_sdf_raster_runtime_proof.rs`: add a
  finite example that builds a hybrid proof report from raster pass labels, SDF
  raymarch evidence, temporal evidence, WR-073 ray-query inspection, fallback
  visual evidence, and timing evidence.
- `engine/Cargo.toml`: register the example.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the example as the preferred portable hybrid proof entry point.

## Non-Goals

- Do not implement hardware ray-query execution, GPU acceleration builds,
  shader tables, denoisers, or production hardware matrix evidence.
- Do not add new renderer runtime APIs unless the example cannot honestly
  compose existing public inspection DTOs.
- Do not claim `runtime_proven` for the RT track; WR-075 remains the production
  evidence milestone.
- Do not make RT hardware mandatory or hide non-RT fallback.

## Required Implementation Shape

WR-074 must provide a finite example that proves:

1. Raster/material pass evidence is present as a labeled renderer pass.
2. SDF raymarch acceleration evidence is consumed.
3. Temporal reconstruction evidence is consumed.
4. WR-073 ray-query capability/acceleration-resource inspection is consumed.
5. Unsupported ray-query hardware produces visible non-RT fallback evidence.
6. Timing evidence separates raster, SDF, temporal, ray-query, and fallback
   labels.
7. The example has a testable `build_report()` path and a stable `cargo run`
   output.

The closeout target is `bounded_contract`. Ray-query production evidence and
hardware matrix proof remain WR-075 scope.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine --example render_hybrid_ray_sdf_raster_runtime_proof
cargo run -p engine --example render_hybrid_ray_sdf_raster_runtime_proof
cargo test -p engine render_ray_query
cargo test -p engine render_sdf_raymarch
cargo test -p engine render_temporal
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Stop Conditions

Stop before product code if:

- `WR-074` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-RT-003 --roadmap WR-074`
  still reports `design_first`, a promotion blocker, or a metadata blocker;
- WR-073, WR-065, or WR-070 closeout evidence is missing or invalid;
- implementation requires RT hardware for the portable baseline;
- implementation would move producer truth or fallback legality into the
  example;
- required validation cannot run or fails;
- closeout evidence cannot honestly claim `bounded_contract`.

## Closeout Requirements

The closeout must include:

- exact changed example, Cargo registration, docs, and metadata paths;
- architecture evidence showing the example consumes renderer evidence without
  owning source truth;
- validation commands and results;
- roadmap and production metadata updates;
- completion quality `bounded_contract`;
- remaining gaps for WR-075 production evidence and final
  `perfectionist_verified` audit.
