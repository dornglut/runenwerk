---
title: Story Manifest, Registry, Runner, And Report Contract Implementation Plan
status: active
type: implementation-plan
wr: WR-175
milestone: PM-UI-STORY-002
---

# Story Manifest, Registry, Runner, And Report Contract

## Decision

Implement the first `domain/ui/ui_story` crate as the domain-owned story proof contract for `UiStoryManifest`, `UiStoryRegistry`, `UiStoryRunner`, `UiStoryRunReport`, and `UiStoryMountEligibility`.

This slice is intentionally contract-first. It creates the durable public API and fail-closed report semantics that later gallery, CLI, static mount, and product mount work must consume. The production-behavior authority is limited to creating that public story contract crate and workspace registration; it must not migrate gallery execution, perform runtime rendering proof, introduce component maturity, or add product host behavior.

## Exact Owners And Files

- `Cargo.toml`: register `domain/ui/ui_story` as a workspace member and workspace dependency.
- `domain/ui/ui_story/Cargo.toml`: crate manifest for package `ui_story`, edition 2024, with only dependencies required by the public contract.
- `domain/ui/ui_story/src/lib.rs`: public module boundary and focused re-exports for normal story workflows.
- `domain/ui/ui_story/src/manifest.rs`: `UiStoryManifest`, `UiStoryId`, source, category, host, theme, viewport, expected outcome, diagnostic expectation, and mount policy input types.
- `domain/ui/ui_story/src/registry.rs`: deterministic `UiStoryRegistry` storage and lookup over manifests; no renderer, ECS, editor, game, designer, filesystem, or app state ownership.
- `domain/ui/ui_story/src/runner.rs`: `UiStoryRunner` and `UiStoryRunRequest` orchestration over registered manifests that always returns a `UiStoryRunReport`.
- `domain/ui/ui_story/src/report.rs`: `UiStoryRunReport`, `UiStoryStageReport`, `UiStoryDiagnostic`, `UiStoryVerdict`, and stage/status vocabulary.
- `domain/ui/ui_story/src/mount.rs`: `UiStoryMountEligibility::from_report` derived only from `UiStoryRunReport`.
- `docs-site/src/content/docs/domain/ui/roadmap.md`: note that the contract crate now exists and that gallery/runtime proof remains downstream.

## Public Contract Requirements

- `UiStoryManifest` must require stable story identity, source kind/path/id, program id, host profile, theme profile, viewport matrix, expected verdict, and mount policy.
- `UiStoryManifest::validate` must fail closed for missing required source, host, theme, viewport, expected-outcome, or mount-policy data.
- `UiStoryRegistry` must provide deterministic construction from manifests, duplicate-id diagnostics, `get`, `contains`, `stories`, and `run_request` style access without touching app-local gallery state.
- `UiStoryRunner::run_story` must never report success for an unknown story, invalid manifest, missing proof stage, or expected-failure story that unexpectedly passes.
- `UiStoryRunReport` must preserve manifest identity, ordered stage reports, stable diagnostics, first failing stage, expected verdict, and final verdict.
- `UiStoryMountEligibility::from_report` must grant eligibility only for non-failure stories whose report verdict passed and whose required proof stages passed.
- All diagnostics must carry stable codes and stage ownership so gallery and CLI consumers can inspect one report format later.

## Non-Goals

- No gallery migration, CLI command, static mount, product host mount, or runtime rendering behavior in this milestone.
- No controls, interaction platform, text platform, GraphCanvas, Timeline, Visual UI Builder, game HUD, world-space UI, or entity-attached UI implementation.
- No renderer-owned, ECS-owned, editor-owned, app-owned, or game-owned UI semantics.
- No filesystem discovery API yet; PM-UI-STORY-003 owns gallery/CLI discovery and asset loading.
- No fake green runtime proof. Stages not implemented in this slice must be represented as not run or missing proof, never as successful rendering.

## Implementation Steps

1. Add the workspace member/dependency entry in `Cargo.toml`.
2. Add the `ui_story` crate manifest and public module boundary.
3. Implement manifest data types and validation with focused unit coverage for a valid story and missing required inputs.
4. Implement deterministic registry insertion and duplicate-id diagnostics.
5. Implement report/stage/verdict vocabulary and fail-closed aggregation helpers.
6. Implement runner lookup/validation/report construction for registered story manifests.
7. Implement mount eligibility derivation from report verdict and required stage states.
8. Update the UI roadmap to record the new contract crate and downstream milestone boundary.

## Validation

- `cargo test -p ui_story`
- `cargo fmt --all --check`
- `task docs:validate`
- `task production:validate`
- `task roadmap:validate`

## Closeout Requirements

The closeout for PM-UI-STORY-002 must include runtime_test, artifact, diagnostics, and source_maps evidence for the new story contract. It must name any downstream work as out of scope rather than as a gap inside this milestone, especially gallery/CLI execution, runtime rendering proof, reusable component maturity, Designer/Workbench product authoring, game HUD behavior, and world-space UI.

## Stop Conditions

- Stop if root `Cargo.toml` cannot be updated under explicit crate-creation authority.
- Stop if implementation leaves `domain/ui/ui_story` except for root workspace registration and the UI roadmap note.
- Stop if any report API claims renderer, editor, game, or product host success without a story report stage.
- Stop if validation fails or closeout evidence cannot be produced.
