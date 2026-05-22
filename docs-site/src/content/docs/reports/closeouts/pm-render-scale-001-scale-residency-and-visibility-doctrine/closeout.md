---
title: PM-RENDER-SCALE-001 Scale Residency And Visibility Doctrine Closeout
description: Closeout evidence for accepting the finite-working-set renderer scale doctrine.
status: completed
owner: engine
layer: engine-runtime / renderer scale doctrine
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-RENDER-SCALE-001 Scale Residency And Visibility Doctrine Closeout

## Result

`PM-RENDER-SCALE-001` is complete at `bounded_contract` quality. The renderer
scale doctrine is accepted and records the long-term contract for finite
renderer working sets: addressable product space is not submitted directly;
renderer execution must distinguish selected, resident, visible, submitted, and
measured work.

No product code or renderer runtime code changed for this milestone. The
accepted design is doctrine and sequence evidence for later bounded WR rows; it
does not authorize `WR-061`, `WR-062`, or `WR-063` implementation by itself.

## Accepted Doctrine

Accepted design:

```text
docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
```

The design records:

- renderer ownership of derived execution records, GPU buffers, residency
  ranges, visibility/LOD/indirect buffers, timing, budgets, and inspection DTOs;
- product/domain ownership of source truth, selection, freshness, authority,
  fallback legality, rebuild policy, residency intent, and semantic LOD policy;
- finite evidence vocabulary for addressable records, selected products,
  resident records, visible candidates, submitted work, scale bands, and
  degraded modes;
- invariants that prevent renderer-owned product truth and silent fallback;
- implementation sequence for `WR-061`, `WR-062`, and `WR-063`;
- required fitness functions before any runtime scale claim.

## Governance Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Accept renderer scale residency and GPU-driven visibility doctrine for PM-RENDER-SCALE-001" --scope "docs-site/src/content/docs/design/active/renderer-scale-residency-and-gpu-driven-visibility-design.md docs-site/src/content/docs/workspace/production-tracks.yaml docs-site/src/content/docs/workspace/roadmap-deferred.yaml docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-working-set-registry-and- docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-gpu-driven-culling-lod-an docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-evidence-and-production-r"
```

Governance decision:

- Bounded context owner: `engine/src/plugins/render`.
- Team Topologies owner: complicated-subsystem renderer team with
  stream-aligned product/editor producers.
- ADR requirement: no ADR for doctrine acceptance because dependency direction
  follows accepted Render Product Graph and GPU Evidence boundaries. ADR is
  required before a later implementation moves residency/fallback authority into
  the renderer, persists a new cross-domain ABI, or changes product ownership.
- Next action after closeout: use the stack coordinator and then apply/promote
  the first legal implementation WR only when its intake and gates are ready.

## Metadata Evidence

Updated source files:

- `docs-site/src/content/docs/design/active/README.md`: removed the scale design
  from active designs.
- `docs-site/src/content/docs/design/accepted/README.md`: added the scale design
  to accepted designs.
- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  accepted and expanded the doctrine.
- `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-SCALE-001`:
  marked the doctrine milestone completed and added this closeout as evidence.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-working-set-registry-and-/proposal.yaml`:
  retargeted intake evidence to the accepted design path.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-gpu-driven-culling-lod-an/proposal.yaml`:
  retargeted intake evidence to the accepted design path.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-evidence-and-production-r/proposal.yaml`:
  retargeted intake evidence to the accepted design path.

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

- Working-set registry and residency budget implementation remains blocked until
  `WR-061` is applied, promoted, implemented, validated, and closed.
- GPU-driven culling, LOD, visible-list compaction, and indirect submission
  remain blocked until `WR-062` is applied, promoted, implemented, validated,
  and closed.
- Runtime scale evidence, hardware profiles, benchmarks, and production docs
  remain blocked until `WR-063` is applied, promoted, implemented, validated,
  and closed.
- This design-only milestone does not claim `runtime_proven` or
  `perfectionist_verified`.

These gaps are expected sequencing boundaries for `PT-RENDER-SCALE`, not hidden
completion defects in `PM-RENDER-SCALE-001`.
