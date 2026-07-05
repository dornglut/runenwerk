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

State: implementation-planning contract drafted; implementation is conditionally authorized only after this docs PR is reviewed/merged and the later code branch stays within the accepted contract.

Lifecycle state: `active-planning`

Owner: `domain/ui` owns UI framework proof planning and reusable UI contracts. The selected implementation owner is a small UI-owned app-integration crate, `domain/ui/ui_app_integration`. `engine::App` / ECS remains the long-term app/runtime host surface, but this first implementation proves ECS-backed state directly and defers public engine::App extension methods.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/workspace/planning/ecs-backed-counter-ui-story-proof-planning.md`, `docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md`, `docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md`, `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`, and `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md`.

Evidence classes: `E8` accepted direction authority from PR #70, `E3` source inspection of current Cargo and crate ownership surfaces, and future `E5` validation evidence before implementation merge.

Complete investigation gate: complete enough for the first implementation contract. Source inspection confirmed current workspace membership, `ui_definition` authored template/node/slot ownership, `ui_program` route/event packet ownership, `ui_program_lowering` formation entrypoint, `ui_controls` registry snapshot, `ui_story` V2 proof surface, `ui_testing` proof/evaluation dependencies, `ecs` public Resource/World exports, and `engine::App` public resource/system APIs with private world state.

Complete design gate: complete for the first implementation contract, not for public AppUiExt ergonomics. The contract selects `C-internal first, then B-public later`: create a small UI-owned `ui_app_integration` crate, use internal/proof-local bridge APIs, prove the Counter UI story loop, and defer public engine::App extension methods to a later slice.

Implementation contract: active after this docs PR is reviewed/merged. The implementation must stay within `ecs-backed-counter-ui-story-proof-planning.md`: exact allowed files, dependency rules, internal type names, tests, validation commands, stop conditions, and closeout evidence are specified there.

Allowed files/crates for the implementation branch: `Cargo.toml`; `domain/ui/ui_app_integration/Cargo.toml`; `domain/ui/ui_app_integration/src/{lib.rs,ids.rs,action.rs,screen.rs,source.rs,bridge.rs,host.rs,report.rs,proof.rs}`; `domain/ui/ui_app_integration/tests/{counter_ui_story_proof.rs,counter_ui_story_fail_closed.rs}`. Only if compile wiring proves necessary: `domain/ui/ui_definition/src/lib.rs`, `domain/ui/ui_program/src/lib.rs`, `domain/ui/ui_hosts/src/lib.rs`, `domain/ui/ui_binding/src/lib.rs`.

Non-owned files/crates: all product/editor/game behavior, engine scheduler/runtime/physics/asset-loading/streaming/LOD/render-resource/world-mutation behavior, engine prelude/App extension API, networking/multiplayer/threading implementations, `foundation/meta`, command execution in `foundation/commands`, generic plugin framework/AppRecipe/PluginSuite behavior, Phase 17 SpatialCanvas implementation files, raw ECS-owned UI semantic models, `ui_definition` callback behavior, `ui_controls` app mutation behavior, renderer backend resources, `domain/app_program` resurrection, and implementation shortcuts that bypass `ui_definition`, `UiProgram`, or story-compatible proof reports.

Principle compliance matrix: recorded in `ecs-backed-counter-ui-story-proof-planning.md`. The contract implements the smallest useful proof bridge, avoids duplicating UI semantics in ECS/app_program/engine, defers shared plugin/framework extraction, preserves owner separation, and decomposes the new crate into IDs/actions/screens/source/bridge/host/report/proof modules.

Module decomposition map: specified in `ecs-backed-counter-ui-story-proof-planning.md` with exact module responsibilities for `ids`, `action`, `screen`, `source`, `bridge`, `host`, `report`, and `proof`.

Maintainability review status: implementation contract drafted. Maintainability must be reviewed in the later code PR against the exact file list, dependency rules, no-bypass evidence, report shape, and validation envelope.

Feature support matrix: the first proof must cover Counter app state, UI screen/source declaration, typed UI action lowering, route/event emission, host/app action resolution, app-owned mutation, next UI output, story-compatible report, negative route/capability/disabled/missing-data cases, no callback/direct-mutation bypass, and no public AppUiExt API.

Future-use-case pressure matrix: covered by UI Story, UI Runtime Rendering Pipeline, UI Component Platform, Interaction V2, SpatialCanvas intake, and the superseded Typed App Program pressure archive. The first proof consumes these as boundaries and must not implement designer, game HUD, world-space UI, SpatialCanvas, shared plugin framework, external-template-only workflow, or public AppUiExt ergonomics.

Hierarchy/composition matrix: target proof hierarchy is `ECS host state -> UI source helper -> ui_definition node/template records -> ui_program_lowering -> UiProgram route/event facts -> route proposal via UiEventPacket -> ui_app_integration bridge -> typed app action -> ECS-backed Counter mutation -> next UI output facts -> UiAppIntegrationReport`.

Ergonomics/usability: the target framework should eventually let app authors register resources, systems, screens, screen routers, and typed UI actions through an App/Plugin extension surface. This implementation explicitly defers those public names. It proves the owner boundary first so future ergonomics are grounded rather than speculative.

Validation expectation: docs-only PR should pass `python tools/docs/validate_docs.py` and `git diff --check`. Later implementation must pass the validation commands listed in `ecs-backed-counter-ui-story-proof-planning.md`, including `cargo test -p ui_app_integration`, the two focused counter tests, focused UI crate tests, workspace tests, docs validation, and diff check.

Known blockers: implementation is blocked until this docs PR is reviewed/merged. Public AppUiExt ergonomics remain blocked until the first proof passes and a new design slice accepts exact public API names and engine dependency direction.

Next action: after this docs PR is accepted, open the implementation branch for `ui_app_integration` using the exact contract in `ecs-backed-counter-ui-story-proof-planning.md`.

Evidence: this active-work update follows merged PR #70 direction authority and the source-inspection pass performed for PR #71.

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
