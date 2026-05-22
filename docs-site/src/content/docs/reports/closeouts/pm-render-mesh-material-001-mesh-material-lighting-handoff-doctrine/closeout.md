---
title: PM-RENDER-MESH-MATERIAL-001 Mesh Material Lighting Handoff Doctrine Closeout
description: Closeout evidence for accepting the renderer mesh, material, lighting, shader, and asset handoff doctrine.
status: completed
owner: engine
layer: engine-runtime / renderer mesh material doctrine
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# PM-RENDER-MESH-MATERIAL-001 Mesh Material Lighting Handoff Doctrine Closeout

## Result

`PM-RENDER-MESH-MATERIAL-001` is complete at `bounded_contract` quality. The
renderer mesh/material/lighting handoff doctrine is accepted and records the
long-term contract for prepared mesh/material/shader inputs, product-surface
previews, renderer-owned pipeline artifacts, pipeline cache diagnostics, and
last-good shader fallback boundaries.

No product code, renderer runtime code, examples, benchmarks, or shader assets
changed for this milestone. This closeout accepts doctrine and sequencing only;
it does not authorize `WR-067`, `WR-068`, or `WR-069` implementation without
their own roadmap gates, implementation contracts, validation, and closeouts.

## Accepted Doctrine

Accepted design:

```text
docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md
```

The design records:

- renderer ownership of derived mesh records, shader artifacts, bind group
  layouts, pipeline specialization keys, pipeline cache entries, lighting
  buffers, last-good shader references, asset-cook outputs, and diagnostics;
- producer ownership of material source documents, scene material assignments,
  asset catalog truth, model/mesh/rig/animation semantics, product lineage,
  freshness, authority class, rebuild policy, residency intent, and fallback
  legality;
- explicit translation points for material graph lowering, prepared mesh/model
  products, mesh/material previews, asset cooking, shader specialization, and
  pipeline cache decisions;
- invariants that prevent renderer-owned material/asset/model truth, raw
  artifact-id assignment, silent fallback, and preview paths that bypass product
  surfaces;
- downstream sequence for `WR-067`, `WR-068`, and `WR-069`.

## Governance Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "Accept mesh material lighting shader and asset handoff doctrine for PM-RENDER-MESH-MATERIAL-001" --scope "docs-site/src/content/docs/design/active/renderer-mesh-material-lighting-and-asset-handoff-design.md docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md docs-site/src/content/docs/workspace/production-tracks.yaml docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-mesh-material-shader-asset-hand docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-lighting-pipeline-cache-and-las docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-mesh-material-production-eviden"
```

Governance decision:

- Bounded context owner: `engine/src/plugins/render`.
- Team Topologies owner: complicated-subsystem renderer platform consuming
  stream-aligned material, asset, model, scene, lighting-source, and product
  producers.
- ADR requirement: no ADR for doctrine acceptance because dependency direction
  preserves accepted Render Product Graph, GPU evidence, Material Lab, and
  product-surface boundaries. ADR is required before a later implementation
  persists a new cross-domain ABI, moves material/asset/model truth into the
  renderer, changes fallback authority, or changes Product Graph/Product Jobs
  ownership.
- Next action after closeout: use the stack coordinator and only apply/promote
  the first legal implementation WR when its intake, dependencies, write scopes,
  and validations are ready.

## Metadata Evidence

Updated source files:

- `docs-site/src/content/docs/design/active/README.md`: removed the renderer
  mesh/material handoff design from active designs.
- `docs-site/src/content/docs/design/accepted/README.md`: added the renderer
  mesh/material handoff design to accepted designs.
- `docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md`:
  accepted and expanded the doctrine.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  retargeted related-design evidence to the accepted handoff design.
- `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-MESH-MATERIAL-001`:
  marked the doctrine milestone completed and added this closeout as evidence.
- Renderer mesh/material roadmap intake proposals for `WR-067`, `WR-068`, and
  `WR-069` now point at the accepted design path.
- Downstream renderer mesh/material production milestones now require the
  accepted handoff design path.

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

- Prepared mesh/material/shader handoff remains blocked until `WR-067` is
  applied, promoted, implemented, validated, and closed.
- Lighting inputs, pipeline specialization/cache diagnostics, and last-good
  shader fallback remain blocked until `WR-068` is applied, promoted,
  implemented, validated, and closed.
- Mesh/material/lighting production evidence remains blocked until `WR-069` is
  applied, promoted, implemented, validated, and closed.
- This design-only milestone does not claim `runtime_proven` or
  `perfectionist_verified`.

These gaps are expected sequencing boundaries for `PT-RENDER-MESH-MATERIAL`,
not hidden completion defects in `PM-RENDER-MESH-MATERIAL-001`.
