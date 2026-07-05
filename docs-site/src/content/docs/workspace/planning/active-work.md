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
  - ./ecs-backed-counter-ui-story-proof-planning.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../design/active/ui-runtime-rendering-pipeline-roadmap.md
  - ../../design/active/runenwerk-ui-platform-capability-roadmap.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-002`

Title: `ECS-backed Counter UI Story Proof Planning`

State: proof-planning intake selected; long-term owner strategy selected; implementation is not authorized.

Lifecycle state: `active-planning`

Owner: `domain/ui` owns UI framework proof planning and reusable UI contracts. The selected long-term owner shape is a small UI-owned app-integration layer, working name `domain/ui/ui_app_integration`, but exact crate/module creation is not authorized yet. `engine::App` / ECS remains the app/runtime host surface to prove. Workspace planning owns this active-work record.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/workspace/planning/ecs-backed-counter-ui-story-proof-planning.md`, `docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md`, `docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md`, `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`, and `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md`.

Evidence classes: `E8` accepted direction authority from PR #70 plus current planning intake evidence; future implementation needs `E3` source inspection of exact owners and `E5` validation evidence before merge.

Complete investigation gate: not complete for implementation. The planning intake names the proof scenario, selected owner strategy, non-owned responsibilities, and stop conditions, but it does not yet inspect exact implementation files deeply enough to authorize code.

Complete design gate: not complete for implementation. The planning intake selects `C-internal first, then B-public later`: a small UI-owned app-integration proof bridge first, AppUiExt-style public ergonomics only after the proof validates dependency and owner boundaries. Exact internal type names, crate/module owner, dependency direction, module decomposition, and validation commands remain undecided.

Implementation contract: none active. Implementation must not start until a revised planning contract turns the selected strategy into exact files, internal API names, dependency rules, tests, validation commands, and closeout evidence.

Allowed files/crates: docs/planning/design files only in the current intake. No product code, no crate creation, no engine runtime changes, no UI runtime implementation changes, no SpatialCanvas implementation, no `domain/app_program` implementation work.

Non-owned files/crates: all product/editor/game behavior, engine scheduler/runtime/physics/asset-loading/streaming/LOD/render-resource/world-mutation behavior, networking/multiplayer/threading implementations, `foundation/meta`, command execution in `foundation/commands`, generic plugin framework/AppRecipe/PluginSuite behavior, Phase 17 SpatialCanvas implementation files, raw ECS-owned UI semantic models, `ui_definition` callback behavior, `ui_controls` app mutation behavior, renderer backend resources, and implementation shortcuts that bypass `ui_definition`, `UiProgram`, or `UiStory` proof.

Principle compliance matrix: recorded in `ecs-backed-counter-ui-story-proof-planning.md`. The planning direction favors the smallest proof that demonstrates the real framework loop, avoids duplicating UI semantics in ECS or app_program, defers shared plugin/framework extraction, preserves owner separation, and requires module decomposition before implementation.

Module decomposition map: deferred until a later implementation-planning revision. Candidate owners are recorded, and the selected preferred owner shape is `domain/ui/ui_app_integration`, but exact files are not accepted yet.

Maintainability review status: planning intake selected. Maintainability must be reviewed before implementation against exact owner files, public API names, crate boundaries, proof reports, validation envelope, and no-bypass evidence.

Feature support matrix: the first proof must cover Counter app state, UI screen/source declaration, typed UI action lowering, route/event emission, host/app action resolution, app-owned mutation, next UI output, story report, negative route/capability/disabled/missing-data cases, and no callback/direct-mutation bypass.

Future-use-case pressure matrix: covered by UI Story, UI Runtime Rendering Pipeline, UI Component Platform, Interaction V2, SpatialCanvas intake, and the superseded Typed App Program pressure archive. The first proof must consume these as boundaries, not implement designer, game HUD, world-space UI, SpatialCanvas, shared plugin framework, or external-template-only workflows.

Hierarchy/composition matrix: target proof hierarchy is `App/ECS host state -> UI screen/component source -> ui_definition validation/normalization -> FormedInteractionModel -> UiProgram -> runtime artifact/output -> UiEventPacket/route proposal -> typed app action -> app/ECS mutation -> next UI output -> UiStoryRunReport`.

Ergonomics/usability: the target framework should eventually let app authors register resources, systems, screens, screen routers, and typed UI actions through an App/Plugin extension surface while preserving lowered UI source, source maps, diagnostics, and story proof. This planning slice explicitly defers public AppUiExt-style names until after the internal bridge proof validates owner and dependency direction.

Validation expectation: this docs-only planning change should pass `python tools/docs/validate_docs.py` and `git diff --check`. No code validation is claimed from this branch.

Known blockers: implementation is blocked until exact implementation owner, internal API names, dependency rules, validation commands, and closeout evidence are accepted.

Next action: revise `ecs-backed-counter-ui-story-proof-planning.md` into an implementation contract for the selected strategy: C-internal first, small UI-owned app-integration proof bridge plus Counter UiStory proof, with public AppUiExt-style API deferred.

Evidence: this active-work update follows merged PR #70 direction authority and keeps the next step planning-only while selecting the long-term owner strategy.

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
