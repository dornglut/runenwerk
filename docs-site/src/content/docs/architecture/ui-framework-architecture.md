---
title: Runenwerk UI Local Runtime and Integration Architecture
description: Canonical Runenwerk-local UI ownership and integration architecture, separating current Runenwerk implementation from standalone RunenUI reusable-framework authority.
status: active
owner: ui
layer: architecture
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../domain/ui/README.md
  - ../domain/ui/architecture.md
  - ../domain/ui/roadmap.md
  - ./live-uiplugin-runtime-platform-architecture.md
  - ../design/deferred/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../design/implemented/ui-program-architecture.md
  - ../design/implemented/ui-program-architecture-owner-map.md
  - ../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../design/deferred/game-runtime-ui-projection-and-hud-platform-design.md
  - ../design/deferred/ui-model-multiple-execution-strategies-design.md
  - domain-authoring-platform-overview.md
  - ../adr/accepted/0009-ui-interaction-formation-v2.md
---

# Runenwerk UI Local Runtime and Integration Architecture

## Purpose

This is the canonical top-down architecture spine for the UI implementation and
integration that currently lives inside Runenwerk.

It answers a deliberately local question: how do Runenwerk-owned UI definition,
interaction, semantic-program, retained-runtime, story-proof, engine-integration,
and renderer-facing contracts fit together today?

It is **not** the reusable UI-framework architecture for the Dornglut repository
family. Future reusable framework semantics belong to standalone
[`dornglut/runen-ui`](https://github.com/dornglut/runen-ui). Current Runenwerk
UI code remains valid until an explicit consumer-cutover issue changes a named
boundary.

## Authority boundary

Use the authority that owns the question:

1. current Runenwerk code and executable tests own current local behavior;
2. accepted Runenwerk ADRs and implemented/accepted local designs own durable
   Runenwerk decisions;
3. this file owns the top-down **Runenwerk-local** UI integration model;
4. [UI Domain Current-State Architecture](../domain/ui/architecture.md) owns the
   detailed local crate and migration map;
5. [UI Substrate and Surface Roadmap](../domain/ui/roadmap.md) owns durable local
   sequencing only;
6. standalone RunenUI [architecture](https://github.com/dornglut/runen-ui/blob/main/ARCHITECTURE.md),
   [status](https://github.com/dornglut/runen-ui/blob/main/docs/status.md), and
   [roadmap](https://github.com/dornglut/runen-ui/blob/main/docs/roadmap.md) own
   future reusable-framework semantics and maturity.

Historical Runenwerk plans and reports remain provenance, not reusable-framework
authority.

## Current local architecture

The current Runenwerk-local UI spine is:

```text
Authored UI / product definitions
  -> ui_definition validation and normalization
  -> FormedInteractionModel
  -> local execution/program paths
       retained ui_tree / ui_widgets / ui_runtime
       UiProgram -> ui_program_lowering -> ui_compiler -> ui_artifacts
                 -> ui_evaluator / ui_runtime_view
  -> UiStory V2 workflow proof where applicable
  -> engine/app host integration
  -> backend-neutral UiFrame / render data
  -> renderer or other product consumer
```

The retained path and the UiProgram path coexist. Neither is declared fully
replaced by the other. Foundation crates such as `ui_math` and `ui_layout` and
retained crates such as `ui_tree`, `ui_widgets`, and `ui_runtime` remain valid
owners alongside the implemented UiProgram family.

## Local owner map

| Responsibility | Current Runenwerk owner |
|---|---|
| UI math and layout vocabulary/algorithms | `ui_math`, `ui_layout` |
| Input, text, and theme contracts | `ui_input`, `ui_text`, `ui_theme` |
| Authored UI definitions and normalization | `ui_definition` |
| Interaction formation | `ui_definition` plus ADR 0009 contracts |
| Retained tree/runtime/widgets | `ui_tree`, `ui_runtime`, `ui_widgets` |
| Semantic UI program | `ui_program` |
| Program lowering | `ui_program_lowering` |
| Control packages and control-facing requirements | `ui_controls` |
| Compiler and runtime artifacts | `ui_compiler`, `ui_artifacts` |
| Artifact-backed evaluation/read models | `ui_evaluator`, `ui_runtime_view` |
| Binding and host-contract vocabulary | `ui_binding`, `ui_hosts` |
| Accessibility contracts | `ui_accessibility` |
| Story/proof orchestration | `ui_story`, `ui_testing` |
| Renderer-neutral output | `ui_render_data`, `ui_render_primitives`, `ui_static_mount` |
| Runenwerk app-facing mount/action integration | `engine::plugins::ui` |
| Historical/proof app bridge | `ui_app_integration` |
| Editor mutation and product effects | editor/app owners |
| Renderer execution | engine renderer owners |

The detailed crate-family partition is enforced by
`domain/ui/ui-crate-ownership.toml`; this page does not redefine that machine
contract.

## Definition and interaction formation

`ui_definition` owns Runenwerk's local authored UI source/IR, validation,
normalization, source maps, and retained formation inputs. Authored identity is
not retained `WidgetId`, ECS entity identity, renderer identity, or app mutation
authority.

ADR 0009 owns execution-neutral interaction formation. Its durable local spine is:

```text
NormalizedUiTemplate
  -> FormedInteractionModel
  -> local execution consumers
```

Popup, scroll, focus, menu sizing, docking/drop-zone, chrome-slot, status, and
input-arbitration facts remain explicit contracts before a retained or other
accepted execution consumer handles them.

## UiProgram and artifact path

The implemented UiProgram architecture is Runenwerk-local current truth. It owns
or coordinates typed control, layout, state, style, interaction, binding,
visual, accessibility, and inspection graph families together with route,
schema, capability, source-map, and diagnostic identities.

The current artifact path is:

```text
UiProgram
  -> UiCompiler
  -> UiRuntimeArtifact
  -> UiEvaluator / UiRuntimeView
```

Artifact tables and manifests are derived executable/read-model products. They
do not make `UiProgram` an authored source tree, retained widget tree, renderer
frame, ECS model, or standalone RunenUI contract.

## Retained runtime coexistence

Retained UI remains an implemented Runenwerk execution path. `ui_tree`,
`ui_widgets`, and `ui_runtime` continue to own retained identity, interaction,
layout orchestration, and frame generation where current product code consumes
them.

The existence of UiProgram/compiler/evaluator proofs does not authorize deleting
or bypassing retained owners. Any named migration requires its own accepted
consumer cutover and proof.

## Story V2 proof boundary

Runenwerk's current `ui_story` implementation uses the V2 workflow graph model,
not the former flat `UiStoryRunReport` model.

A V2 story manifest selects a workflow profile. Built-in profiles include:

```text
ui_story.workflow.source_load_only
ui_story.workflow.compiler_only
ui_story.workflow.static_preview
ui_story.workflow.executable_interaction_proof
```

Workflow nodes carry owner-produced evidence. `UiStoryWorkflowReportV2` records
the graph, node outcomes, diagnostics, expected-failure matching, and first
blocker. `UiStoryMountDecisionV2` derives a fail-closed mount decision; a passed
preview alone is not permission to mutate product state or bypass host policy.

The local story workflow is proof and product-consumer infrastructure. It is not
a second reusable-framework authority and does not define future RunenUI testing,
controls, platform, renderer, or authoring semantics.

## Host integration and mutation ownership

Hosts own mutation and effects. Generic Runenwerk UI controls emit facts,
proposals, or typed events; they do not directly mutate app, editor, game,
renderer, network, filesystem, or provider truth.

The accepted Runenwerk tree contains the local engine integration surface:

- `engine::plugins::ui::UiPlugin`;
- `AppUiExt::mount_ui(...)` and the `app.ui().mount(...)` facade;
- typed `UiScreen`/source/action contracts;
- typed `UiActionHandler` dispatch;
- mounted runtime/session resources;
- producer-generic surface-frame publication.

Those are Runenwerk implementation facts. They do **not** prove standalone
RunenUI adoption.

## Renderer and product consumers

Renderer-facing products remain derived data. `UiFrame` and related local render
contracts may be consumed by the engine renderer, headless/static proof, or a
separately accepted product target. Renderer resources and submission do not own
UI source, route/action semantics, control meaning, or product mutation.

Game HUD, SDF, and world-space UI are not promoted here into reusable framework
semantics. Their exact Runenwerk product integration remains separately owned or
deferred until a concrete consumer issue accepts it.

## Standalone RunenUI adoption boundary

Standalone RunenUI is the sole Dornglut owner for future reusable UI-framework
semantics. Runenwerk does not mirror its roadmap locally.

A future Runenwerk adoption or partial cutover must:

1. start from a new owning issue;
2. inspect the exact then-accepted RunenUI revision and current Runenwerk main;
3. name the concrete consumer boundary being replaced;
4. prove migration of current Runenwerk behavior and tests;
5. preserve product/engine ownership of host mutation and renderer execution;
6. remove replaced local authority only after the consumer cutover is accepted.

The deferred [Live UiPlugin consumer-integration design](../design/deferred/live-uiplugin-runtime-and-surface-frame-rendering-design.md)
records historical Runenwerk integration context; it is not activation authority.

## Non-goals

This architecture does not authorize:

- standalone RunenUI adoption or dependency changes;
- wholesale replacement of retained Runenwerk UI;
- a second reusable UI framework inside Runenwerk;
- future generic controls, animation, virtualization, platform, renderer,
  accessibility, or authoring roadmaps in Runenwerk;
- compiled-reactive or ECS-driven UI execution without separate accepted
  authority;
- SDF-, renderer-, or ECS-owned UI semantics;
- app/editor/game mutation from generic UI controls;
- `foundation/meta` or shared plugin-framework extraction.

## Diagram

Diagram source:
[ui-framework-architecture.puml](./diagrams/ui-framework-architecture.puml).

## Related docs

- [UI Domain](../domain/ui/README.md)
- [UI Domain Current-State Architecture](../domain/ui/architecture.md)
- [UI Substrate and Surface Roadmap](../domain/ui/roadmap.md)
- [Live UiPlugin Runtime Platform Architecture](./live-uiplugin-runtime-platform-architecture.md)
- [Deferred Live UiPlugin Consumer Integration](../design/deferred/live-uiplugin-runtime-and-surface-frame-rendering-design.md)
- [UI Program Architecture](../design/implemented/ui-program-architecture.md)
- [UI Program Architecture Owner Map](../design/implemented/ui-program-architecture-owner-map.md)
- [Runenwerk UI Story V2 Consumer and Proof Boundary](../design/active/runenwerk-ui-story-driven-golden-workflow-design.md)
- [ADR 0009: UI Interaction Formation V2](../adr/accepted/0009-ui-interaction-formation-v2.md)
- [Standalone RunenUI Architecture](https://github.com/dornglut/runen-ui/blob/main/ARCHITECTURE.md)
