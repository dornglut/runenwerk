---
title: WR-026 Source-backed Asset Editor Adapters Contract
description: Promotion and implementation-readiness contract for editor adapters over source-backed asset contracts.
status: active
owner: apps/runenwerk_editor
layer: app-runtime / editor-ui
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../../../design/active/material-lab-and-material-preview-design.md
  - ../../../design/active/sdf-prefab-composition-system-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-track-index.md
  - ../../../workspace/roadmap-index.md
  - ../../../engine/roadmaps/fully-featured-renderer-roadmap.md
related_reports:
  - ../../../reports/closeouts/wr-020-source-backed-asset-core-contracts/closeout.md
---

# WR-026 Source-backed Asset Editor Adapters Contract

## Goal

Establish implementation readiness for `WR-026` under the `PM-SDF-OW-001`
production product spine. The slice must turn the editor-side asset pipeline
from mostly runtime-local state into source-backed project adapters over the
landed `domain/asset` contracts.

The first implementation is a bounded full V1: project catalog load/save,
deterministic import orchestration, diagnostics surfacing, and prior-valid
artifact workflow through a full Asset Browser / Import Inspector experience.
This contract does not implement product code, promote roadmap state, or
complete `WR-026`.

## Source Of Truth

- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns
  `PT-SDF-OW` and active milestone `PM-SDF-OW-001`. The milestone links
  `WR-019`, `WR-026`, and `WR-021`, and requires asset catalog adapters without
  parallel asset truth stores or renderer-owned semantic sources.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns `WR-026`.
  The row is `ready_next`, blocker `B3`, depends on `WR-020:completed`, and
  names project catalog load/save, deterministic import job orchestration,
  diagnostics surfacing, and prior-valid artifact preservation as next
  evidence.
- `docs-site/src/content/docs/design/active/editor-asset-pipeline-and-content-workflow-design.md`
  is the active owning design. It requires editor adapters to consume
  `domain/asset` contracts instead of becoming semantic asset truth.
- `docs-site/src/content/docs/reports/closeouts/wr-020-source-backed-asset-core-contracts/closeout.md`
  provides the completed upstream evidence: source roots, project catalog
  descriptors, importer-aware source descriptors, deterministic import plans,
  dependency graph contracts, diagnostics, catalog ratification, and checked
  prior-valid artifact preservation.
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`
  keeps renderer consumers downstream of source-backed products and must not be
  used as an asset catalog authority.

Readiness checks completed for this contract:

- `task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-026`
  classified the next action as `write_promotion_contract`.
- `task planning:validate` passed before the contract was finalized.

## Readiness

Promotion verdict: `WR-026` can honestly carry a bounded implementation
contract, but remains `ready_next` until implementation, validation, and closeout
evidence land.

This prompt is the explicit WR-026 selection that clears the old `B3` note that
the slice was excluded from the prior objective. That clearance applies only to
writing this contract and preparing a later implementation task; it is not a
roadmap state promotion, implementation approval, or closeout.

The remaining pre-implementation gate is write-scope governance. The current
WR-026 row names `apps/runenwerk_editor/src/asset_pipeline`, `domain/editor`,
the active asset-pipeline design, and the renderer roadmap. A full-browser V1
also needs `apps/runenwerk_editor/src/shell/providers/` and shell command
dispatch files. Before code starts, the WR-026 write scopes must be expanded
and rendered/validated through roadmap checks, or the implementation must be
reduced to a non-interactive adapter slice.

ADR requirement: no ADR is required while implementation keeps `domain/asset` as
asset semantic truth and confines host IO, job execution, provider behavior, and
runtime state to editor adapters. Start architecture governance before code if
source/catalog ownership moves out of `domain/asset`, if renderer code becomes
asset source authority, or if app runtime state becomes the canonical asset
store.

## Implementation Scope

Owning domain and crate boundaries:

- `domain/asset` owns asset ids, source descriptors, artifact descriptors,
  import settings, deterministic import plans, dependency graph contracts,
  diagnostics, catalog validation, project catalog descriptors, ratification,
  and prior-valid preservation contracts.
- `domain/editor/editor_persistence` owns versioned editor project DTOs and the
  RON codec. It may map project file DTOs to `domain/asset` descriptors but
  must not perform host IO beyond existing persistence helpers.
- `domain/editor/editor_shell` owns typed asset workflow surface contracts:
  asset browser view models, import inspector view models, local actions, and
  command proposal types. It may add a narrow `asset.workspace = true`
  dependency for typed asset identifiers and display metadata only.
- `apps/runenwerk_editor/src/asset_pipeline` owns editor project session state,
  catalog file load/save, host path resolution, import job orchestration,
  diagnostics capture, catalog publication timing, and prior-valid adapter
  workflow.
- `apps/runenwerk_editor/src/shell/providers` owns Asset Browser and Import
  Inspector presentation and maps user actions to app commands.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` owns execution of
  asset workflow shell commands against app-owned adapters.
- `engine` may consume formed products through existing product/runtime
  contracts only. It must not load, save, or ratify editor asset catalogs.

Required V1 interface and implementation steps:

1. Add `apps/runenwerk_editor/src/asset_pipeline/project_session.rs` and export
   it from `apps/runenwerk_editor/src/asset_pipeline/mod.rs`. The module owns
   `EditorAssetProjectSession`, with exactly these responsibilities: project
   root, `ProjectFileV2`, `AssetProjectCatalogDescriptor`, resolved catalog
   path, resolved artifact cache root, resolved field-product cache root, dirty
   catalog flag, and last load/save status.
2. Add `apps/runenwerk_editor/src/asset_pipeline/catalog_persistence.rs`. It
   must provide app-owned helpers to load and save `AssetCatalog` RON at the
   project catalog path, ratify with `asset::ratify_asset_catalog`, and update
   `AssetCatalogRuntime::replace_catalog` only after successful decode and
   ratification. Failed loads or saves record diagnostics and must not replace
   the prior valid in-memory catalog.
3. Extend `apps/runenwerk_editor/src/asset_pipeline/catalog_runtime.rs` with
   full-browser projection methods for asset rows, source rows, artifact rows,
   dependency rows, dirty summaries, diagnostics summaries, prior-valid
   artifact summaries, and selected asset/artifact details. These methods are
   projections only; they must not perform IO, import execution, or
   ratification.
4. Add `apps/runenwerk_editor/src/asset_pipeline/import_orchestration.rs`. It
   must select sources from the loaded catalog, resolve import execution
   identity through `EditorImportExecutionLedger`, create deterministic
   `asset::ImportPlan` values, ratify with
   `asset::ratify_asset_import_plan_against_source`, call existing
   `run_import_job` or `run_field_product_job`, preserve prior valid artifacts
   on failure, record diagnostics, and publish artifacts only through existing
   catalog/product publication boundaries.
5. Keep `apps/runenwerk_editor/src/asset_pipeline/import_jobs.rs::run_import_job`
   and `apps/runenwerk_editor/src/asset_pipeline/field_product_jobs.rs::run_field_product_job`
   as job executors. Do not move deterministic plan construction,
   source/catalog invariants, or artifact ratification out of `domain/asset`.

### Long-Term Identity Contract

`apps/runenwerk_editor/src/asset_pipeline/import_orchestration.rs` owns the
app-side `EditorImportExecutionLedger`. The ledger is persisted at
`.runenwerk/import-jobs.ron` under the project root and maps deterministic
`asset::ArtifactCacheKey` values to stable `asset::ImportJobId` and
`asset::AssetArtifactId` values.

Import orchestration must reuse an existing ledger entry for the same cache key.
When no entry exists, it allocates the next nonzero id above every import job id
and artifact id already present in the ledger and loaded catalog. This preserves
stable numeric identity across sessions without deriving public typed ids from a
hash.

Duplicate job ids, duplicate artifact ids, cache-key collisions, or
ledger/catalog disagreement are blocking diagnostics. In those cases, import
publication stops before any catalog mutation, artifact replacement, or product
publication. The ledger is execution identity state only; `domain/asset` remains
semantic asset truth for source, artifact, catalog, import-plan, diagnostic,
ratification, and prior-valid preservation semantics.

6. Add `asset.workspace = true` to
   `domain/editor/editor_shell/Cargo.toml`.
7. Add `domain/editor/editor_shell/src/surfaces/asset.rs` and re-export it from
   `domain/editor/editor_shell/src/surfaces/mod.rs`. It owns typed view models
   and actions for `AssetBrowserSurfaceAction` and
   `ImportInspectorSurfaceAction`: select asset, select artifact, save catalog,
   load catalog, reimport selected source, reimport dirty assets, clear
   diagnostics, and reveal prior-valid artifact details. Use
   `asset::AssetId`, `asset::AssetSourceId`, `asset::AssetArtifactId`, and
   `asset::AssetKind`; do not introduce parallel editor-shell asset id wrappers.
8. Extend `domain/editor/editor_shell/src/surface_provider.rs::SurfaceLocalAction`
   with asset browser and import inspector variants. Keep the action payloads
   typed by asset ids, source ids, artifact ids, and stable command intent; do
   not encode host paths or raw provider strings as semantic identifiers.
9. Extend `domain/editor/editor_shell/src/commands/shell_command.rs::ShellCommand`
   with asset workflow commands for load catalog, save catalog, select asset,
   select artifact, reimport selected source, reimport dirty assets, clear
   diagnostics, and show prior-valid details. Every new asset workflow command
   variant must carry `projection_epoch`, and
   `ShellCommand::projection_epoch` must return it so stale projected UI actions
   fail closed.
10. Update `apps/runenwerk_editor/src/shell/providers/asset_browser.rs::build_frame`
   to render the full browser projection: asset list, source and artifact
   counts, selected asset details, dependency summary, dirty state, reload
   status, prior-valid markers, and action routes. Update
   `AssetBrowserProvider::map_action` to return only epoch-carrying shell
   command proposals.
11. Update `apps/runenwerk_editor/src/shell/providers/import_inspector.rs::build_frame`
    to render import queue/status, deterministic plan summaries, diagnostics,
    failed-preserved artifacts, dirty asset summary, and reimport routes. Update
    `ImportInspectorProvider::map_action` to return only epoch-carrying shell
    command proposals.
12. Update `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` to route
    the new asset workflow commands into the app-owned project session,
    persistence, and orchestration adapters. Every failure must append a console
    diagnostic and leave the prior valid catalog/artifact state intact. Add
    every new asset workflow command to `shell_command_label` so diagnostics and
    debug traces stay explicit.
13. Update `apps/runenwerk_editor/src/editor_app/state.rs::RunenwerkEditorApp`
    to own the optional `EditorAssetProjectSession` beside
    `AssetCatalogRuntime`. Keep semantic asset data in the catalog runtime and
    session descriptors; do not duplicate catalog truth in shell provider state.
14. Preserve existing provider surfaces that consume catalog projections
    (`FieldProductViewerProvider`, `SdfBrushBrowserProvider`, material and
    texture providers). They may display new projection lines but must not gain
    direct catalog mutation paths.

Required write-scope update before implementation:

- Add `apps/runenwerk_editor/src/shell/providers`
- Add `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs`
- Add `apps/runenwerk_editor/src/editor_app/state.rs`
- Add `docs-site/src/content/docs/reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md`

Implementation must not start until the write-scope update is rendered and
validated.

Non-goals:

- No product code in this contract-writing slice.
- No renderer-owned asset catalog, source descriptor, import plan, diagnostic,
  or ratification authority.
- No app-owned replacement for `domain/asset` invariants.
- No external importer redesign, Blender tool redesign, marketplace/package
  workflow, package cache garbage collection, or live runtime hot-reload stream.
- No Material Lab authoring, prefab runtime instancing, terrain production,
  water, vegetation, physics, animation, gameplay graph, or world simulation
  implementation.
- No completion update for `WR-026` or `PM-SDF-OW-001` during implementation.

## Acceptance Criteria

- Project catalog load reads the project-selected `assets/catalog.ron` path,
  decodes `asset::AssetCatalog`, ratifies it, replaces
  `AssetCatalogRuntime` only on accepted catalog state, and reports controlled
  diagnostics on missing, malformed, or rejected catalogs.
- Project catalog save writes the current ratified catalog to the project
  catalog path, creates parent directories as needed, records save status, and
  does not write invalid catalog state.
- Import orchestration produces deterministic plans from source descriptors and
  import settings, rejects source/plan mismatches before execution, and records
  diagnostics without silently succeeding.
- Import orchestration reuses `EditorImportExecutionLedger` entries for
  identical deterministic cache keys, allocates new ids above ledger/catalog
  maxima, and rejects duplicate ids or ledger/catalog disagreement before
  publication.
- Failed imports preserve prior valid artifacts through `domain/asset`
  preservation contracts and surface the preserved artifact in Asset Browser and
  Import Inspector.
- Asset Browser provides interactive asset selection, selected details,
  source/artifact/dependency summaries, dirty state, prior-valid markers, and
  load/save/reimport routes.
- Import Inspector provides interactive reimport controls, plan summaries,
  diagnostics, queue/status summaries, dirty asset summary, and prior-valid
  artifact details.
- Provider actions are projection-epoch guarded and route through shell command
  dispatch. Provider build functions remain projections and do not execute IO
  or import jobs.
- `domain/editor/editor_shell` asset workflow types use `asset` typed ids only,
  compile without app/runtime dependencies, and remain IO-free.
- Existing field-product publication behavior still updates the catalog only at
  the product publication barrier.

## Validation

Run these gates for the implementation:

```text
cargo test -p asset
cargo test -p editor_shell asset
cargo test -p runenwerk_editor asset
cargo test -p runenwerk_editor catalog
cargo check -p runenwerk_editor
task roadmap:render
task production:render
task planning:validate
```

Required implementation tests:

- `domain/asset/src/catalog.rs` and `domain/asset/src/ratification.rs` keep
  catalog invariants and reject invalid project/source/artifact state.
- `domain/editor/editor_shell/src/surfaces/asset.rs` tests typed asset browser
  and import inspector actions/view models without app IO.
- `apps/runenwerk_editor/src/asset_pipeline/catalog_persistence.rs` tests
  accepted catalog load/save, rejected catalog preservation, missing catalog
  diagnostics, and save rejection for invalid catalog state.
- `apps/runenwerk_editor/src/asset_pipeline/import_orchestration.rs` tests
  deterministic plan orchestration, ledger reuse for identical cache keys, new
  id allocation above ledger/catalog maxima, duplicate id rejection,
  ledger/catalog disagreement rejection, source mismatch rejection, failed
  import preservation, dirty-asset reimport, and diagnostic capture.
- `apps/runenwerk_editor/src/shell/providers/asset_browser.rs::build_frame` and
  `AssetBrowserProvider::map_action` tests prove full-browser routes and
  projection-only behavior.
- `apps/runenwerk_editor/src/shell/providers/import_inspector.rs::build_frame`
  and `ImportInspectorProvider::map_action` tests prove import routes,
  diagnostics, and prior-valid detail surfacing.
- `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs` tests prove every
  asset workflow shell command carries a projection epoch, appears in
  `ShellCommand::projection_epoch`, appears in `shell_command_label`, calls app
  adapters, preserves prior valid state on failure, and rejects stale projection
  commands.

## Stop Conditions

Stop implementation and write design or architecture-governance work if:

- implementation requires renderer code to own asset source, catalog,
  diagnostic, import, or ratification semantics;
- implementation needs to move source/catalog truth out of `domain/asset`;
- full-browser UI requires provider write scopes that have not been added to
  the WR-026 row and rendered/validated;
- catalog persistence requires a new public project-file format beyond additive
  use of `ProjectFileV2` and `AssetProjectCatalogDescriptor`;
- import orchestration requires broad external tool execution policy beyond
  existing `run_import_job` and `run_field_product_job` contracts;
- import execution identity cannot be made stable through
  `EditorImportExecutionLedger` without weakening typed id guarantees or
  catalog ratification;
- failed imports cannot preserve prior valid artifact state without weakening
  ratification;
- implementation expands into Material Lab, prefab runtime instancing, terrain,
  renderer storage-buffer scene packets, hot-reload streams, or package/cache
  garbage collection.

## Closeout Requirements

The future WR-026 closeout must include:

- exact files changed and owning modules for asset project session, catalog
  persistence, import orchestration and ledger identity, typed asset surface
  contracts, providers, shell dispatch, and app state;
- evidence that WR-026 write scopes were expanded before implementation, then
  rendered and validated;
- test evidence for catalog round trip, deterministic job orchestration, failed
  import preservation, ledger id reuse/allocation/rejection, diagnostics
  surfacing, full-browser routes, and stale projection rejection;
- confirmation that `domain/asset` remains semantic asset truth and that
  renderer code did not gain asset catalog authority;
- confirmation that `domain/editor/editor_shell` only depends on `asset` for
  typed ids and display metadata, with no app IO, catalog mutation, import
  execution, or ratification;
- explicit deferred work for Material Lab (`WR-021`), SDF Prefab V2 (`WR-022`),
  terrain/world substrate work, external importer expansion, cache garbage
  collection, and runtime hot-reload streams.

After implementation and closeout evidence land, update
`docs-site/src/content/docs/workspace/roadmap-items.yaml` for `WR-026`, then run:

```text
task roadmap:render
task production:render
task planning:validate
```

Only update `PM-SDF-OW-001` evidence if the WR-026 closeout changes production
milestone evidence. Do not mark `PM-SDF-OW-001` complete until its linked WR
rows and milestone acceptance criteria are satisfied.
