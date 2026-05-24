---
title: PM-RENDER-TEMPORAL-001 Temporal Reconstruction Doctrine Closeout
description: Closeout evidence for accepting the portable renderer temporal reconstruction, dynamic resolution, and optional upscaling adapter doctrine.
status: completed
owner: engine
layer: engine-runtime / renderer temporal doctrine
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# PM-RENDER-TEMPORAL-001 Temporal Reconstruction Doctrine Closeout

## Result

`PM-RENDER-TEMPORAL-001` is complete at `bounded_contract` quality. The renderer
temporal reconstruction doctrine is accepted and records the long-term contract
for portable TAA/TAAU, history validity, jitter, dynamic internal resolution,
input availability, optional upscaling adapters, and explicit fallback
diagnostics.

No product code, renderer runtime code, examples, benchmarks, or shader assets
changed for this milestone. This closeout accepts doctrine and sequencing only;
it does not authorize `WR-070`, `WR-071`, or `WR-072` implementation without
their own roadmap gates, implementation contracts, validation, and closeouts.

## Accepted Doctrine

Accepted design:

```text
docs-site/src/content/docs/design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
```

The design records:

- renderer ownership of derived temporal execution state, history textures,
  history signatures, jitter phase, dynamic-resolution state, reconstruction
  mode, timing, capability diagnostics, and adapter invocation records;
- producer ownership of camera, scene, product, exposure, material reactivity,
  SDF, ray-query, freshness, authority, and fallback-legality truth;
- typed contracts for internal/output resolution, jitter sequence, history
  validity, motion-vector/depth/exposure/reactive input availability,
  reconstruction mode, adapter capability, timing, quality diagnostics, and
  artifact evidence;
- invariants that keep vendor adapters optional, require visible native fallback
  on unsupported capability, and prevent silent history reuse on signature
  mismatch;
- downstream sequence for `WR-070`, `WR-071`, and `WR-072`.

## Governance Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Accept renderer temporal reconstruction and dynamic resolution doctrine for PM-RENDER-TEMPORAL-001" --scope "docs-site/src/content/docs/design/active/renderer-temporal-reconstruction-and-dynamic-resolution-design.md docs-site/src/content/docs/workspace/production-tracks.yaml docs-site/src/content/docs/workspace/roadmap-deferred.yaml docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-temporal-inputs-history-and-dyn docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-upscaling-adapters-and-ray-reco docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-temporal-production-evidence"
```

Governance decision:

- Bounded context owner: `engine/src/plugins/render`.
- Team Topologies owner: complicated-subsystem renderer platform consuming
  stream-aligned camera, scene, product, material, SDF, ray-query, and exposure
  producers.
- ADR requirement: no ADR for doctrine acceptance because dependency direction
  preserves accepted Render Product Graph, scale, SDF, and GPU evidence
  boundaries. ADR is required before a later implementation persists a new
  cross-domain ABI, moves camera/product/exposure/fallback authority into the
  renderer, or makes vendor upscalers baseline requirements.
- Next action after closeout: use the stack coordinator and only apply/promote
  the first legal implementation WR when its intake, dependencies, write
  scopes, and validations are ready.

## Metadata Evidence

Updated source files:

- `docs-site/src/content/docs/design/active/README.md`: removed the renderer
  temporal design from active designs.
- `docs-site/src/content/docs/design/accepted/README.md`: added the renderer
  temporal design to accepted designs.
- `docs-site/src/content/docs/design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md`:
  accepted and expanded the doctrine.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  retargeted related-design evidence to the accepted temporal design.
- `docs-site/src/content/docs/design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md`
  and
  `docs-site/src/content/docs/design/active/renderer-production-audit-and-perfectionist-verification-design.md`:
  retargeted related-design evidence to the accepted temporal design.
- `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-TEMPORAL-001`:
  marked the doctrine milestone completed and added this closeout as evidence.
- Temporal roadmap intake proposals and deferred rows for `WR-070`, `WR-071`,
  and `WR-072` now point at the accepted design path and this closeout gate.
- Downstream temporal production milestones now require the accepted temporal
  design path.

## Validation

Design validation after metadata updates:

```text
task docs:validate
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Temporal inputs, history validity, and dynamic internal resolution remain
  blocked until `WR-070` is applied, promoted, implemented, validated, and
  closed.
- Upscaling adapters and ray reconstruction inputs remain blocked until
  `WR-071` is applied, promoted, implemented, validated, and closed.
- Temporal production evidence remains blocked until `WR-072` is applied,
  promoted, implemented, validated, and closed.
- This design-only milestone does not claim `runtime_proven` or
  `perfectionist_verified`.

These gaps are expected sequencing boundaries for `PT-RENDER-TEMPORAL`, not
hidden completion defects in `PM-RENDER-TEMPORAL-001`.
