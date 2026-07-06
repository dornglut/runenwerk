---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/ui-framework-architecture.md
  - ./ecs-backed-counter-ui-story-proof-planning.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../../design/active/ui-runtime-rendering-pipeline-roadmap.md
  - ../../design/active/runenwerk-ui-platform-capability-roadmap.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

Architecture reference: the canonical top-down architecture is
`docs-site/src/content/docs/architecture/ui-framework-architecture.md`; the
current post-merge focus is PR #72 closeout and planning truth for the
ECS-backed Counter UI Story Proof. This closeout must not expand into compiler
DSLs, SDF UI, SpatialCanvas, or public AppUiExt.

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-002`

Title: `ECS-backed Counter UI Story Proof Post-Merge Truth`

State: PR #72 is merged into `main`; the implementation proof exists in
`domain/ui/ui_app_integration`; post-merge closeout and planning truth are
pending before the next app-framework implementation focus.

Lifecycle state: `review` pending closeout truth.

Owner: `domain/ui` owns UI framework proof planning and reusable UI contracts.
The delivered proof owner is the small UI-owned app-integration crate,
`domain/ui/ui_app_integration`. `engine::App` / ECS remains the long-term
app/runtime host surface, but PR #72 proves ECS-backed state directly and still
defers public engine::App extension methods.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md`, `docs-site/src/content/docs/workspace/complete-investigation-gate.md`, `docs-site/src/content/docs/workspace/complete-design-gate.md`, `docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md`, `docs-site/src/content/docs/workspace/planning/ecs-backed-counter-ui-story-proof-planning.md`, `docs-site/src/content/docs/design/active/ui-framework-app-integration-direction-review.md`, `docs-site/src/content/docs/design/active/runenwerk-ui-story-driven-golden-workflow-design.md`, `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`, `docs-site/src/content/docs/design/active/runenwerk-ui-platform-capability-roadmap.md`, and PR #72 merge evidence on `origin/main`.

Evidence classes: `E8` accepted direction authority from PR #70 and planning
authority from PR #71, `E3` source/code inspection of the delivered
`ui_app_integration` owner surface, and `E5` local git evidence that
`origin/main` contains PR #72 at `e093eb1a`.

Complete investigation gate: complete for the first implementation contract and
now awaiting post-merge truth. Closeout must confirm the delivered PR #72 scope,
validation, no-bypass evidence, and remaining gaps against
`ecs-backed-counter-ui-story-proof-planning.md`.

Complete design gate: complete for the first implementation contract, not for
public AppUiExt ergonomics. The delivered shape remains `C-internal first, then
B-public later`: a small UI-owned `ui_app_integration` crate, internal/proof-local
bridge APIs, Counter UI story loop proof, and public engine::App extension
methods deferred to a later slice.

Implementation contract: implemented by PR #72. The next required work is
closeout/post-merge truth, not opening the implementation branch. Use
`ecs-backed-counter-ui-story-proof-planning.md` as the promised contract and
PR #72 as delivered-contract evidence.

Delivered files/crates to verify in closeout: `Cargo.toml`;
`domain/ui/ui_app_integration/Cargo.toml`;
`domain/ui/ui_app_integration/src/{lib.rs,ids.rs,action.rs,screen.rs,source.rs,bridge.rs,host.rs,report.rs,proof.rs}`;
`domain/ui/ui_app_integration/tests/{counter_ui_story_proof.rs,counter_ui_story_fail_closed.rs}`;
and any narrow compile-wiring edits made by PR #72. This active-work state does
not authorize new implementation files.

Non-owned files/crates: all product/editor/game behavior, engine scheduler/runtime/physics/asset-loading/streaming/LOD/render-resource/world-mutation behavior, engine prelude/App extension API, networking/multiplayer/threading implementations, `foundation/meta`, command execution in `foundation/commands`, generic plugin framework/AppRecipe/PluginSuite behavior, Phase 17 SpatialCanvas implementation files, raw ECS-owned UI semantic models, `ui_definition` callback behavior, `ui_controls` app mutation behavior, renderer backend resources, `domain/app_program` resurrection, and implementation shortcuts that bypass `ui_definition`, `UiProgram`, or story-compatible proof reports.

Principle compliance matrix: recorded in `ecs-backed-counter-ui-story-proof-planning.md`; closeout must confirm PR #72 preserved the same matrix. The proof bridge should remain small, avoid duplicating UI semantics in ECS/app_program/engine, defer shared plugin/framework extraction, preserve owner separation, and keep the new crate decomposed into IDs/actions/screens/source/bridge/host/report/proof modules.

Module decomposition map: specified in `ecs-backed-counter-ui-story-proof-planning.md` with exact module responsibilities for `ids`, `action`, `screen`, `source`, `bridge`, `host`, `report`, and `proof`; closeout must compare PR #72 against that map.

Maintainability review status: pending post-merge closeout. Review PR #72
against the exact file list, dependency rules, no-bypass evidence, report
shape, and validation envelope before marking the proof completed.

Feature support matrix: closeout must confirm PR #72 covers Counter app state,
UI screen/source declaration, typed UI action lowering, route/event emission,
host/app action resolution, app-owned mutation, next UI output,
story-compatible report, negative route/capability/disabled/missing-data cases,
no callback/direct-mutation bypass, and no public AppUiExt API.

Future-use-case pressure matrix: covered by UI Story, UI Runtime Rendering Pipeline, UI Component Platform, Interaction V2, SpatialCanvas intake, and the superseded Typed App Program pressure archive. The first proof consumes these as boundaries and must not implement designer, game HUD, world-space UI, SpatialCanvas, shared plugin framework, external-template-only workflow, or public AppUiExt ergonomics.

Hierarchy/composition matrix: target proof hierarchy is `ECS host state -> UI source helper -> ui_definition node/template records -> ui_program_lowering -> UiProgram route/event facts -> route proposal via UiEventPacket -> ui_app_integration bridge -> typed app action -> ECS-backed Counter mutation -> next UI output facts -> UiAppIntegrationReport`.

Ergonomics/usability: the target framework should eventually let app authors register resources, systems, screens, screen routers, and typed UI actions through an App/Plugin extension surface. PR #72 intentionally defers those public names. The proof exists to ground future ergonomics rather than freeze them prematurely.

Validation expectation: closeout must record the PR #72 validation results from
the implementation branch, including the focused `ui_app_integration` commands,
focused UI crate tests, workspace tests, docs validation, and diff check named
in `ecs-backed-counter-ui-story-proof-planning.md`. Docs-only follow-up changes
should pass `python tools/docs/validate_docs.py` and `git diff --check`.

Known blockers: no implementation-opening blocker remains for
`PT-UI-FRAMEWORK-APP-INTEGRATION-002`; PR #72 is merged. Public AppUiExt
ergonomics, authoring frontend expansion, SDF/game/world-space target
positioning, and execution-strategy work remain blocked until closeout records
the PR #72 truth and a later design slice accepts exact public API names,
dependency direction, and activation conditions.

Next action: run and record PR #72 closeout/post-merge truth, then update
completed work, roadmap, production-track, and decision-register state as
needed before opening any next implementation focus.

Evidence: this active-work update follows merged PR #70 direction authority,
PR #71 implementation-planning authority, and local fetch/git inspection showing
PR #72 merged on `origin/main` as `e093eb1a`.

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
