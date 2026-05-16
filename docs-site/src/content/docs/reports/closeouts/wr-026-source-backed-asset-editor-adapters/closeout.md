---
title: WR-026 Source-backed Asset Editor Adapters Closeout
description: Completion and drift-check record for app-owned source-backed asset editor adapters over domain asset contracts.
status: completed
owner: editor
layer: app-runtime / editor-ui
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-index.md
related_reports:
  - ../../implementation-plans/wr-026-source-backed-asset-editor-adapters/plan.md
  - ../wr-020-source-backed-asset-core-contracts/closeout.md
---

# WR-026 Source-backed Asset Editor Adapters Closeout

## Status

Complete as of 2026-05-16.

WR-026 moves the editor asset workflow from status-only runtime projection into
source-backed app adapters over `domain/asset`. The implementation keeps
semantic asset truth in `domain/asset`, while `apps/runenwerk_editor` owns
project-root IO, catalog load/save, deterministic import execution identity,
provider routing, shell dispatch, and UI diagnostics.

Before product code changes, the WR-026 row was updated from the stale B3
exclusion note to the accepted B2 implementation gate, its write scopes were
expanded to include asset pipeline adapters, providers, shell dispatch, app
state, the implementation plan, and this closeout path, and the roadmap was
rendered, validated, and checked.

## Owning Scope

- `apps/runenwerk_editor/src/asset_pipeline/project_session.rs::EditorAssetProjectSession`
  owns the active app-side asset project root, domain project catalog
  descriptor, catalog status lines, and the imported execution ledger handle.
- `apps/runenwerk_editor/src/asset_pipeline/catalog_persistence.rs` owns RON
  catalog load/save, parent directory creation, decode diagnostics, and
  ratification-before-publication for `asset::AssetCatalog`.
- `apps/runenwerk_editor/src/asset_pipeline/import_orchestration.rs::EditorImportExecutionLedger`
  owns durable execution identity persisted at `.runenwerk/import-jobs.ron`.
- `apps/runenwerk_editor/src/asset_pipeline/import_orchestration.rs::execute_import_for_asset`
  owns deterministic plan creation, ledger reuse/allocation, plan ratification,
  job execution, prior-valid preservation handoff, and publication candidate
  formation.
- `apps/runenwerk_editor/src/asset_pipeline/import_jobs.rs::run_import_job`
  now consumes the ledger-provided `asset::AssetArtifactId` instead of deriving
  artifact identity from the job id.
- `apps/runenwerk_editor/src/editor_app/state.rs::RunenwerkEditorApp` owns the
  optional asset project session and exposes catalog load/save/reimport
  commands to shell dispatch.
- `domain/editor/editor_shell/src/surfaces/asset.rs` owns typed Asset Browser
  and Import Inspector view models/actions using `asset::AssetId`,
  `asset::AssetSourceId`, `asset::AssetArtifactId`, and `asset::AssetKind`.
- `domain/editor/editor_shell/src/commands/shell_command.rs::ShellCommand`
  owns epoch-carrying asset workflow shell commands.
- `apps/runenwerk_editor/src/shell/providers/asset_browser.rs::build_frame` and
  `AssetBrowserProvider::map_action` own full asset browser projection and
  provider action routing.
- `apps/runenwerk_editor/src/shell/providers/import_inspector.rs::build_frame`
  and `ImportInspectorProvider::map_action` own import diagnostics, dirty asset,
  prior-valid, and reimport routes.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::dispatch_shell_command_with_viewport_commands`
  owns asset command dispatch to app adapters and stale projection rejection via
  the existing shell epoch gate.

## Completion Evidence

- The app now has a durable `EditorImportExecutionLedger` serialized as RON at
  `.runenwerk/import-jobs.ron` under the project root.
- Ledger entries map deterministic `asset::ArtifactCacheKey` values to stable
  `asset::ImportJobId` and `asset::AssetArtifactId` values.
- Existing ledger entries are reused for identical deterministic cache keys.
  New identities allocate the next nonzero job id above the ledger maximum and
  the next nonzero artifact id above both ledger and catalog maxima.
- Duplicate ledger job ids, duplicate ledger artifact ids, duplicate ledger
  cache keys, catalog cache-key collisions, and ledger/catalog cache-key
  disagreement produce blocking diagnostics and stop import publication.
- Catalog load only replaces `AssetCatalogRuntime` after decode and
  `asset::ratify_asset_catalog` acceptance.
- Catalog save ratifies the current catalog before writing and refuses invalid
  catalog state without success-shaped IO.
- Import execution ratifies the generated `asset::ImportPlan` against the
  selected source descriptor before any job output is published.
- Failed imports use `domain/asset` prior-valid preservation contracts through
  `asset::try_preserve_prior_valid_artifact`.
- Asset Browser and Import Inspector now expose routes for catalog load/save,
  asset selection, selected/dirty asset reimport, and diagnostic clearing.
- Every new asset workflow `ShellCommand` carries a `projection_epoch`, is
  included in `ShellCommand::projection_epoch`, and is labeled in
  `shell_command_label`.
- Provider `map_action` implementations create only epoch-carrying asset shell
  commands. Providers remain projection/routing code and do not perform host IO
  or import execution.

## Drift Findings

- No ADR is required. The implementation did not move source/catalog/import,
  diagnostic, ratification, or artifact truth out of `domain/asset`.
- `domain/editor/editor_shell` gained a narrow `asset.workspace = true`
  dependency for typed ids, asset kind display metadata, and IO-free view
  models/actions only.
- Renderer code did not gain asset catalog authority.
- The app remains the correct adapter owner for project-root IO, RON files,
  import execution, and shell dispatch.
- Material Lab, prefab runtime instancing, terrain/world substrate work,
  external importer expansion, cache garbage collection, and runtime hot-reload
  streams remain separate roadmap slices.

## Validation

Implementation validation completed on 2026-05-16:

- `cargo test -p editor_shell asset` passed: typed asset view model/action
  contract and asset command epoch coverage.
- `cargo test -p runenwerk_editor asset` passed: 21 filtered asset/app tests
  plus the matching viewport architecture guard, including ledger reuse,
  allocation, rejection, malformed ledger blocking, catalog load/save, provider
  routes, stale asset command rejection, and prior-valid preservation.
- `cargo test -p runenwerk_editor catalog` passed: 12 filtered catalog/app tests
  plus 2 matching viewport architecture guard tests.
- `cargo test -p asset` passed: 18 domain asset tests.
- `cargo check -p runenwerk_editor` passed after `cargo clean -p editor_viewport`
  removed stale local build metadata.

## Deferred Work

- Material Lab (`WR-021`) remains the source-backed material authoring and
  preview-product workflow slice.
- SDF Prefab V2 (`WR-022`) remains the prefab source/catalog and runtime
  instancing slice.
- External importer tool policy, package/cache garbage collection, runtime
  hot-reload streams, terrain/world substrate products, and renderer storage
  packet consumers remain outside WR-026.
