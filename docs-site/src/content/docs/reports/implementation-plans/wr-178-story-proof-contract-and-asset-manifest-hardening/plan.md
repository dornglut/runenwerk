---
title: WR-178 Story Proof Contract And Asset Manifest Hardening
description: Implementation contract for hardening UI story proof contracts, asset-backed story manifests, exact expected-failure diagnostics, and gallery no-bypass behavior before PM-UI-STORY-005.
status: active
owner: ui
layer: domain/ui ui_story / editor gallery adapter
canonical: false
last_reviewed: 2026-06-17
related_designs:
  - ../../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../../design/active/runenwerk-ui-platform-capability-roadmap.md
  - ../../../design/active/ui-runtime-rendering-pipeline-roadmap.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/track-execution-manifests/pt-ui-story-platform.yaml
---

# WR-178 Story Proof Contract And Asset Manifest Hardening

## Goal

Execute `PM-UI-STORY-HARDEN-001` / `WR-178` as the prerequisite hardening slice
before `PM-UI-STORY-005`.

This work closes story proof-substrate gaps that would make a no-gap audit
dishonest: hardcoded story metadata, loose expected-failure semantics, and any
gallery/static-mount path that could publish preview evidence without a passing
`UiStoryRunReport` plus story-derived `UiStoryMountEligibility`.

## Architecture Governance

Kickoff command:

```text
task ai:architecture-governance -- --task "PT-UI-STORY-PLATFORM story proof contract and asset manifest hardening prerequisite before PM-UI-STORY-005" --scope "domain/ui/ui_story; assets/ui_gallery/stories; apps/runenwerk_editor UI gallery integration; docs-site/src/content/docs/workspace/production-tracks.yaml; docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-story-platform.yaml; docs-site/src/content/docs/workspace/roadmap-items.yaml"
```

Governance decision:

- DDD bounded context owner: `domain/ui/ui_story` owns manifest schema,
  story proof requirements, run reports, diagnostic expectations, and mount
  eligibility contracts.
- Adapter owner: `apps/runenwerk_editor` may load story assets, render gallery
  previews, and expose CLI inspection, but it consumes `UiStoryRunReport` and
  `UiStoryMountEligibility` instead of owning UI semantic truth.
- Source truth owner: checked-in `.story.ron` files under
  `assets/ui_gallery/stories` own gallery story metadata for this slice.
- ADR decision: no ADR is required for bounded proof/report/manifest hardening.
  Require an ADR or accepted design update before changing durable UI definition
  ownership, dependency direction, renderer semantics, component platform
  ownership, or product authoring contracts.
- Ownership mode: domain UI proof-substrate hardening with editor adapter
  consumption.

## Source Of Truth

- `AGENTS.md`: repository workflow, documentation, validation, and response
  requirements.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PT-UI-STORY-PLATFORM`, `PM-UI-STORY-HARDEN-001`, and PM-005 closeout
  sequencing.
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-story-platform.yaml`:
  legal write scope, stop conditions, validation, and PM-005 closeout-only
  constraint.
- `docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md`:
  story workflow authority and `UiStoryRunReport` proof envelope.
- `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`:
  renderer-neutral runtime proof boundaries.
- `/Users/joshua/Downloads/runenwerk_ui_longterm_roadmap_v3.md`:
  review input only, not repository source of truth.

## Required Changes

- Add `domain/ui/ui_story/src/proof.rs` and export public proof contract types:
  `UiStoryProofProducerId`, `UiStoryProofKey`, `UiStoryProofRequirement`,
  `UiStoryProofContract`, `UiStoryProofEvidence`, `UiStoryProofSubject`, and
  `UiStoryProofDiagnosticExpectation`.
- Add versioned manifest fields for schema, story revision, proof contract
  version, and explicit compatibility/migration policy.
- Keep `UiStoryStageKind` as the compatibility/report view for this slice; do
  not introduce a semantic `UiStoryStageGraph`.
- Add `.story.ron` manifests for `basic`, `selected`, and
  `missing_source.failure` under
  `assets/ui_gallery/stories/controls/button/`.
- Replace hardcoded checked-in story metadata as the source of truth with
  deterministic manifest loading/parsing. Constants may remain only as manifest
  path/source indexes or tests.
- Require expected-failure manifests to declare exact expected diagnostics and
  fail closed for wrong stage/proof key, wrong code, wrong severity, missing
  expected diagnostics, or extra unexpected `Error` diagnostics.
- Preserve the rule that expected-failure stories may have a passing story
  verdict for the declared failure but are never mount eligible.
- Ensure `runenwerk_ui_gallery --inspect-stories` and `UiGalleryResource`
  consume only `UiStoryRunReport` plus `UiStoryMountEligibility` for preview
  eligibility. Static mount success alone must not imply gallery/product mount
  eligibility.

## Non-Goals

- Do not implement `PM-UI-STORY-005` product code. PM-005 remains closeout-only.
- Do not split `ui_definition`, redesign `ui_surface`, create a generic render
  primitive registry, or move physical UI domain folders unless validation
  proves a direct story-proof bypass.
- Do not implement reusable component maturity, Designer/Workbench product
  authoring, game HUD behavior, or world-space/entity-attached UI.
- Do not make renderer, ECS, editor, or game code the owner of authored UI
  semantics.

## Acceptance Criteria

- `.story.ron` manifests round-trip and unsupported future schema versions are
  rejected.
- The missing-source story passes only with the declared source-load diagnostic.
- Wrong stage/proof key, wrong code, wrong severity, missing expected
  diagnostic, and extra unexpected `Error` diagnostics all fail.
- Expected-failure stories never become mount eligible, even when the story
  verdict passes.
- Gallery preview publication refuses failed story reports even if render or
  static-mount data exists.
- Production, roadmap, docs, planning, and `ai:goal` validation agree that
  `PM-UI-STORY-HARDEN-001` is the implementation milestone before PM-005.

## Validation

Run after source edits:

```text
cargo fmt --check
cargo check --workspace
cargo test -p ui_story story_manifest
cargo test -p ui_story expected_failure
cargo test -p ui_story proof_contract
cargo test -p runenwerk_editor story
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
task planning:validate
task ai:goal -- --track PT-UI-STORY-PLATFORM --scope non-deferred
```
