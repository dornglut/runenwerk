---
title: WR-103 Shader-Bound Sparse SDF Terrain Runtime Governance And Track Activation
description: Docs-only implementation contract for activating the follow-on shader-bound sparse SDF terrain runtime production track.
status: active
owner: engine
layer: engine-runtime / renderer sdf runtime
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/sparse-sdf-terrain-runtime-integration-design.md
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-103 Shader-Bound Sparse SDF Terrain Runtime Governance And Track Activation

## Goal

Execute `PM-RENDER-SDF-RUNTIME-001` / `WR-103` as a docs-only governance row.
This row activates `PT-RENDER-SDF-RUNTIME`, records the shader-bound sparse SDF
terrain runtime gap, creates the active integration design, and splits future
implementation into durable milestones.

No Rust, WGSL, asset, or example code is in scope for this row.

## Architecture Governance

Kickoff command:

```text
task ai:architecture-governance -- --task "WR-103 shader-bound sparse SDF terrain runtime governance and track activation" --scope "docs-site/src/content/docs/workspace; docs-site/src/content/docs/design/active; docs-site/src/content/docs/reports/implementation-plans"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer
  execution, derived GPU residency, shader binding plans, diagnostics, timing,
  examples, and evidence.
- Source truth owner: `domain/world_sdf::SdfChunkPayload`, SDF product
  publication, and upstream product generators own terrain payload truth,
  generations, query policy, fallback legality, and open-world product
  integration.
- Translation boundary: renderer code may derive GPU resources from source
  generations and expose read-only diagnostics, but must not promote derived
  caches into product truth.
- ADR decision: no ADR for docs-only governance. Require an ADR or accepted
  design update before follow-on work changes durable GPU ABI ownership,
  dependency direction, SDF source-truth authority, fallback legality, or
  cross-domain product contracts.
- Ownership mode: complicated-subsystem renderer platform with stream-aligned
  SDF and world-product producer collaboration.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md`:
  accepted doctrine for sparse pages, bricks, clipmaps, distance mips,
  candidate lists, conservative stepping, and fullscreen-per-view raymarching.
- `docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md`:
  accepted boundary between product-owned SDF truth and renderer-owned derived
  GPU residency.
- `docs-site/src/content/docs/reports/closeouts/wr-066-sdf-world-runtime-evidence/closeout.md`:
  completed runtime evidence aggregation that this row consumes without
  reopening `PT-RENDER-SDF`.
- `engine/examples/procedural_sky_sdf_terrain`:
  current analytic visual terrain example. It remains useful but cannot satisfy
  shader-bound sparse SDF runtime proof.

## Required Changes

- Add `PT-RENDER-SDF-RUNTIME` after completed `PT-RENDER-SDF` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Add milestones `PM-RENDER-SDF-RUNTIME-001` through
  `PM-RENDER-SDF-RUNTIME-004`, with only the first milestone active and the
  implementation/evidence milestones designing.
- Add ready-next roadmap row `WR-103` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- Add active design
  `docs-site/src/content/docs/design/active/sparse-sdf-terrain-runtime-integration-design.md`
  and link it from the active design index.
- Render and validate production and roadmap generated docs after the source
  changes.

## Non-Goals

- Do not edit `engine/src`, `engine/examples`, `assets/shaders`, tests, or
  benchmark code in this row.
- Do not create `RenderSdfTerrainRuntime*` Rust types in this row.
- Do not create a sparse SDF runtime WGSL shader in this row.
- Do not apply or reuse `WR-102`; it is already reserved by renderer
  procedural population behavior-authoring intake.
- Do not close `PM-SDF-OW-002`; real open-world terrain product integration
  remains downstream.
- Do not claim `runtime_proven` or `perfectionist_verified` from governance
  alone.

## Acceptance Criteria

- `PT-RENDER-SDF-RUNTIME`, `PM-RENDER-SDF-RUNTIME-001`, and `WR-103` exist and
  validate.
- `PT-RENDER-SDF` remains completed and unchanged in meaning.
- The active design records the durable data flow:
  `SdfChunkPayload -> RenderSdfResidencyResource::derive_from_sources -> inspect_sdf_raymarch_acceleration -> RenderSdfTerrainRuntimeBindPlan -> shader-bound render flow`.
- The design states that `procedural_sky_sdf_terrain` is analytic-only visual
  demo evidence, not production sparse runtime proof.
- Future implementation milestones require WGSL consumption of page table,
  brick atlas, distance mip, and candidate-list resources.
- Final diff contains docs/governance files only.

## Validation

Run after source edits:

```text
task production:plan -- --milestone "PM-RENDER-SDF-RUNTIME-001" --roadmap "WR-103"
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task planning:validate
task puml:validate
task docs:validate
```

Final hygiene:

```text
git diff --name-only -- engine
git diff --name-only
```
