---
title: WR-073 Renderer Ray Query Capability And Acceleration Resources Closeout
description: Closeout evidence for renderer-owned optional ray-query capability diagnostics and derived acceleration-resource inspection.
status: completed
owner: engine
layer: engine-runtime / renderer ray-query capability
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-hardware-ray-query-and-hybrid-tracing-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-073-renderer-ray-query-capability-and-acceleration-resources/plan.md
---

# WR-073 Renderer Ray Query Capability And Acceleration Resources Closeout

## Result

`WR-073` is complete at `bounded_contract` quality. The renderer now exposes
typed inspection evidence for optional ray-query capability, unsupported
hardware states, visible non-RT fallback, and derived acceleration-resource
lineage without exposing mutable backend handles or moving source truth into
the renderer.

This closeout does not claim `runtime_proven` or `perfectionist_verified`.
Hybrid runtime proof remains WR-074 scope, ray-query production evidence
remains WR-075 scope, and final no-gap verification remains
`PT-RENDER-PERFECTION` scope.

## Changed Modules

- `engine/src/plugins/render/inspect/ray_query.rs`:
  added `inspect_render_ray_query_capability`,
  `RenderRayQueryCapabilityProfile`, `RenderRayQueryCapabilityState`,
  `RenderRayQueryAccelerationResourceEvidence`,
  `RenderRayQueryAccelerationResourceKind`,
  `RenderRayQueryAccelerationResourceStatus`,
  `RenderRayQueryAccelerationSourceLineage`,
  `RenderRayQueryAccelerationResourceCounts`, `RenderRayQueryInspection`,
  `RenderRayQueryInspectionRequest`, `RenderRayQueryDiagnostic`, and
  `RenderRayQueryDiagnosticSeverity`.
- `engine/src/plugins/render/inspect/mod.rs`:
  exports the ray-query capability and acceleration-resource inspection
  surface.
- `engine/tests/render_ray_query.rs`:
  guards supported capability readiness, portable unsupported fallback
  diagnostics, hidden-fallback failures, missing source lineage failures,
  backend-handle privacy, stale resource invalidation, and memory budget
  diagnostics.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents the public inspection DTOs and optional-capability ownership
  contract.

## Architecture Evidence

WR-073 followed the accepted renderer hardware ray-query doctrine:

- Source truth remains with scene, mesh, material, product, SDF, temporal,
  camera, exposure, and fallback-policy owners.
- The renderer owns backend capability labels, unsupported diagnostics, derived
  acceleration-resource lineage, build/update status, memory evidence, debug
  labels, and public inspection DTOs.
- Acceleration resources are derived execution evidence keyed by source
  lineage. They are not product, scene, mesh, material, or SDF authority.
- Public inspection rejects mutable backend-handle exposure.
- No ADR was required because WR-073 added renderer-owned DTOs and diagnostics
  without changing dependency direction, fallback authority, persisted ABI, or
  baseline hardware requirements.

## Evidence

The implementation provides explicit bounded evidence for:

- supported, unsupported, disabled, and pending capability states;
- typed required capability labels and unsupported reasons;
- visible non-RT fallback when hardware capability is unavailable;
- bottom-level and top-level derived acceleration-resource counts;
- source lineage, product generation, cache identity, memory bytes, and build
  version evidence;
- stale source invalidation diagnostics;
- fail-closed diagnostics for missing capability labels, missing unsupported
  reasons, hidden fallback, missing source lineage, missing memory/build
  evidence, stale resources without invalidation reasons, budget overflow, and
  public backend-handle exposure.

## Validation

Focused validation passed:

```text
cargo fmt
cargo test -p engine render_ray_query
cargo test -p engine render_runtime_inspect
cargo test -p engine render_resource_model
```

The `render_ray_query` filter ran the new WR-073 guard suite: 7 tests passed.
The runtime and resource-model filters confirm the new public inspection surface
does not regress existing renderer inspection and resource model contracts.

Final planning validation after roadmap and production metadata updates:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- WR-073 does not implement hardware ray-query execution, GPU acceleration
  structure builds, shader tables, denoisers, or hybrid runtime rendering.
- Hybrid raster/SDF/ray-query runtime proof remains WR-074 scope.
- Ray-query production examples, hardware matrix, benchmark/report artifacts,
  fallback evidence, and production diagnostics remain WR-075 scope.
- WR-073 does not claim `runtime_proven` or `perfectionist_verified`.

These gaps are expected sequencing boundaries for `PT-RENDER-RT`, not hidden
completion defects in the bounded WR-073 contract.
