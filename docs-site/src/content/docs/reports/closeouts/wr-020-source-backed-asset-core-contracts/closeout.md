---
title: WR-020 Source-backed Asset Core Contracts Closeout
description: Completion and drift-check record for domain-owned source/catalog/import/dependency/diagnostic/ratification asset contracts.
status: completed
owner: asset
layer: domain
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../../../design/active/editor-rendered-world-and-multi-entity-viewport-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-index.md
related_reports:
  - ../wr-018-rendered-world-v1/closeout.md
---

# WR-020 Source-backed Asset Core Contracts Closeout

## Status

Complete as of 2026-05-16.

WR-020 lands engine-agnostic asset truth in `domain/asset` after WR-018 rendered-world V1 closeout evidence. It does not start WR-026 editor adapters, project-catalog load/save UI, import job orchestration UI, material graph authoring, field visualizer routing, prefab runtime instancing, or external importer execution.

Implementation started only after the WR-018 closeout was written, roadmap docs were rendered and validated, and `WR-020` was promoted to `planning_state=current_candidate`. This final closeout moves `WR-020` from that temporary current-candidate state to completed evidence.

## Owning Scope

- `domain/asset/src/catalog.rs::AssetCatalog` owns V1 catalog records, source roots, sources, artifacts, and dependency graph state while preserving backward-compatible deserialization for catalogs that predate `source_roots`.
- `domain/asset/src/project_catalog.rs` owns project catalog descriptor contracts for unique source roots, artifact roots, catalog path, and import profile defaults.
- `domain/asset/src/source.rs::AssetSourceDescriptor` owns durable source identity, source hash, provenance, root membership, and importer choice.
- `domain/asset/src/import_settings.rs::ImportSettings` owns separate source-kind and artifact-kind compatibility for SDF graph, field product descriptor, material graph/material, prefab, UI definition, texture, shader, scene, raw RON, and foreign-reference sources.
- `domain/asset/src/import_plan.rs::ImportPlan` owns deterministic artifact-kind-aware cache keys, expected artifacts, dependencies, validation requirements, and product-job descriptors without executing host IO.
- `domain/asset/src/dependency_graph.rs::AssetDependencyGraph` owns deterministic dependency edges and invalidation order contracts.
- `domain/asset/src/artifact.rs::try_preserve_prior_valid_artifact` owns the checked failed-import preservation contract.
- `domain/asset/src/ratification.rs` owns source, artifact, catalog, import-plan, and project-catalog descriptor ratification.

## Completion Evidence

- `domain/asset/src/project_catalog.rs::AssetProjectCatalogDescriptor` records source roots, artifact cache root, field-product cache root, catalog file path, and import profile defaults as IO-free domain descriptors.
- `domain/asset/src/source.rs::AssetSourceDescriptor` includes optional `AssetImporterId`, so importer choice participates in durable source identity without making source paths the only identity.
- `domain/asset/src/import_plan.rs::deterministic_cache_key` includes asset id, source id, importer choice, import-settings label, expected artifact kind, and source hash in deterministic cache identity, so same-source/same-settings imports cannot collide across output kinds.
- `domain/asset/src/import_settings.rs::ImportSettings::supports_source_kind` and `ImportSettings::supports_artifact_kind` split input/source compatibility from output/artifact compatibility.
- `domain/asset/src/ratification.rs::ratify_asset_catalog` composes source and artifact ratifiers before topology checks, rejecting invalid source paths, artifact publication paths, missing source assets, missing source roots, primary-source ownership mismatches, missing artifact sources, source/artifact asset mismatches, missing dependency assets, self-dependencies, and unsupported catalog versions.
- `domain/asset/src/ratification.rs::ratify_asset_import_plan_against_source` validates plan/source identity, source hash, required expected artifacts, expected artifact cache-key equality, duplicate/self dependencies, source-kind import-settings compatibility, and artifact-kind import-settings compatibility before an adapter can execute a job.
- `domain/asset/src/ratification.rs::ratify_asset_project_catalog_descriptor` validates project catalog descriptor version, unique source root ids, unique source root paths, non-empty import profile names, import profile setting/source-kind compatibility, and strict project-relative source/cache/catalog paths.
- `domain/asset/src/artifact.rs::try_preserve_prior_valid_artifact` rejects preservation unless the previous artifact is `ArtifactValidity::Valid`; successful preservation keeps identity/path and attaches the new diagnostic.
- `tools/workflow/roadmap_state.py::validate_completion_evidence` now prevents `/goal` or manual edits from marking roadmap items completed unless the row references an existing completed closeout or finalized batch evidence path and includes that path in `write_scopes`.

## Drift Findings

- The active asset pipeline design correctly assigned semantic asset truth to `domain/asset`; the implementation stayed inside that owner and did not move app IO, external importer execution, editor UI, renderer resources, or SDF/world payload semantics into the asset domain.
- `apps/runenwerk_editor/src/asset_pipeline/` already had adapter-local preservation behavior. WR-020 now gives that behavior a reusable domain contract without changing editor adapter orchestration.
- `ProjectFileV2` remains in `domain/editor/editor_persistence` because editor project persistence is editor-specific; `domain/asset` now owns the reusable project catalog descriptor that persistence/adapters can consume later.
- `WR-026` remains downstream. No files under `apps/runenwerk_editor/src/asset_pipeline` or editor provider surfaces were changed for this closeout.

## Validation

Required WR-020 validation completed on 2026-05-16:

- `cargo test -p asset` passed: 18 tests.
- `task docs:validate` passed.

Additional integration checks completed:

- `cargo fmt --all -- --check` passed.
- `cargo check -p runenwerk_editor` passed.
- `cargo test -p runenwerk_editor asset` passed: existing app-side asset pipeline consumers still compile and pass without adding WR-026 adapter work.
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task puml:validate` passed: 20 diagrams.

## Deferred Work

- `WR-026` may later add editor adapter load/save, import orchestration, diagnostics surfacing, and prior-valid artifact UI over these contracts, but it is not part of this work.
- Field Visualizer, Material Lab, and prefab runtime work must consume these domain contracts and their owning designs rather than making editor canvas/session state semantic asset truth.
- External importer execution and host IO stay in app/tool adapters.
