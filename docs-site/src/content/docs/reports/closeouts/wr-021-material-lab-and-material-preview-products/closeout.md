---
title: WR-021 Material Lab And Material Preview Products Closeout
description: Implementation closeout for source-backed Material Lab documents, typed material recipes, catalog-backed preview artifacts, viewport selection, and renderer prepared-data handoff.
status: completed
owner: apps/runenwerk_editor
layer: domain / app-runtime / editor-ui / engine-render
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/material-lab-and-material-preview-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_reports:
  - ../../../reports/implementation-plans/wr-021-material-lab-and-material-preview-products/plan.md
---

# WR-021 Material Lab And Material Preview Products Closeout

## Status

Complete as of 2026-05-16.

WR-021 now implements the first source-backed Material Lab product spine. Material
graph source documents remain `domain/material_graph` truth; app runtime code owns
project IO, typed recipe resolution, import-ledger identity, catalog publication,
prior-valid preservation, viewport product selection, and renderer prepared-data
handoff.

## Owning Scope

- `domain/material_graph/src/authored.rs::MaterialGraphDocument` owns material
  source identity and output target.
- `domain/material_graph/src/persistence.rs::MaterialGraphSourceFileV1` owns
  versioned material graph source-file round trips.
- `domain/material_graph/src/lowering.rs::lower_material_graph` owns deterministic
  lowering, cache keys, source maps, specialization fragments, and formed material
  products.
- `apps/runenwerk_editor/src/material_lab/recipes.rs::ResolvedMaterialLoweringRecipe`
  owns the V1 typed runtime recipe boundary for serialized `"preview"` and
  `"render_material"` project policy.
- `apps/runenwerk_editor/src/material_lab/workflow.rs::rebuild_material_preview_for_asset`
  owns material source loading, recipe rejection before ledger allocation, lowering
  orchestration, ledger reuse/allocation, catalog artifact publication, and
  prior-valid preservation.
- `apps/runenwerk_editor/src/material_lab/publication.rs::publish_pending_material_preview_publications`
  owns ProductPublication barrier publication and active-preview state updates.
- `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs::prepared_material_resource_for_preview`
  owns renderer prepared material payload formation.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs::prepare_material_preview_render_resource_system`
  hands active material preview data to `PreparedMaterialFeatureResource`.
- `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs::material_preview_descriptor`
  and `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs` own material
  preview viewport product descriptors and selectable primary target routing.
- `apps/runenwerk_editor/src/runtime/viewport/query_snapshots.rs::product_authority`
  classifies `ExpressionSourceRealityClass::DerivedMaterial` as deterministic
  derived authority.
- `domain/editor/editor_shell/src/surfaces/material.rs` owns typed Material Lab
  view models/actions; shell/providers remain IO-free projection and routing code.
- `domain/editor/editor_shell/src/commands/shell_command.rs::ShellCommand` owns
  epoch-carrying material workflow commands, and
  `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::dispatch_shell_command_with_viewport_commands`
  rejects stale material commands before mutation.
- `engine/src/plugins/render/runtime/frame_prepare.rs` consumes
  `PreparedMaterialFeatureResource` into `MATERIAL_RENDER_FEATURE_ID` frame
  contributions.

## Completion Evidence

- Source-backed material graph documents round-trip through versioned RON source
  files and deterministic asset/source-derived document ids.
- Typed material recipes replace raw string runtime policy. The only V1 serialized
  material lowering targets are `"preview"` and `"render_material"`; empty,
  unknown, source-kind mismatched, or non-`AssetKind::Material` artifact targets
  produce blocking `ImportProfileRejected` diagnostics before ledger allocation.
- Material lowering uses `MaterialNodeCatalog::first_slice()` through the resolved
  recipe boundary, not through workflow-local string checks.
- Material recipe cache identity includes the import cache key, resolved recipe
  cache component, formed material cache key, source hash, source id, importer id,
  and expected artifact kind.
- `domain/asset::AssetKind::Material` is now a formed product kind, so material
  graph imports produce product jobs instead of bypassing runtime publication.
- Failed material source reads, document identity mismatches, document output
  target mismatches, and lowering failures preserve the prior valid material
  artifact without publishing invalid active preview or renderer data.
- Material preview publication goes through the ProductPublication barrier, records
  publication journals, updates active preview state only after accepted
  publication, and registers selectable viewport material preview products.
- Renderer handoff publishes prepared material instances through
  `PreparedMaterialFeatureResource` and the existing `MATERIAL_RENDER_FEATURE_ID`
  frame contribution path. Renderer code consumes prepared data and does not own
  material graph semantics.
- Material Graph Canvas, Material Inspector, and Material Preview providers project
  typed view models/actions and route commands through app-owned workflow
  adapters.
- Every material shell command carries `projection_epoch`; stale material commands
  return without mutating material selection, diagnostics, queued publications, or
  active preview state.

## Drift Findings

- No ADR is required. Material graph semantic ownership stayed in
  `domain/material_graph`, and renderer/app runtime did not become material source
  authority.
- The WR-021 implementation was reconciled into the prepared batch worktree
  `C:/Users/joshi/Projekte/Runenwerk-worktrees/2026-05-16-next-current-candidate-roadmap-batch-wr-021/WR-021`
  on branch `codex/2026-05-16-next-current-candidate-roadmap-batch-wr-021-wr-021`.
- Batch write scope was expanded for the necessary `Cargo.lock` and `domain/asset`
  changes because material source persistence adds a crate dependency and material
  artifacts must be recognized as formed products by the asset domain.
- Dynamic importer plugins and multiple material node catalogs remain deferred.
  WR-021 intentionally proves a typed recipe boundary first.

## Validation

Implementation validation completed on 2026-05-16:

- `cargo check -p runenwerk_editor` passed.
- `cargo test -p asset` passed: 25 tests.
- `cargo test -p material_graph` passed: 7 tests.
- `cargo test -p editor_shell material` passed: typed material view model/action
  and epoch command contract.
- `cargo test -p editor_viewport material` passed: material preview product
  descriptor classification.
- `cargo test -p runenwerk_editor material` passed: 18 material workflow/provider
  tests.
- `cargo test -p runenwerk_editor material_recipe` passed: recipe resolution,
  rejection, ledger non-allocation on rejected recipe, and changed-recipe cache
  split coverage.
- `cargo test -p runenwerk_editor material_epoch` passed: stale material commands
  produce no material workflow mutation.
- `cargo test -p runenwerk_editor material_handoff` passed: prepared material
  resource formation.
- `cargo test -p runenwerk_editor material_viewport` passed: material preview
  primary selection through viewport presentation state.
- `cargo test -p runenwerk_editor viewport` passed: 100 unit tests plus viewport
  smoke/architecture tests, with the GPU smoke still ignored by design.
- `cargo test -p engine material_handoff` passed: prepared material data reaches
  `MATERIAL_RENDER_FEATURE_ID`.
- `task batch:scope-check -- --batch docs-site/src/content/docs/reports/batches/2026-05-16-next-current-candidate-roadmap-batch-wr-021/batch.toml` passed.

Closeout validation after roadmap/production evidence updates:

- `task docs:validate` passed on the integrated root checkout.
- `task roadmap:render` completed and refreshed generated roadmap docs.
- `task roadmap:validate` passed: 26 items, 34 edges.
- `task roadmap:check` passed: schema and rendered roadmap docs are in sync.
- `task production:validate` passed: 1 track, 9 milestones.
- `task production:check` passed: schema and generated production docs are in sync.

## Deferred Work

- A broader dynamic material importer/plugin framework remains out of scope until
  a later WR/ADR proves stable extension points over the typed recipe boundary.
- Multiple material node catalogs and recipe-selected node catalog versions remain
  future work; both V1 recipes intentionally use `MaterialNodeCatalog::first_slice()`.
- Rich node editing UX, shader authoring, material package distribution, and
  renderer-side specialization cache management remain separate production slices.
