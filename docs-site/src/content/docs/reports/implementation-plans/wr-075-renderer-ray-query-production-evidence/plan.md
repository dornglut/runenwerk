---
title: WR-075 Renderer Ray Query Production Evidence Implementation Contract
description: Design-first contract for optional ray-query production evidence, hardware matrix, fallback evidence, docs, and diagnostics.
status: active
owner: engine
layer: engine-runtime / renderer optional ray-query production evidence
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-075 Renderer Ray Query Production Evidence Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-RT-004` and `WR-075`.
This row must close optional ray-query production evidence by documenting the
supported, unsupported, disabled, and fallback states that the renderer exposes
through WR-073 inspection DTOs and the WR-074 hybrid proof example.

The target is `runtime_proven` for the optional RT production evidence packet,
not mandatory hardware ray-query execution. The portable renderer baseline must
remain valid without RT hardware.

This is a design-first and promotion-readiness contract. Product changes are
authorized only after `WR-075` is applied to the active roadmap, promoted by the
roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`:
  accepted doctrine for optional ray-query capability, derived acceleration
  resources, hybrid paths, and mandatory non-RT fallback.
- `docs-site/src/content/docs/reports/closeouts/wr-073-renderer-ray-query-capability-and-acceleration-resources/closeout.md`:
  completed capability and acceleration-resource inspection evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-074-renderer-hybrid-ray-sdf-raster-runtime-proof/closeout.md`:
  completed hybrid raster/SDF/temporal/ray-query proof and visible fallback
  evidence.
- `engine/examples/render_hybrid_ray_sdf_raster_runtime_proof.rs`: existing
  portable example that produces stable runtime evidence output.
- `docs-site/src/content/docs/engine/reference/plugins/render`: owning docs
  location for renderer public reference and production evidence guidance.

## Readiness

`task production:plan -- --milestone PM-RENDER-RT-004 --roadmap WR-075`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- roadmap dependencies: none;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-075-renderer-ray-query-production-evidence/plan.md`.

This contract clears the design-first gap by naming the completed dependency
evidence, docs-only production-evidence scope, write scopes, validation, stop
conditions, and closeout quality before implementation.

After applying the intake proposal, WR-075 may be promoted only when:

- `WR-073` and `WR-074` remain completed with valid closeout evidence;
- `PM-RENDER-RT-004` is active and still selected by the stack coordinator;
- `task production:plan -- --milestone PM-RENDER-RT-004 --roadmap WR-075`
  reports a promotable or promotion-contract action rather than `design_first`;
- roadmap, production, docs, and planning validators pass.

The promotion preflight now reports:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-073:completed` and `WR-074:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-075 --state current_candidate --evidence "Accepted renderer hardware ray-query doctrine, completed WR-073 ray-query capability and acceleration-resource closeout, completed WR-074 hybrid runtime proof closeout, and active WR-075 ray-query production evidence contract."
```

WR-075 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-RT-004`.

After promotion, `task production:plan -- --milestone PM-RENDER-RT-004 --roadmap
WR-075` reports:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-073:completed` and `WR-074:completed`;
- next action: `write_implementation_contract`;
- implementation contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-075-renderer-ray-query-production-evidence/plan.md`.

WR-075 implementation may start only after this current-candidate evidence
remains true and the stack coordinator advances from contract writing to
contract execution.

## Governance Decisions

- DDD bounded context owner: `docs-site/src/content/docs/engine/reference/plugins/render`
  for renderer production evidence docs, consuming public `engine` examples and
  inspection DTO evidence.
- Renderer-owned vocabulary: optional ray-query capability matrix, acceleration
  resource lineage, unsupported diagnostics, visible non-RT fallback, hybrid
  proof output, timing label separation, benchmark/report evidence, and
  production evidence packet.
- Producer-owned vocabulary: scene truth, mesh/material truth, SDF product
  truth, camera/exposure truth, product freshness, fallback legality, quality
  authority, and hardware procurement/support policy.
- Translation boundary: WR-075 may document and cross-link renderer evidence. It
  must not add backend RT execution, hardware support claims not backed by
  evidence, or source-truth decisions in docs.
- ADR requirement: no ADR is required for docs/evidence hardening that consumes
  accepted doctrine and completed WR-073/WR-074 evidence. Stop for ADR if the
  implementation adds a durable RT ABI, changes fallback authority, changes
  runtime ownership, or makes RT hardware a baseline requirement.
- Team Topologies ownership: enabling documentation/evidence for the
  complicated-subsystem renderer platform.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-ray-query-production-evidence
docs-site/src/content/docs/reports/implementation-plans/wr-075-renderer-ray-query-production-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-075-renderer-ray-query-production-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `docs-site/src/content/docs/engine/reference/plugins/render/ray-query-production-evidence.md`:
  add the production evidence packet with capability matrix, fallback matrix,
  diagnostics, validation commands, and evidence links.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  link the production evidence packet from the public renderer API reference.

## Non-Goals

- Do not add hardware ray-query execution, acceleration-structure builders,
  shader tables, denoisers, or backend-specific RT code.
- Do not claim RT hardware is required for baseline rendering.
- Do not claim unverified vendor hardware support or visual parity beyond the
  documented evidence packet.
- Do not move fallback legality, scene truth, SDF truth, material truth,
  temporal truth, or product freshness into renderer docs.
- Do not add new public renderer APIs unless the production evidence cannot be
  documented honestly with existing WR-073 and WR-074 surfaces.

## Required Implementation Shape

WR-075 must provide production evidence that records:

1. Optional ray-query capability states: supported, unsupported, disabled, and
   pending/degraded timing or readback.
2. Derived BLAS/TLAS evidence and source-lineage expectations without exposing
   backend handles.
3. Visible non-RT fallback behavior and the command that proves the portable
   fallback path.
4. Hybrid proof timing labels for raster, SDF, temporal, ray-query, and
   fallback work.
5. Quality/parity notes that distinguish proven portable fallback evidence from
   future vendor hardware evidence.
6. Validation commands that keep the evidence reproducible.
7. Remaining gaps for final `PT-RENDER-PERFECTION` no-gap verification.

The closeout target is `runtime_proven` for optional RT production evidence
with mandatory fallback, not `perfectionist_verified`.

## Validation

Focused validation expected for implementation:

```text
cargo test -p engine render_ray_query
cargo test -p engine --example render_hybrid_ray_sdf_raster_runtime_proof
cargo run -p engine --example render_hybrid_ray_sdf_raster_runtime_proof
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

Stop before implementation if:

- `WR-075` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-RT-004 --roadmap WR-075`
  still reports `design_first`, a promotion blocker, or a metadata blocker;
- WR-073 or WR-074 closeout evidence is missing or invalid;
- implementation requires code outside the approved docs/evidence scope;
- implementation needs real RT hardware claims not backed by current evidence;
- implementation would make RT hardware mandatory or hide fallback behavior;
- required validation cannot run or fails;
- closeout evidence cannot honestly claim optional RT `runtime_proven` quality.

## Closeout Requirements

The closeout must include:

- exact changed docs and metadata paths;
- architecture evidence showing the docs consume renderer evidence without
  owning source truth or hardware policy;
- validation commands and results;
- roadmap and production metadata updates;
- completion quality `runtime_proven`;
- known gaps for final perfectionist verification.
