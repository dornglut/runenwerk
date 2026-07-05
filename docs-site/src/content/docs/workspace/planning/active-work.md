---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-05
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ./typed-app-program-ui-proof-001-planning.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../reports/investigations/typed-app-program-current-state-investigation.md
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../design/active/ui-runtime-rendering-pipeline-roadmap.md
  - ../../design/active/runenwerk-ui-platform-capability-roadmap.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-001`

Title: `UI Framework App Integration Direction Review`

State: direction-correction planning selected; implementation is not authorized.

Lifecycle state: `active-planning`

Owner: `domain/ui` owns the UI framework direction and reusable UI contracts. `engine::App` / ECS remains the app/runtime host surface for future integration design. Workspace planning owns this active-work record. `domain/app_program` is no longer the active implementation foundation unless a later accepted planning contract explicitly reactivates it as proof/report vocabulary only.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md`, `docs-site/src/content/docs/domain/ui/roadmap.md`, `docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md`, `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`, `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md`, `docs-site/src/content/docs/adr/accepted/0009-ui-interaction-formation-v2.md`, and the prior Typed App Program investigation/design files as superseded pressure evidence.

Evidence classes: `E3` current source and planning inspection from UI domain docs, `E8` accepted design/roadmap/ADR authority, and future `E5` validation evidence required before any implementation merge.

Complete investigation gate: not complete for implementation. The new direction-review document records the current evidence base and the questions to answer before implementation. A follow-up implementation plan must still name exact files, crates, validation commands, evidence expectations, and stop conditions.

Complete design gate: not complete for implementation. The selected design direction is ECS/App/Plugin-hosted app authoring plus `ui_definition`-backed UI source, `UiProgram` semantic contracts, `UiStory` proof/mount eligibility, and host/app-owned mutation. Exact public API names and owner files remain undecided.

Implementation contract: none active. The previous `Typed App Program UI Proof 001 — Headless Counter App Proof` implementation direction is superseded as the next foundation. PR #69 must not be merged as the framework foundation unless this direction decision is explicitly reverted.

Allowed files/crates: docs/planning/design files only in the current direction-review work. No product code, no crate creation, no engine runtime changes, no UI runtime implementation changes, no SpatialCanvas implementation, no `domain/app_program` implementation work.

Non-owned files/crates: all product/editor/game behavior, all engine scheduler/runtime/physics/asset-loading/streaming/LOD/render-resource/world-mutation behavior, networking/multiplayer/threading implementations, `foundation/meta`, command execution in `foundation/commands`, generic plugin framework/AppRecipe/PluginSuite behavior, Phase 17 SpatialCanvas implementation files, raw ECS-owned UI semantic models, `ui_definition` callback behavior, `ui_controls` app mutation behavior, renderer backend resources, and production dependency shortcuts that bypass `ui_definition`, `UiProgram`, or `UiStory` proof.

Principle compliance matrix: recorded in `ui-framework-app-integration-direction-review.md`. The direction favors the simplest real framework proof, avoids duplicating UI semantics in ECS or `app_program`, defers speculative plugin/framework extraction, preserves owner separation, and requires module decomposition before implementation.

Module decomposition map: deferred until a later implementation-planning contract. The direction review names candidate responsibilities only: UI app-integration bridge, `ui_definition` source records, `UiProgram` route/event facts, `UiStory` proof/report envelope, and App/ECS host mutation proof.

Maintainability review status: planning selected. Maintainability must be reviewed before implementation against exact owner files, public API names, crate boundaries, proof reports, validation envelope, and no-bypass evidence.

Feature support matrix: the future first proof must cover app state, UI screen/source declaration, typed UI action lowering, route/event emission, host/app action resolution, app-owned mutation, next UI output, story report, negative route/capability/disabled/missing-data cases, and no callback/direct-mutation bypass.

Future-use-case pressure matrix: covered by UI Story, UI Runtime Rendering Pipeline, UI Component Platform, Interaction V2, SpatialCanvas intake, and Typed App Program pressure docs. The next proof must consume these as boundaries, not implement designer, game HUD, world-space UI, SpatialCanvas, shared plugin framework, or external-template-only workflows.

Hierarchy/composition matrix: selected direction is `App/ECS host state -> UI screen/component source -> ui_definition validation/normalization -> FormedInteractionModel -> UiProgram -> runtime artifact/output -> UiEventPacket/route proposal -> typed app action -> app/ECS mutation -> next UI output -> UiStoryRunReport`. `app_program` proof IR may be reconsidered later as report vocabulary, not as the active app framework surface.

Ergonomics/usability: the target framework should eventually let app authors register resources, systems, screens, screen routers, and typed UI actions through an App/Plugin extension surface while preserving lowered UI source, source maps, diagnostics, and story proof. The user-facing path must not require manual `AppModelSnapshot`, `RouteActionMap`, `decode_action`, or direct `UiProgram` graph-row construction for a simple counter.

Validation expectation: this docs-only planning change must pass `python tools/docs/validate_docs.py` and `git diff --check`. No code validation is claimed from this branch.

Known blockers: implementation is blocked until the direction review is accepted and a new implementation-planning contract is written for the first ECS-backed Counter UI Story Proof.

Next action: review and accept, revise, or reject `ui-framework-app-integration-direction-review.md`. If accepted, close or keep PR #69 only as a superseded spike, then open a separate planning contract for `ECS-backed Counter UI Story Proof`.

Evidence: this active-work update is based on current UI architecture/roadmap/story/runtime/component-platform authority and deliberately resets the active focus from manual `app_program` implementation to framework-direction planning.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Principle compliance matrix:
Module decomposition map:
Maintainability review status:
Feature support matrix:
Future-use-case pressure matrix:
Hierarchy/composition matrix:
Ergonomics/usability:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
