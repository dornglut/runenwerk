---
title: WR-021 Material Lab And Material Preview Products Contract
description: Promotion and implementation-readiness contract for source-backed Material Lab and material preview product handoff.
status: active
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
  - ../../../design/implemented/render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-index.md
  - ../../../engine/roadmaps/fully-featured-renderer-roadmap.md
related_reports:
  - ../../../reports/closeouts/wr-018-rendered-world-v1/closeout.md
  - ../../../reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md
  - ../../../reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md
---

# WR-021 Material Lab And Material Preview Products Contract

## Goal

Establish implementation readiness for `WR-021` under the
`PM-SDF-OW-001` production product spine. The slice must turn Material Lab from
descriptor/status provider stubs into a source-backed material product workflow:
material graph source persistence, deterministic lowering, catalog-backed
formed material artifacts, viewport product selection, prior-valid
preservation, and prepared renderer handoff.

`WR-021` is promotable only as a full bounded V1 material product spine. It is
not acceptable to close this slice with canvas-local material truth, provider
text panels, direct renderer mutation, or catalog bypasses. This contract does
not implement product code or complete `WR-021`.

This revision also records the post-kickoff drift review: the current WR-021
implementation attempt is incomplete, does not yet compile, bypassed the
prepared batch worktree, and leaves material import recipes too string-shaped
for a durable Material Lab product spine. The implementation contract below
therefore adds hard reconciliation, typed-recipe, build, viewport, renderer,
shell, and closeout gates before `WR-021` may complete.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-SDF-OW` and active milestone `PM-SDF-OW-001`. The milestone links
  `WR-019`, `WR-026`, and `WR-021`, and requires product workflows to avoid
  parallel viewers, parallel truth stores, and renderer-owned semantic sources.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-021`.
  The row is `current_candidate`, blocker `B2`, depends on `WR-018:completed`,
  `WR-019:completed`, and `WR-026:completed`, and names source-backed
  `MaterialGraphDocument`, lowering/ratification, preview product,
  diagnostics, and render handoff as next evidence.
- `docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md`
  is the active owning design. It states that `MaterialGraphDocument` is source
  truth, canvas state is projection, V1 requires failed preview preserving the
  prior valid artifact, and providers fail closed until product handoff is
  available.
- `docs-site/src/content/docs/design/active/editor-asset-pipeline-and-content-workflow-design.md`
  owns source-backed asset/catalog policy. `WR-021` must consume the `WR-026`
  source-backed adapter path instead of adding a separate material truth store.
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`
  owns FR-4 sequencing. Material preview products must hand off renderer-ready
  data without moving material semantics into renderer code.

Readiness checks completed for this revision:

- `task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-021`
  classifies the current next action as `write_implementation_contract`.
- `task planning:validate` must pass before product-code implementation resumes
  and again before implementation closeout. The current known validation blocker
  is the active WR-021 batch state remaining approved with implementation and
  closeout not started; resolve that through the prepared batch workflow before
  accepting implementation or closeout evidence.

## Readiness

Promotion verdict: `WR-021` can honestly carry a bounded implementation
contract, but remains `current_candidate` until implementation, validation, and
closeout evidence land.

The dependencies are satisfied:

- `WR-018` completed rendered-world V1 and stabilized the viewport packet /
  product handoff path.
- `WR-019` completed the field visualizer product workflow and product-aware
  viewport routing evidence.
- `WR-026` completed source-backed asset editor adapters and the project
  catalog/import adapter spine.

The remaining pre-implementation gate is write-scope governance. The current
`WR-021` row includes `apps/runenwerk_editor`, `domain/editor`,
`domain/material_graph`, active design docs, and the renderer roadmap. Full V1
material preview handoff also needs explicit write scopes for
`engine/src/plugins/render`, `domain/editor/editor_viewport`,
`apps/runenwerk_editor/src/shell/providers`, shell command dispatch, the app
material workflow module, and the `WR-021` closeout path. Implementation must
not start until those scopes are added to the roadmap row, rendered, validated,
and checked.

ADR requirement: no ADR is required while `domain/material_graph` remains
material semantic truth and renderer/app runtime only consume formed products
and prepared data. Start architecture governance before implementation if
material graph ownership moves out of `domain/material_graph`, if renderer code
becomes material source authority, or if app runtime state becomes canonical
material truth.

## Current Drift And Reconciliation Gate

The current in-progress WR-021 implementation exists in the root `main`
checkout, while the accepted batch worktree branch is
`codex/2026-05-16-next-current-candidate-roadmap-batch-wr-021-wr-021`.
Implementation must happen in
`C:/Users/joshi/Projekte/Runenwerk-worktrees/2026-05-16-next-current-candidate-roadmap-batch-wr-021/WR-021`
on that branch. Product-code changes currently in root `main` must be replayed
or moved into the prepared WR-021 batch worktree before implementation resumes.
The implementation path has no roadmap-note escape hatch.

The approved WR-021 batch state must also be advanced or reconciled so
`task planning:validate` no longer reports an approved/not-started batch as
stale or unindexed active work. The batch TOML must advance from
`integration_status = "not_started"` and `closeout_status = "not_started"`
before closeout can be accepted.

`cargo check -p runenwerk_editor` must pass before any closeout evidence is
accepted. A compile failure is a blocking readiness failure, not partial
evidence. The current draft closeout file remains non-evidence until it is
replaced with implementation details, validation output, and roadmap/production
notes.

## Implementation Scope

Owning domain and crate boundaries:

- `domain/material_graph` owns source material graph documents, material graph
  ids, semantic ratification, deterministic lowering, formed material products,
  source maps, material cache keys, specialization fragments, and material
  product descriptors.
- `domain/asset` owns project source/catalog/artifact identity, import settings,
  deterministic import plans, diagnostics, ratification, and prior-valid
  preservation contracts. Serialized `ImportSettings::MaterialGraph` may carry
  project policy input, but it must not become the runtime material lowering
  recipe or learn renderer internals.
- `domain/editor/editor_shell` owns typed material view models, local material
  surface actions, and epoch-carrying command proposals. It must not perform IO,
  lowering, catalog mutation, renderer mutation, or ratification.
- `domain/editor/editor_viewport` owns typed viewport expression product kinds,
  product descriptors, presentation selection, and product-target contracts.
- `apps/runenwerk_editor` owns material workflow host IO, typed material recipe
  resolution, asset/material id bridging, catalog publication, preview
  orchestration, diagnostics mapping, prior-valid preservation, provider
  routing, and viewport product registry adapters.
- `engine/src/plugins/render` owns prepared material feature consumption only.
  It may consume specialization fragments and parameter blobs through existing
  prepared-data contracts; it must not load, save, ratify, or lower material
  graphs.

Required V1 implementation steps:

1. Add versioned material graph source persistence owned outside renderer code.
   The source document must round-trip a `material_graph::MaterialGraphDocument`
   with explicit versioning and deterministic source identity. Host IO belongs
   in the editor app or persistence layer, not in `domain/material_graph`.
2. Define the asset/material identity bridge. A material graph asset must map
   deterministically from `asset::AssetId` / `asset::AssetSourceId` to
   `material_graph::MaterialGraphDocumentId`. A formed material product must map
   deterministically to `material_graph::MaterialProductId`,
   `asset::AssetArtifactId`, catalog artifact payload, source lineage, and
   prior-valid preservation identity.
3. Add `apps/runenwerk_editor/src/material_lab/recipes.rs` as the typed material
   recipe boundary before lowering. Define `MaterialLoweringRecipe` with exactly
   two V1 variants: `PreviewMaterial` and `RenderMaterial`.
4. Define `ResolvedMaterialLoweringRecipe` as the runtime lowering contract used
   by `rebuild_material_preview_for_asset`. It owns material output target,
   material node catalog selection, stable cache-key component, expected
   artifact kind, specialization policy, and renderer-prepared parameter
   profile. Both V1 recipes use `MaterialNodeCatalog::first_slice()` until a
   later contract introduces multiple node catalogs.
5. Treat `ImportSettings::MaterialGraph { lowering_target }` as serialized
   project policy only. It must resolve to `ResolvedMaterialLoweringRecipe`
   before import ledger allocation or material graph lowering:
   - `"preview"` maps to `MaterialLoweringRecipe::PreviewMaterial` with
     `MaterialOutputTarget::PbrPreview`.
   - `"render_material"` maps to `MaterialLoweringRecipe::RenderMaterial` with
     `MaterialOutputTarget::RenderMaterial`.
   - Empty targets, unknown targets, mismatched source kinds, and any expected
     artifact target other than `AssetKind::Material` emit blocking
     `ImportProfileRejected` diagnostics.
6. Unknown, incompatible, or ambiguous material recipe resolution must stop
   before import ledger allocation, material graph lowering, catalog artifact
   publication, or renderer prepared-data publication.
7. Add `apps/runenwerk_editor/src/material_lab/` for material document IO,
   source/catalog identity bridging, lowering orchestration, diagnostics
   mapping, preview publication, and prior-valid preservation. This module owns
   app workflow state only; `domain/material_graph` remains semantic truth.
8. Route Material Lab through the `WR-026` project session and catalog adapters.
   Valid material previews publish formed material artifacts through guarded
   catalog/runtime barriers. Invalid material graphs or failed preview
   publication must record diagnostics and preserve the previous valid material
   artifact.
9. Extend `domain/editor/editor_shell` with typed material surface view models
   and actions for Material Graph Canvas, Material Inspector, and Material
   Preview. Every material workflow shell command must carry
   `projection_epoch`, and stale commands must fail closed.
10. Update `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs`,
   `material_inspector.rs`, and `material_preview.rs` to render typed material
   projections and route actions through app-owned material workflow adapters.
   Existing status lines may remain as diagnostics only; they are not acceptance
   evidence.
11. Extend `domain/editor/editor_viewport` and
   `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs` with a
   material preview expression product kind/target, and update viewport query
   snapshot authority classification so `DerivedMaterial` is treated as
   deterministic derived product authority. Material preview must be selectable
   through the existing viewport product selection path rather than merely
   appended to descriptor lists or shown in a parallel viewer.
12. Integrate renderer handoff through existing prepared-data contracts:
   `engine::plugins::render::frame::PreparedMaterialFeatureContribution`,
   `PreparedMaterialInstanceInput`,
   `engine::plugins::render::features::PreparedMaterialFeatureResource`, and
   `MATERIAL_RENDER_FEATURE_ID`. Renderer code consumes prepared material data
   only; material graph lowering and source truth stay outside renderer code.

Required write-scope update before implementation:

- Add `engine/src/plugins/render`
- Add `domain/editor/editor_viewport`
- Add `apps/runenwerk_editor/src/material_lab`
- Add `apps/runenwerk_editor/src/material_lab/recipes.rs`
- Add `apps/runenwerk_editor/src/shell/providers`
- Add `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- Add `apps/runenwerk_editor/src/editor_app/state.rs`
- Add `docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md`

Implementation must not start until the write-scope update is rendered and
validated.

Non-goals:

- No product code in this contract-writing slice.
- No canvas-local material truth.
- No renderer-owned material graph source, lowering, ratification, or catalog
  authority.
- No direct renderer mutation from providers.
- No catalog or product publication bypass.
- No status-panel-only Material Lab closeout.
- No string-only material lowering policy as runtime recipe authority.
- No inactive descriptor-only material preview that cannot be selected through
  the existing viewport product-selection path.
- No compile-failing implementation or placeholder closeout as completion
  evidence.
- No prefab material binding before prefab V2 has source/catalog identity.
- No broad material node editor, shader authoring language, texture generation
  system, marketplace importer, or external material plugin framework.
- No roadmap completion update for `WR-021` or `PM-SDF-OW-001` during
  implementation.

## Acceptance Criteria

- A source-backed material graph document can round-trip from project-owned
  storage and is addressable from a material graph asset source descriptor.
- Invalid material graphs block lowering with diagnostics and do not publish
  catalog artifacts or renderer data.
- Valid material graphs lower to `material_graph::FormedMaterialProduct` with
  deterministic cache keys, source maps, specialization fragments, and product
  descriptors.
- Material graph project import policy resolves into
  `ResolvedMaterialLoweringRecipe` before lowering; `"preview"` and
  `"render_material"` are the only accepted serialized V1 targets.
- Unknown, empty, mismatched-source, or non-`AssetKind::Material` recipe targets
  produce blocking `ImportProfileRejected` diagnostics before ledger allocation.
- Formed material products publish as catalog artifacts and runtime material
  products through the guarded app publication barrier.
- Failed material lowering or preview publication preserves the prior valid
  material artifact, exposes that preserved state in provider projections, and
  does not publish invalid renderer data.
- Material preview is selectable through viewport product selection and is not
  implemented as a parallel viewer.
- Material providers expose typed controls/actions and reject stale projection
  epochs before mutating material workflow state.
- Renderer handoff consumes prepared material data through
  `PreparedMaterialFeatureContribution` / `PreparedMaterialFeatureResource`
  without becoming material graph truth.
- `cargo check -p runenwerk_editor` passes before closeout evidence is accepted.

## Stop Conditions

Stop implementation and return to planning if any of these happen:

- Material graph/canvas/provider/session state becomes semantic material truth.
- Renderer code starts owning material graph lowering, ratification, source
  document IO, or catalog identity.
- Formed material products cannot be mapped deterministically to both material
  product ids and asset artifact ids.
- Material import policy remains raw string runtime behavior instead of
  resolving into a typed material recipe.
- Preview publication needs to mutate catalog/runtime/renderer state outside
  the guarded publication barrier.
- Stale shell commands can mutate material workflow state.
- Failed material preview cannot preserve the prior valid artifact.
- `ExpressionSourceRealityClass::DerivedMaterial` cannot be integrated into
  query snapshot authority classification as deterministic derived authority.
- Engine renderer files are required but roadmap write scopes have not been
  expanded, rendered, and validated.
- The implementation cannot provide real viewport product selection and would
  close with status-panel-only provider behavior.
- The implementation is not reconciled with the accepted WR-021 batch
  branch/worktree.
- `task planning:validate` does not pass before product-code implementation
  resumes or before closeout.

## Validation

Contract validation:

- `task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-021`
- `task planning:validate`
- `task docs:validate`

Future implementation validation:

- `cargo check -p runenwerk_editor`
- `cargo test -p material_graph`
- `cargo test -p editor_shell material`
- `cargo test -p editor_viewport material`
- `cargo test -p runenwerk_editor material`
- `cargo test -p runenwerk_editor viewport`
- `cargo test -p runenwerk_editor material_recipe`
- `cargo test -p runenwerk_editor material_epoch`
- `cargo test -p runenwerk_editor material_handoff`
- `cargo test -p runenwerk_editor material_viewport`
- Renderer/product handoff test proving prepared material data reaches
  `MATERIAL_RENDER_FEATURE_ID`.
- Material shell/provider tests proving provider-created material commands carry
  `projection_epoch` and stale material commands do not mutate workflow state.
- `task docs:validate`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:validate`
- `task production:check`

## Closeout Requirements

The implementation closeout must be written to
`docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md`
and must include:

- evidence that the implementation was reconciled with the accepted WR-021 batch
  branch/worktree and that the batch TOML advanced from
  `integration_status = "not_started"` / `closeout_status = "not_started"`;
- evidence that `cargo check -p runenwerk_editor` passed before closeout;
- exact source documents, modules, and command paths changed;
- evidence that material project import policy resolves through
  `apps/runenwerk_editor/src/material_lab/recipes.rs` into
  `ResolvedMaterialLoweringRecipe`, accepts only `"preview"` and
  `"render_material"` as V1 targets, and blocks unknown/empty/incompatible
  recipes before ledger allocation;
- evidence that source-backed material graph round trip passed;
- evidence that invalid graphs block lowering and preserve prior valid state;
- evidence that formed material products publish through catalog/runtime
  barriers;
- evidence that material preview is selectable through the existing viewport
  product-selection path and `DerivedMaterial` participates in query snapshot
  authority classification;
- evidence that renderer handoff uses prepared material data only and reaches
  `MATERIAL_RENDER_FEATURE_ID`;
- evidence that material provider actions carry projection epochs and stale
  shell commands fail closed;
- validation command output for all required gates;
- roadmap and production evidence updates, followed by render/validate/check
  gates.
- `task planning:validate` output passing before implementation resume and
  before closeout.

Only after implementation evidence and closeout land may `WR-021` move from
`current_candidate` to `completed`.
