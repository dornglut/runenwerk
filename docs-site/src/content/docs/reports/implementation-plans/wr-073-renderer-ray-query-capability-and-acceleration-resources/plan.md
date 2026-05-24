---
title: WR-073 Renderer Ray Query Capability And Acceleration Resources Implementation Contract
description: Design-first contract for optional ray-query capability diagnostics and derived acceleration-resource inspection.
status: active
owner: engine
layer: engine-runtime / renderer ray-query capability
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-073 Renderer Ray Query Capability And Acceleration Resources Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-RT-002` and `WR-073`.
This row must add renderer-owned inspection evidence for optional hardware
ray-query capability and derived acceleration resources. The implementation
must prove unsupported states explicitly and keep acceleration resources as
derived renderer execution evidence, not scene, mesh, material, SDF, product,
or fallback truth.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-073` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`:
  accepted doctrine for optional ray-query capability, derived acceleration
  resources, hybrid paths, and mandatory non-RT fallback.
- `docs-site/src/content/docs/reports/closeouts/pm-render-rt-001-hardware-ray-query-doctrine/closeout.md`:
  completed doctrine acceptance evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`:
  completed renderer working-set, product-lineage, and residency budget
  evidence consumed by derived acceleration-resource inspection.
- `engine/src/plugins/render/inspect`: owning renderer inspection boundary for
  capability and acceleration-resource DTOs.

## Readiness

`task production:plan -- --milestone PM-RENDER-RT-002 --roadmap WR-073`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-073-renderer-ray-query-capability-and-acceleration-resources/plan.md`.

This contract clears the design-first gap by making accepted doctrine,
completed dependency evidence, runtime scope, write scopes, validation, stop
conditions, and closeout quality explicit before implementation.

After applying the intake proposal, WR-073 may be promoted only when:

- `PM-RENDER-RT-001` remains completed with valid closeout evidence;
- `WR-061` remains completed with valid working-set closeout evidence;
- `PM-RENDER-RT-002` is active and still selected by the stack coordinator;
- `task production:plan -- --milestone PM-RENDER-RT-002 --roadmap WR-073`
  reports a promotable or promotion-contract action rather than `design_first`;
- roadmap, production, docs, and planning validators pass.

The promotion preflight now reports:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependency: `WR-061:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-073 --state current_candidate --evidence "Accepted renderer hardware ray-query doctrine, completed PM-RENDER-RT-001 hardware ray-query doctrine closeout, completed WR-061 working-set residency closeout, and active WR-073 ray-query capability implementation contract."
```

WR-073 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-RT-002`.

## Governance Decisions

- DDD bounded context owner: `engine/src/plugins/render`.
- Renderer-owned vocabulary: ray-query capability profile, raytracing pipeline
  support, acceleration-resource build/update status, unsupported diagnostic,
  fallback visibility, derived resource lineage, memory estimate, and debug
  identity.
- Source-owner vocabulary: scene semantics, mesh/material truth, SDF query
  policy, product lineage truth, product freshness, fallback legality, and
  gameplay/editor authority.
- Translation boundary: WR-073 may aggregate renderer-facing source lineage
  and residency evidence into derived acceleration-resource inspection. It must
  not expose mutable backend handles or make acceleration resources source
  truth.
- Clean Architecture direction: examples and tests may consume renderer public
  APIs. Renderer inspection must not depend on docs, app/editor state, or
  benchmark artifacts for runtime correctness.
- ADR requirement: no ADR is required for renderer-owned DTOs and fail-closed
  diagnostics. Stop for ADR if implementation introduces a durable
  cross-domain acceleration-resource ABI, moves source truth or fallback
  authority into renderer code, or requires RT hardware for baseline
  correctness.
- Team Topologies ownership: complicated-subsystem renderer platform consuming
  stream-aligned scene, mesh, material, product, SDF, temporal, camera, and
  exposure producers.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-ray-query-capability-and-accele
docs-site/src/content/docs/reports/implementation-plans/wr-073-renderer-ray-query-capability-and-acceleration-resources/plan.md
docs-site/src/content/docs/reports/closeouts/wr-073-renderer-ray-query-capability-and-acceleration-resources/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/inspect/ray_query.rs`: add typed ray-query
  capability, unsupported diagnostic, derived acceleration-resource lineage,
  build/update status, memory estimate, fallback visibility, and inspection
  report DTOs.
- `engine/src/plugins/render/inspect/mod.rs`: export the ray-query inspection
  API.
- `engine/tests/render_ray_query.rs`: guard supported capability readiness,
  unsupported capability diagnostics, missing source lineage failures, backend
  handle privacy, fallback visibility, stale resource invalidation, and memory
  budget diagnostics.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the public inspection DTOs and optional-capability contract.

## Non-Goals

- Do not implement hardware ray-query execution, raytracing pipelines,
  acceleration-structure GPU builds, shader tables, denoisers, or hybrid
  runtime rendering.
- Do not expose backend handles as public authority.
- Do not move scene, mesh, material, product, SDF, temporal, camera, exposure,
  or fallback truth into renderer inspection.
- Do not claim `runtime_proven` or `perfectionist_verified`; this row is a
  bounded implementation contract. Runtime proof remains WR-074 and production
  evidence remains WR-075.
- Do not broaden app/editor UI.

## Required Implementation Shape

WR-073 must provide typed renderer evidence for:

1. Backend ray-query/raytracing capability states: supported, unsupported,
   disabled, and readback/timing pending where relevant.
2. Required capability names and typed unsupported reasons.
3. Derived acceleration-resource source lineage and debug identity.
4. Build/update status, invalidation reason, and memory estimate.
5. Fallback visibility when ray-query or acceleration resources are
   unsupported or disabled.
6. Diagnostics for missing source lineage, unsupported capability without a
   reason, stale derived resources, hidden fallback, and backend handle leaks.

The closeout target is `bounded_contract`. Any remaining gap must stay visible
and prevent the RT track from claiming `runtime_proven` until WR-074 and WR-075
close.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_ray_query
cargo test -p engine render_runtime_inspect
cargo test -p engine render_resource_model
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

- `WR-073` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-RT-002 --roadmap WR-073`
  still reports `design_first`, a promotion blocker, or a metadata blocker;
- PM-RENDER-RT-001 or WR-061 closeout evidence is missing or invalid;
- implementation would require RT hardware for baseline correctness;
- implementation would expose mutable backend handles as public authority;
- implementation would move producer truth or fallback legality into renderer
  inspection;
- required validation cannot run or fails;
- closeout evidence cannot honestly claim `bounded_contract`.

## Closeout Requirements

The closeout must include:

- exact changed modules, functions, tests, and docs;
- architecture evidence showing acceleration resources are renderer-derived
  evidence, not source truth;
- validation commands and results;
- public API/doc updates;
- roadmap and production metadata updates;
- completion quality `bounded_contract`;
- remaining gaps for WR-074 runtime proof, WR-075 production evidence, and
  final `perfectionist_verified` audit.
