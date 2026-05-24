---
title: Sparse SDF Terrain Runtime Integration
description: Active design for a shader-bound sparse SDF terrain runtime that consumes renderer-owned SDF GPU resources without making renderer code own SDF product truth.
status: active
owner: engine
layer: engine-runtime / renderer / sdf-products
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../accepted/render-production-readiness-and-inspection-design.md
  - ../accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
  - ../../reports/implementation-plans/wr-103-shader-bound-sparse-sdf-terrain-runtime-governance-and-track-activation/plan.md
---

# Sparse SDF Terrain Runtime Integration

## Decision

Create a renderer follow-up track for shader-bound sparse SDF terrain runtime
integration. The completed `PT-RENDER-SDF` track remains valid and closed:
it proved derived residency, raymarch acceleration inspection, runtime
evidence aggregation, and diagnostics. This design records the remaining gap:
there is not yet a production-oriented terrain render path where WGSL traversal
consumes the sparse SDF page table, brick atlas, distance mip, and candidate
list resources produced by renderer residency.

The source truth stays in `domain/world_sdf::SdfChunkPayload`, product
publication, and source generations. The renderer owns only derived GPU
resources, bind plans, pass execution, timing, and diagnostics.

## Current State

`engine/examples/procedural_sky_sdf_terrain` is a useful visual demo. It renders
terrain with analytic shader functions and fullscreen raymarching, but its WGSL
does not traverse renderer-owned sparse SDF residency resources. It remains an
analytic visual example and must not be used to satisfy the sparse runtime
proof.

The existing renderer SDF code exposes deterministic inspection evidence:

- `RenderSdfResidencyResource::derive_from_sources` derives residency DTOs
  from source payloads.
- `inspect_sdf_raymarch_acceleration` derives distance-mip, safe-step,
  candidate-list, and diagnostic evidence.
- `inspect_render_sdf_production_evidence` aggregates residency, raymarch,
  timing, benchmark, and visual evidence for closeout.

Those surfaces are necessary but not sufficient for the runtime gap. The
missing contract is the shader-bound data flow from source payloads into GPU
bindings consumed by a terrain raymarch pass.

## Runtime Contract

The intended data flow is:

```text
SdfChunkPayload
  -> RenderSdfResidencyResource::derive_from_sources
  -> inspect_sdf_raymarch_acceleration / acceleration resource
  -> RenderSdfTerrainRuntimeBindPlan
  -> shader-bound fullscreen terrain raymarch pass
  -> RenderSdfTerrainRuntimeFrameInspection
```

Follow-on implementation should introduce renderer-facing types with these
responsibilities:

- `RenderSdfTerrainRuntimeConfig`: per-runtime limits, budgets, scale bands,
  camera-relative framing policy, and diagnostic toggles.
- `RenderSdfTerrainRuntimeBindPlan`: renderer-owned binding plan for page
  table, brick atlas, distance mip, candidate lists, generations, limits, and
  camera-relative origin data.
- `RenderSdfTerrainRuntimeFrameInspection`: per-frame evidence that the shader
  path used sparse resources, not analytic-only terrain math.
- `RenderSdfTerrainRuntimeDiagnostic`: fail-closed diagnostics for missing
  residency, stale generations, unsupported limits, candidate explosion,
  unsafe overstep, over-budget residency, and empty candidate coverage.

The first proof should use deterministic synthetic `SdfChunkPayload` terrain
products. That keeps the renderer integration unblocked while the final
open-world terrain product pipeline remains `PM-SDF-OW-002` scope.

## Rendering Rules

- Use one fullscreen terrain raymarch pass per prepared view.
- Never multiply terrain raymarch work by chunk, entity, source, or payload
  count.
- WGSL must consume sparse SDF resource bindings for traversal. Analytic-only
  terrain functions, hardcoded heightfields, or CPU-only inspection DTOs cannot
  satisfy the runtime proof.
- Page table, brick atlas, distance mip, and candidate-list bindings must be
  part of the shader-visible contract.
- Camera-relative world framing is required so endless-world coordinates do
  not depend on fragile absolute `f32` positions.

## Track Split

`PM-RENDER-SDF-RUNTIME-001` is governance only. It creates the track, this
design, `WR-103`, and the implementation contract.

`PM-RENDER-SDF-RUNTIME-002` owns GPU ABI and runtime bind-plan implementation.
It must keep backend handles inside renderer boundaries and keep product truth
outside renderer state.

`PM-RENDER-SDF-RUNTIME-003` owns the dedicated sparse terrain runtime example
and WGSL traversal proof.

`PM-RENDER-SDF-RUNTIME-004` owns evidence commands, focused tests, benchmarks,
docs, and closeout. It is the only milestone in this track that may claim
`runtime_proven`.

## Non-Goals

- Do not reopen `PT-RENDER-SDF` or rewrite its closeout claims.
- Do not make `procedural_sky_sdf_terrain` the production sparse runtime proof.
- Do not implement renderer modules, shaders, assets, or examples in `WR-103`.
- Do not make renderer code authoritative for SDF payload truth, collision
  truth, query policy, product fallback legality, or open-world generation.
- Do not claim `perfectionist_verified`; that remains `PT-RENDER-PERFECTION`
  scope.

## Evidence Expectations

Follow-on implementation evidence must prove:

- deterministic GPU-layout derivation from source generations;
- fail-closed missing and stale residency diagnostics;
- bounded candidate-list behavior;
- conservative distance-mip stepping with unsafe-overstep diagnostics;
- camera-relative far-origin behavior;
- shader-bound sparse resource consumption rather than analytic-only terrain;
- runtime budget and residency pressure reporting;
- docs that distinguish renderer runtime proof from real open-world terrain
  product integration.
