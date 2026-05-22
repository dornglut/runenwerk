---
title: WR-082 Renderer GPU Evidence And Procedural Visuals Doctrine Acceptance Plan
description: Design-first planning contract for accepting the renderer GPU evidence and procedural visuals doctrine before implementation rows start.
status: active
owner: engine
layer: engine-runtime / render doctrine
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-082 Renderer GPU Evidence And Procedural Visuals Doctrine Acceptance Plan

## Goal

Clear the design-first gate for `PM-RENDER-GPU-001` by turning the active
renderer GPU evidence and procedural visuals doctrine into an accepted
implementation contract. This action is design and roadmap-readiness work only.
It does not authorize renderer code, tests, examples, benchmarks, or production
milestone completion.

The accepted doctrine must make the later `PT-RENDER-GPU` rows boring to
execute: GPU timing, pass-shape guards, procedural instance APIs, canonical
boids proof, and production evidence each remain separate bounded WR slices with
their own validation and closeout.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-GPU-001`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml::WR-082`.
- Active design:
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`.
- Accepted renderer boundary:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- Accepted readiness boundary:
  `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`.
- Roadmap intake evidence:
  `docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-gpu-evidence-and-procedural-vis`.

`task production:plan -- --milestone PM-RENDER-GPU-001 --roadmap WR-082`
reported:

- milestone state: `active`;
- WR state: `blocked_deferred`;
- WR blocker: `B5`;
- dependencies: none;
- next action: `design_first`.

## Readiness

`WR-082` is not ready for implementation or completion while it remains a
policy-deferred intake row. The next bounded action after this contract is to
use the accepted doctrine as roadmap evidence only if validation passes and the
remaining roadmap metadata can be changed without bypassing deferred-source
governance.

The `B5` blocker can be cleared only after there is accepted design evidence
that names:

- renderer-owned GPU evidence versus product-owned truth and policy;
- the exact implementation row sequence from `WR-056` through `WR-060`;
- non-goals that prevent SDF residency, scale, temporal, ray-query,
  mesh/material, product-visual, and audit work from leaking into this track;
- validation and closeout expectations for the later `runtime_proven` claim;
- architecture governance requirements before code changes.

## Ownership And Boundaries

DDD bounded context owner:

- `engine/src/plugins/render` owns renderer execution evidence, render-flow
  validation, procedural instance execution APIs, renderer examples,
  backend-neutral DTOs, and runtime inspection projections.

Producer and product-domain owners:

- product producers own selected visual products, source truth, freshness,
  authority, fallback legality, rebuild policy, residency intent, gameplay VFX
  semantics, material/model truth, and emitter policy;
- renderer inputs are prepared or declared products; renderer output is derived
  execution state, diagnostics, timing, and presentation/capture evidence.

Team Topologies ownership:

- the renderer is a complicated subsystem that enables stream-aligned product,
  editor, world, and content teams through typed contracts and diagnostics.

ADR requirement:

- no ADR is required for accepting this doctrine while it preserves the accepted
  renderer/product boundary;
- create an ADR or stop if the doctrine changes dependency direction, makes the
  renderer authoritative for product truth or residency, defines a persisted
  cross-domain ABI, or makes GPU timing/procedural APIs mandatory outside the
  renderer ownership boundary.

## Design Acceptance Decisions

The doctrine acceptance update must resolve the current open questions as
explicit decisions:

1. Fullscreen plus large instance count is rejected by default. Any opt-in must
   be an explicit advanced render-flow policy with typed intent, pass identity,
   bounded instance evidence, and inspection output.
2. Backends without timestamp-query support can still satisfy the first
   implementation slice only when they expose typed unsupported diagnostics,
   CPU encode/submit timing, pass-shape evidence, and runtime inspection that
   proves timing absence is capability-driven rather than silently missing.
3. The first public SDF impostor API is local 2D sprite/impostor oriented. 3D
   SDF hooks remain out of scope until the later sparse SDF and raymarch tracks
   provide accepted residency and acceleration contracts.
4. Boids is the mandatory canonical proof for this track. Additional examples
   may support docs or coverage, but they are not required to unblock the
   doctrine milestone.
5. Initial boids production evidence should measure stable bounded work rather
   than inventing premature universal frame thresholds. The production evidence
   row must record scene size, backend capability, CPU timing, GPU timing or
   unsupported diagnostics, pass shape, instance count, and benchmark command.

## Implementation Scope For Later Rows

This contract permits only design acceptance and roadmap-readiness edits. Later
implementation rows may use these write scopes only after their own gates pass:

```text
engine/src/plugins/render
engine/examples
engine/tests
engine/benches
assets/shaders/boids_compose.wgsl
assets/shaders/boids_compute.wgsl
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
docs-site/src/content/docs/reports/implementation-plans/wr-082-renderer-gpu-evidence-and-procedural-visuals-doctrine-acceptance/plan.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-gpu-evidence-and-procedural-vis
docs-site/src/content/docs/reports/closeouts/pm-render-gpu-001-gpu-evidence-and-procedural-visuals-doctrine/closeout.md
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/roadmap-deferred-register.md
docs-site/src/content/docs/workspace/production-tracks.yaml
docs-site/src/content/docs/workspace/production-track-index.md
docs-site/src/content/docs/workspace/production-milestone-register.md
docs-site/src/content/docs/workspace/diagrams/production-track-roadmap.puml
```

Expected later implementation modules must be chosen from nearby renderer
subsystems and named in each row contract before code changes. Do not create
catch-all helper files.

## Non-Goals

Do not implement or accept as part of this doctrine milestone:

- GPU timestamp query code, render-flow guards, procedural APIs, boids rewrite,
  docs hardening, benchmarks, or examples;
- SDF brick/page-table residency, clipmaps, raymarch acceleration, temporal
  reconstruction, hardware ray queries, mesh/material truth, lighting pipeline
  policy, product VFX emitters, or final renderer audit;
- renderer-owned product selection, source truth, freshness, authority,
  fallback legality, rebuild policy, residency policy, gameplay particle
  semantics, material truth, model truth, or editor authoring truth;
- a `runtime_proven` or `perfectionist_verified` completion claim for
  `PM-RENDER-GPU-001`.

## Acceptance Criteria

- The doctrine design has accepted frontmatter status and lives in the accepted
  design area, or the workflow has an explicit accepted-status rule for the
  current path.
- All five open decisions are resolved in the design text.
- The design preserves the accepted product/render boundary and readiness
  inspection boundary.
- `WR-082` remains design-first until roadmap metadata can honestly reference
  accepted doctrine evidence.
- Later implementation rows remain blocked until their own gates, contracts,
  validation, and closeout evidence exist.

## Validation

Focused validation for the design-first action:

```text
task docs:validate
task production:validate
task roadmap:validate
task planning:validate
```

After any roadmap or production metadata changes, also run:

```text
task roadmap:render
task roadmap:check
task production:render
task production:check
task ai:goal -- --track PT-RENDER-PERFECTION --stack
```

## Stop Conditions

- Stop if accepting the doctrine would require renderer ownership of product
  truth, residency policy, gameplay/VFX semantics, material truth, model truth,
  or source authority.
- Stop if an ADR becomes required and is not written and accepted.
- Stop if docs, roadmap, production, or planning validation fails.
- Stop if generated roadmap or production outputs are stale after metadata
  changes.
- Stop if implementation code would be required to justify this design-only
  milestone.

## Closeout Requirements

Close `PM-RENDER-GPU-001` only after a dedicated closeout document records:

- accepted doctrine path and status;
- validation commands and results;
- evidence that `WR-082` was handled as design-first, not product code;
- remaining known quality gaps for later implementation rows;
- production and roadmap metadata updates, if any.

The closeout path should be:

```text
docs-site/src/content/docs/reports/closeouts/pm-render-gpu-001-gpu-evidence-and-procedural-visuals-doctrine/closeout.md
```

## Perfectionist Closeout Audit

Expected completion quality for `PM-RENDER-GPU-001`: `bounded_contract`.

Known quality gaps that must remain visible until later rows close:

- GPU pass timing is not implemented until `WR-056`.
- Render-flow pass-shape guards are not implemented until `WR-057`.
- Hybrid procedural instance APIs are not implemented until `WR-058`.
- Canonical boids runtime proof is not implemented until `WR-059`.
- Runtime production evidence is not complete until `WR-060`.

`runtime_proven` is available only for the completed `PT-RENDER-GPU` track after
runtime/GPU or explicit unsupported-capability evidence exists. This doctrine
milestone must not claim `perfectionist_verified`.
