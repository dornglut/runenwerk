---
title: Runenwerk UI Local Runtime and Integration Architecture
description: Canonical Runenwerk-local UI ownership and RunenUI adoption architecture, separating retained product/authoring/composition semantics from predecessor reusable-runtime authority.
status: active
owner: ui
layer: architecture
canonical: true
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_docs:
  - ../domain/ui/README.md
  - ../domain/ui/architecture.md
  - ../domain/ui/roadmap.md
  - ./live-uiplugin-runtime-platform-architecture.md
  - ../design/implemented/ui-program-architecture.md
  - ../design/implemented/ui-program-architecture-owner-map.md
  - ../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ../adr/accepted/0009-ui-interaction-formation-v2.md
  - ../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# Runenwerk UI Local Runtime and Integration Architecture

## Purpose

This is the canonical top-down architecture for UI semantics that currently live
in Runenwerk and for their eventual consumer adoption of standalone RunenUI.

It distinguishes three facts that must not be collapsed:

1. **current implementation authority** — local code remains authoritative for a
   consumer until an accepted cutover replaces it;
2. **retained Runenwerk semantic authority** — authored/program/product/
   composition semantics that RunenUI does not own;
3. **predecessor reusable-runtime authority** — local framework-shaped semantics
   whose long-term reusable owner is standalone RunenUI.

This document does not authorize a RunenUI dependency or source migration by
itself.

## Cross-repository authority

Standalone `dornglut/runen-ui` owns reusable UI-framework semantics over its
public contracts, including:

- transient typed View/Element authoring;
- keyed reconciliation and persistent mounted-runtime identity;
- framework-local state/lifecycle/invalidation;
- pointer/keyboard/text/IME/focus/interaction routing;
- style, layout, renderer-neutral production text and geometry;
- renderer-neutral paint/hit/semantic publication;
- accessibility semantics and platform projection contracts;
- deterministic headless execution/testing;
- public wgpu and winit edge integrations where accepted.

Runenwerk must not create a second reusable-framework roadmap for those concerns.

RunenUI does **not** implicitly own Runenwerk's UiProgram language/toolchain,
Story V2, editor definition/self-authoring, application structural composition,
workspace persistence, provider/product policy, renderer policy, or app/domain
mutation.

## Current local implementation

Before a consumer cutover, the current Runenwerk path remains:

```text
Authored UI / product definitions
  -> ui_definition validation and normalization
  -> FormedInteractionModel
  -> local execution/program paths
       retained ui_tree / ui_widgets / ui_runtime
       UiProgram -> ui_program_lowering -> ui_compiler -> ui_artifacts
                 -> ui_evaluator / ui_runtime_view
  -> UiStory V2 workflow proof where applicable
  -> engine::plugins::ui
  -> local renderer-neutral frame/publication
  -> renderer or other product consumer
```

This is current code truth, not the desired final reusable-framework boundary.

## Adoption disposition map

The semantic reason for each family determines its future, not its filename.

| Current family | Classification | Long-term decision |
|---|---|---|
| `ui_composition` | **Retained Runenwerk authority** | Keep app-neutral structural composition, transactions, history and persistence in Runenwerk. |
| `ui_adaptive_composition` | **Retained Runenwerk authority** | Keep transient structural projection/proposals subordinate to `ui_composition`. |
| `ui_definition` | **Retained authored authority** | Keep source/normalization semantics; project explicitly into a runtime consumer. |
| `ui_schema` | **Retained authored authority** | Keep Runenwerk schema contracts where used by UiProgram/source tooling. |
| `ui_program` | **Retained authored/program authority** | Keep semantic program meaning; do not make RunenUI interpret UiProgram directly. |
| `ui_program_lowering` | **Retained program/toolchain authority** | Keep source-to-program lowering; runtime-specific lowering must terminate at an explicit adapter. |
| `ui_compiler` | **Retained program/toolchain authority** | Keep deterministic compiler/source-map/diagnostic semantics. |
| `ui_artifacts` | **Retained program/toolchain authority** | Keep derived UiProgram artifacts where independently consumed. |
| `ui_story` | **Retained proof/orchestration authority** | Keep Story V2 workflow/verdict semantics; it may exercise RunenUI as a consumer. |
| `ui_graph_editor` | **Runenwerk product/UI-tool concern** | Keep only product-facing graph-editor semantics not supplied by generic RunenUI controls. |
| `ui_app_integration` | **Proof/integration residue** | Retain only independently used app/proof contracts; otherwise delete during consumer cuts. |
| `ui_math`, `ui_geometry` | **Predecessor framework authority** | Prefer public RunenUI geometry/math for migrated consumers; delete local duplication when unreferenced. |
| `ui_input` | **Predecessor framework authority** | Migrated consumers use RunenUI input/focus contracts; RunenInput-to-RunenUI translation remains an adapter concern. |
| `ui_layout` | **Predecessor framework authority** | Migrated consumers use RunenUI layout semantics; retained authored layout vocabulary must be separated if still source meaning. |
| `ui_text` | **Predecessor framework authority** | Migrated consumers use `runenui_text`; no second shaping/layout authority. |
| `ui_theme` | **Mostly predecessor framework authority** | Runtime style/theme moves to RunenUI; retain only source/product theme vocabulary proven independent of runtime. |
| `ui_tree`, `ui_widgets`, `ui_runtime` | **Predecessor framework authority** | Delete for each migrated consumer; RunenUI mounted runtime becomes the sole runtime authority. |
| `ui_state` | **Predecessor/mixed** | Framework-local widget state moves to RunenUI; app/domain state remains with its Runenwerk owner. |
| `ui_accessibility` | **Predecessor/mixed** | Reusable semantics/platform publication move to RunenUI; product-specific semantic meaning remains with the product adapter. |
| `ui_testing` | **Predecessor/mixed proof support** | Framework-runtime testing moves to `runenui_testing`; Story/product assertions remain Runenwerk-owned. |
| `ui_controls` | **Mixed** | Separate authored/control-package requirements from local runtime/widget implementation; only the former may remain. |
| `ui_binding` | **Mixed** | Keep Runenwerk source/program binding semantics only; mounted/runtime binding execution belongs to RunenUI adapter/runtime. |
| `ui_hosts` | **Predecessor/mixed** | Narrow any retained contract to UiProgram-specific lifecycle/event/output semantics; delete generic host/runtime authority and rename only if a real retained package remains. |
| `ui_surface` | **Temporary predecessor compatibility boundary** | Map every responsibility to RunenUI or the actual Runenwerk app/domain/product owner, then delete the package. No new authority. |
| `ui_evaluator`, `ui_runtime_view` | **Mixed execution/tooling** | Keep deterministic UiProgram evaluation/read-model semantics only where independently required; remove any second mounted/runtime execution role. |
| `ui_render_data`, `ui_render_primitives`, `ui_static_mount` | **Mixed output/integration** | Framework-generic publication moves to RunenUI; Runenwerk Render adapter/product payloads remain only when they express Runenwerk integration. |
| `ui_headless_render`, `ui_headless_render_data` | **Predecessor/proof execution** | Migrate framework execution to ordinary RunenUI headless/testing contracts; retain only Story/product-specific evidence envelopes if needed. |
| `engine::plugins::ui` | **Current owner; future adapter** | Stop owning mounted/runtime semantics for migrated consumers; install/project/dispatch between Runenwerk owners and RunenUI. |

The detailed current dependency allow-list remains in
`domain/ui/ui-crate-ownership.toml`. It describes current source legality; it
does not override this adoption disposition.

## Retained authored/program boundary

Runenwerk authored UI remains execution-neutral at the repository boundary.

The future runtime path is:

```text
ui_definition / UiProgram / product facts
    -> explicit Runenwerk-owned projection
        -> public RunenUI View/Element/action/semantic contracts
            -> runenui_runtime
```

The projection may resolve Runenwerk source maps, authored route IDs, product
bindings and command descriptors, but it must not:

- manufacture a second mounted identity;
- expose private RunenUI runtime internals;
- make RunenUI interpret UiProgram bytecode/tables as a new language authority;
- move app/domain mutation into RunenUI;
- persist RunenUI mounted identity into authored source or `ui_composition`.

A missing or ambiguous projection is a migration blocker, not a reason to keep
two runtimes live for one consumer.

## Structural composition boundary

`ui_composition` describes application structure: targets, roots, regions,
mounted-content references, structural transactions/history and persistence.

RunenUI visual/runtime composition describes a framework runtime's mounted
presentation and publication.

These are different authorities.

```text
ui_composition snapshot
    -> Runenwerk app/editor content resolution
        -> RunenUI view/runtime projection for a concrete target
```

RunenUI must not become a second saved application-layout authority.

## `ui_surface` and `ui_hosts`

ADR 0013 remains authoritative:

- `ui_surface` is temporary predecessor compatibility debt and must be deleted
  after exact responsibility mapping;
- generic `ui_hosts` authority is not an end state;
- only independently useful UiProgram-specific host contracts may remain under
  an owner-accurate boundary;
- current code may continue using these packages until a named consumer cut is
  accepted, but new generic authority must not be added.

This resolves the prior documentation contradiction where accepted ADR 0013
required supersession while current architecture described the packages as if
they were durable long-term owners.

## Engine/App integration boundary

`engine::plugins::ui` currently owns real mounted/session/runtime resources in
the local path. That is current implementation fact.

For a migrated consumer its role narrows to:

- install/configure RunenUI integration in Runenwerk App composition;
- project Runenwerk-authored/product facts into public RunenUI input;
- translate RunenInput/Host ingress into the accepted RunenUI edge;
- consume RunenUI actions/semantic output and route them to Runenwerk app/domain
  owners;
- publish renderer-facing products through an explicit Runenwerk Render adapter
  when that route is selected;
- expose integration diagnostics without becoming the semantic owner of layout,
  text, mounted state, focus, interaction or accessibility.

No generic Host trait, service locator, backend registry or RunenApp extraction
is implied.

## Native host and renderer boundary

Where adopted:

```text
Native Host
    -> runenui_winit translation
        -> RunenUI runtime

RunenUI publication
    -> accepted runenui_render_wgpu edge
       OR
    -> explicit Runenwerk Render adapter
```

Native Host keeps event-loop/window/presentation policy. Runenwerk Render keeps
renderer/product policy when it is the chosen consumer. A renderer adapter must
not reconstruct UI semantics from private/local runtime state.

## Story V2 boundary

Story V2 remains Runenwerk proof/orchestration infrastructure.

A story may:

- form/compile Runenwerk-owned source/program facts;
- invoke a public RunenUI-backed execution adapter;
- collect owner-produced evidence;
- compare expected diagnostics/outcomes;
- derive Runenwerk-local mount eligibility.

Story V2 must not become an alternative mounted runtime or a private RunenUI
conformance suite.

## Consumer sequence

Migration is deliberately sequenced by proof pressure:

1. **Headless authored/program proof** — prove one maintained Runenwerk source or
   UiProgram consumer through public RunenUI runtime/testing contracts. This
   establishes the projection boundary without Editor, Winit or renderer
   migration.
2. **Engine/App mounted integration** — migrate one ordinary app-facing mounted
   consumer so `engine::plugins::ui` becomes an adapter for that consumer.
3. **Draw or another bounded non-Editor product** — prove a real interactive
   product path and renderer/input integration without Editor breadth.
4. **Editor shell/chrome and target-local UI** — migrate only after the adapter,
   multi-target, text/editing, accessibility and interaction contracts are
   already proven.
5. Delete predecessor packages as their final maintained consumers disappear.

This order is not a release roadmap for RunenUI. Each source-bearing cut requires
its own accepted Runenwerk issue and current-source recensus.

## First consumer gate

The first source-bearing adoption issue is not created by this architecture
change.

Before it is decision-complete, its census must identify:

- one maintained headless Story/UiProgram or authored-definition consumer;
- the exact projection into ordinary public RunenUI View/Element/action/semantic
  contracts;
- the exact local evaluator/runtime-view/retained-runtime path it replaces for
  that consumer;
- the source maps and diagnostics that remain Runenwerk-owned;
- the proof showing no dual runtime or semantic reinterpretation.

If those facts cannot be named from current source, adoption remains blocked
rather than adding a speculative adapter framework.

## Clean-cutover rules

Every consumer cut must:

1. re-resolve exact Runenwerk and RunenUI revisions;
2. name the replaced local owner and retained Runenwerk owner;
3. migrate tests/proofs through ordinary public RunenUI contracts;
4. preserve app/domain mutation outside the framework;
5. preserve `ui_composition` structural authority;
6. remove the replaced local runtime path for that consumer in the same cut;
7. leave no forwarding namespace, mirror state or dual input/focus/layout/text
   authority;
8. record diagnostics and failure semantics;
9. pass exact-head repository validation.

## Non-goals

This architecture does not authorize:

- adding the RunenUI dependency;
- bulk source deletion;
- Editor-wide migration;
- moving `ui_composition` into RunenUI;
- moving UiProgram/Story/self-authoring semantics into RunenUI;
- a compatibility facade between local runtime types and RunenUI;
- a new generic host/service/renderer registry;
- RunenApp extraction;
- preserving a local reusable UI framework after its consumers migrate.

## Diagram

Diagram source:
[ui-framework-architecture.puml](./diagrams/ui-framework-architecture.puml).

## Related docs

- [UI Domain](../domain/ui/README.md)
- [UI Domain Current-State Architecture](../domain/ui/architecture.md)
- [UI Substrate and Surface Roadmap](../domain/ui/roadmap.md)
- [ADR 0013](../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md)
- [Live UiPlugin Runtime Platform Architecture](./live-uiplugin-runtime-platform-architecture.md)
- [UI Program Architecture](https://github.com/dornglut/runenwerk/blob/main/docs-site/src/content/docs/design/implemented/ui-program-architecture.md)
- [UI Program Architecture](https://github.com/dornglut/runenwerk/blob/main/docs-site/src/content/docs/design/implemented/ui-program-architecture.md)
- [Runenwerk UI Story V2 Consumer and Proof Boundary](../design/active/runenwerk-ui-story-driven-golden-workflow-design.md)
- [Standalone RunenUI Architecture](https://github.com/dornglut/runen-ui/blob/main/ARCHITECTURE.md)
