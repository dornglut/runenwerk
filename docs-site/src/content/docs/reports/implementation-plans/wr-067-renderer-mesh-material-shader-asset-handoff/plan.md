---
title: WR-067 Renderer Mesh Material Shader Asset Handoff Implementation Contract
description: Design-first contract for prepared mesh, material, shader, and asset handoff through renderer-owned execution contracts.
status: active
owner: engine
layer: engine-runtime / renderer mesh material handoff
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/active/material-lab-and-material-preview-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-067 Renderer Mesh Material Shader Asset Handoff Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-MESH-MATERIAL-002` and
`WR-067`. This row must harden the prepared mesh/material/shader handoff path
so renderer execution can consume source-backed material products, model/mesh
material selections, shader artifacts, and asset-cook metadata without moving
asset, material, model, scene, or product truth into the renderer.

This is a design-first contract. It clears the deferred intake questions and
prepares WR-067 for roadmap application and promotion. It does not authorize
product code changes until the stack coordinator selects WR-067 for
implementation after roadmap gates are satisfied.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md`:
  accepted doctrine for renderer-owned prepared inputs, shader artifacts,
  pipeline execution state, product-surface previews, diagnostics, and
  non-ownership of material/asset/model truth.
- `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`:
  product selection, freshness, authority, fallback legality, rebuild policy,
  and residency intent stay outside renderer execution.
- `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`:
  renderer evidence must remain explicit, inspectable, pass-shape-aware, and
  honest about unsupported runtime capabilities.
- `docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md`:
  Material Lab source documents, ratification, preview products, and generated
  shader handoff remain material owner truth and product projections.
- `engine/src/plugins/render/frame/contributions.rs`:
  existing renderer contract owner for `PreparedMaterialFeatureContribution`,
  `PreparedSceneMaterialBundle`,
  `PreparedModelMeshMaterialSourceIdentity`,
  `PreparedModelMeshMaterialRegionIdentity`, and
  `PreparedModelMeshMaterialSelection`.
- `engine/src/plugins/render/renderer/render_flow/provenance.rs` and
  `engine/src/plugins/render/inspect/pass_provenance.rs`:
  existing material binding evidence path for compiled passes.
- `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs`:
  current app-side translation from material preview and scene material slots
  into renderer prepared material resources.
- `domain/editor/editor_scene/src/model/material.rs`:
  source owner for scene model/mesh material assignment validation.

## Readiness

`task production:plan -- --milestone PM-RENDER-MESH-MATERIAL-002 --roadmap WR-067`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-067-renderer-mesh-material-shader-asset-handoff/plan.md`.

Architecture governance for `PM-RENDER-MESH-MATERIAL-001` accepted the doctrine
with these decisions:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer
  execution artifacts, prepared material contracts, shader artifact
  consumption, GPU resource allocation, and inspection DTOs.
- Source owners: `domain/material_graph`, `domain/editor/editor_scene`, asset
  workflow/catalog owners, and model/mesh source domains own material, scene
  assignment, asset, and model truth.
- Translation boundary: app/domain producers emit prepared products and
  renderer contributions. Renderer code validates execution contracts and
  portable limits but does not edit or ratify source semantics.
- ADR requirement: no ADR is required if WR-067 hardens existing prepared
  contracts and diagnostics. Stop for ADR if implementation persists a new
  cross-domain ABI, moves material/asset/model truth into the renderer, or
  changes fallback authority.
- Team Topologies ownership: complicated-subsystem renderer platform consuming
  stream-aligned product producers.

## Promotion Readiness

WR-067 can become promotable only after the intake proposal records:

- dependency on completed `PM-RENDER-MESH-MATERIAL-001` doctrine evidence;
- dependency on completed renderer product graph and GPU evidence milestones
  already named by the production track;
- accepted handoff design gate;
- this active implementation contract as a design gate;
- implementation write scopes that include renderer prepared contracts,
  renderer inspection/provenance, focused renderer tests, relevant Material Lab
  handoff tests only when the app-side adapter must be adjusted, docs, intake,
  roadmap metadata, and production metadata;
- focused validation commands before any closeout claim.

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render/frame
engine/src/plugins/render/inspect
engine/src/plugins/render/renderer/render_flow
engine/src/plugins/render/renderer
engine/src/plugins/render/runtime
engine/tests
apps/runenwerk_editor/src/material_lab
apps/runenwerk_editor/src/runtime
domain/editor/editor_scene
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-mesh-material-shader-asset-hand
docs-site/src/content/docs/reports/implementation-plans/wr-067-renderer-mesh-material-shader-asset-handoff/plan.md
docs-site/src/content/docs/reports/closeouts/wr-067-renderer-mesh-material-shader-asset-handoff/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/frame/contributions.rs`:
  harden prepared material, scene bundle, source identity, region identity,
  model/mesh material selection, portable limit, and diagnostic contracts.
- `engine/src/plugins/render/inspect/pass_provenance.rs`:
  expose complete material handoff evidence needed by closeout and docs.
- `engine/src/plugins/render/renderer/render_flow/provenance.rs`:
  prove compiled material-consuming passes see the prepared material bundle and
  source-backed model/mesh material selections.
- `engine/src/plugins/render/renderer/prepare.rs` and
  `engine/src/plugins/render/renderer/execute_passes.rs`:
  consume prepared material GPU resources only through renderer execution
  contracts. Extend only when inspection proves a missing consumed path.
- `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs`:
  adjust only if the renderer contract needs a producer-side field that already
  belongs to Material Lab or scene material product projection.
- `domain/editor/editor_scene/src/model/material.rs`:
  adjust only for source-owned validation needed before renderer handoff; do
  not move renderer concepts into scene material source truth.
- `engine/tests/render_runtime_inspect.rs`,
  `engine/tests/render_flow_fragments.rs`, or a new focused
  `engine/tests/render_mesh_material_handoff.rs`:
  assert handoff and inspection invariants.

## Required Contracts

WR-067 must provide or verify explicit typed evidence for:

- material source lineage from source document or generated material product to
  `PreparedMaterialFeatureContribution`;
- scene material table identity, shader artifact identity, shader cache key,
  shader path, shader identity, material table identity, and resource layout
  identity through `PreparedSceneMaterialBundle`;
- source-backed model/mesh material region identity that rejects transient
  renderer keys such as renderable index, draw order, mesh table index, or
  residency slot;
- requested material slot, resolved material slot, material table index, and
  default fallback state for every model/mesh material selection;
- pass-level material binding evidence showing whether a pass consumes material
  resources and which source-backed selections are available to that pass;
- portable resource and material-table limit diagnostics that fail closed.

## Non Goals

- No implementation of lighting pipeline cache policy or last-good shader
  fallback beyond preserving existing `FeatureFallbackPolicy` and provenance
  data needed by WR-068.
- No production benchmark, runtime evidence report, or runtime-proven closeout;
  those belong to WR-069.
- No new material graph language semantics.
- No asset catalog, package trust, or import-policy redesign.
- No direct ECS live extraction path during renderer submission.
- No broad public API reshuffle unless required to make existing prepared
  contracts discoverable and hard to misuse.

## Implementation Steps

1. Inspect existing prepared material, scene bundle, model/mesh selection,
   provenance, runtime prepare, and Material Lab handoff paths before editing.
2. Identify the smallest missing contract fields or diagnostics needed to prove
   source-to-renderer handoff. Prefer extending existing DTOs over adding a new
   parallel material handoff path.
3. Harden validation in `engine/src/plugins/render/frame/contributions.rs` for
   source identity, region identity, table indices, fallback flags, shader
   artifact identity, resource layout identity, and portable limits.
4. Extend pass provenance inspection only where compiled material-consuming
   passes cannot already report prepared material availability, scene bundle
   identity, and model/mesh selection evidence.
5. Add focused tests that fail if renderer prepared data can be constructed from
   transient renderer identity, missing source lineage, missing shader identity,
   duplicate material regions, table-index overflow, or unconsumed material
   passes that claim readiness.
6. Update renderer public API/reference docs to describe the preferred prepared
   material handoff path and the source-truth boundary.
7. Add closeout evidence and roadmap/production metadata only after tests and
   docs validation pass.

## Acceptance Criteria

- Prepared mesh/material/shader handoff can be inspected end to end from
  source-backed producer identity to renderer pass material binding evidence.
- Material and model/mesh prepared contracts reject transient renderer identity,
  empty source identity, duplicate source regions, zero material slots, and
  portable-limit overflow.
- Renderer code only consumes prepared material products and derived GPU
  resources; it does not assign scene materials, ratify material graphs, edit
  assets, or decide product fallback legality.
- Tests cover both accepted handoff and fail-closed diagnostics.
- Docs explain what users should import or inspect for the common handoff path.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_mesh_material
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow_fragments
cargo test -p runenwerk_editor material_lab
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If exact test filters differ after inspection, the closeout must name the
actual focused tests and explain why they cover the same contracts.

## Stop Conditions

Stop before implementation if:

- WR-067 remains `blocked_deferred` or lacks accepted design, dependency,
  write-scope, validation, or contract gates;
- implementation would require renderer ownership of material graph truth,
  asset catalog truth, model/mesh source truth, scene material assignments, or
  product fallback legality;
- a new persisted cross-domain ABI is required without ADR review;
- the only evidence available is descriptor-only, status-panel-only,
  prepared-data-only, fallback-only, or unconsumed by compiled render passes;
- source files drift enough that this contract no longer describes the owning
  modules.

## Closeout Requirements

The closeout must include:

- exact changed files and owning modules;
- governance evidence and ADR decision;
- tests and command output summaries;
- docs, roadmap, production, and planning validation results;
- evidence that renderer pass material binding inspection consumes source-backed
  prepared material data;
- explicit known quality gaps for WR-068 and WR-069.

Roadmap and production metadata must move WR-067 through only the legal states:
apply intake, promote to `current_candidate`, implement, validate, archive with
completion evidence, and mark `PM-RENDER-MESH-MATERIAL-002` completed only when
its evidence gate points to the WR-067 closeout.

## Perfectionist Closeout Audit

Expected completion quality for WR-067: `bounded_contract`.

WR-067 can prove the prepared handoff contract, but it must not claim
`runtime_proven` unless it also supplies runtime visible mesh/material pixels,
shader failure behavior, pipeline cache/fallback evidence, and production
reports. Those are downstream WR-068 and WR-069 responsibilities.

Known quality gaps that must remain visible at WR-067 closeout:

- lighting pipeline cache and last-good fallback behavior remain WR-068 scope;
- production evidence, examples, benchmarks, runtime proof, and docs closeout
  remain WR-069 scope;
- final `perfectionist_verified` remains blocked until
  `PT-RENDER-PERFECTION` audits the completed renderer stack.
