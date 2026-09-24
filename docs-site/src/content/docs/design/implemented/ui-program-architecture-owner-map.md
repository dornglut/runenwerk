---
title: UI Program Architecture Owner Map
description: Implemented Runenwerk UI ownership map for UiProgram, retained UI compatibility/foundation crates, runtime artifacts, evaluation, rendering boundaries, proof, and host contracts.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related:
  - ./ui-program-architecture.md
  - ../../architecture/ui-framework-architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Program Architecture Owner Map

## Status

This is the implemented owner map for the Runenwerk-local UiProgram architecture.
It summarizes the current semantic ownership encoded more mechanically in
`domain/ui/ui-crate-ownership.toml`.

The table is not a standalone RunenUI extraction plan and does not declare the
retained UI stack obsolete. Foundation and retained compatibility owners remain
current wherever the repository ownership map assigns them responsibility.

## Ownership rules

1. One semantic responsibility has one obvious owner.
2. Program contracts stay renderer- and application-neutral.
3. Host/application owners keep domain mutation authority.
4. Renderer owners consume renderer-neutral output; they do not own UI semantics.
5. Retained/foundation paths remain valid until a separately accepted cutover proves
   migration and deletion.
6. Transitional dependencies are explicit and mechanically checked rather than
   treated as the desired final dependency graph.

## final owner map

| Layer | Crate / path | Current responsibility |
| --- | --- | --- |
| foundation | `domain/ui/ui_math` | Primitive UI math/value contracts. |
| foundation | `domain/ui/ui_schema` | Schema references, property/value schemas, validation vocabulary. |
| foundation | `domain/ui/ui_text` | Text model contracts and renderer-neutral text facts. |
| foundation | `domain/ui/ui_theme` | Theme tokens and styling contracts. |
| foundation | `domain/ui/ui_input` | Input event/value contracts. |
| foundation | `domain/ui/ui_layout` | Layout contracts and layout facts. |
| composition | `domain/ui/ui_composition` | App-neutral structural composition definitions, ratified state, transactions, history, diagnostics, promotion primitives, and fixtures. |
| composition | `domain/ui/ui_adaptive_composition` | Transient adaptive projection/proposal/preview/session mechanisms and promotion deltas. |
| retained | `domain/ui/ui_tree` | Retained UI tree compatibility substrate. |
| retained | `domain/ui/ui_widgets` | Retained widget compatibility substrate. |
| retained | `domain/ui/ui_runtime` | Retained UI runtime compatibility substrate. |
| retained | `domain/ui/ui_graph_editor` | Existing graph-editor UI compatibility layer. |
| definition | `domain/ui/ui_definition` | Authored and normalized UI definition contracts. |
| program | `domain/ui/ui_controls` | Control packages, control kinds, and control semantic contracts. |
| program | `domain/ui/ui_program` | `UiProgram` identity, typed graph families, routes, source maps, diagnostics, and semantic program contracts. |
| program | `domain/ui/ui_program_lowering` | Deterministic lowering from normalized UI definitions into `UiProgram`. |
| program | `domain/ui/ui_compiler` | Graph/package/capability checks and compiler reports. |
| program | `domain/ui/ui_artifacts` | Runtime artifact manifests, tables, source maps, cache keys, and artifact diagnostics. |
| program | `domain/ui/ui_evaluator` | Evaluator facts, passes, traces, and inspection output. |
| program | `domain/ui/ui_binding` | Binding contracts between UI runtime facts and host-provided data. |
| program | `domain/ui/ui_state` | UI state contracts and transition facts. |
| program | `domain/ui/ui_hosts` | Generic host profile, capability, and route contracts. |
| program | `domain/ui/ui_accessibility` | Accessibility facts and proof contracts. |
| program | `domain/ui/ui_geometry` | UI geometry facts and geometry contracts. |
| program | `domain/ui/ui_runtime_view` | Runtime read model over UI artifacts and host facts. |
| render | `domain/ui/ui_render_data` | Renderer-neutral render-data contracts. |
| render | `domain/ui/ui_render_primitives` | Renderer-neutral primitive generation reports and provenance. |
| proof | `domain/ui/ui_headless_render` | Headless rendering proof support. |
| proof | `domain/ui/ui_headless_render_data` | Headless render-data proof reports. |
| proof | `domain/ui/ui_static_mount` | Static-mount proof facts and reports. |
| proof | `domain/ui/ui_story` | UI Story V2 proof orchestration and workflow evidence contracts. |
| testing | `domain/ui/ui_testing` | UI conformance and test utilities. |
| surface | `domain/ui/ui_surface` | Generic surface definition/mount/intent/observation/presentation envelopes. |

The mechanical source of dependency truth is `domain/ui/ui-crate-ownership.toml`.
If prose and that checked ownership file diverge, current source and mechanically
validated ownership win and this document must be corrected.

## Retained and foundation coexistence

The UiProgram architecture does not collapse all UI responsibilities into
`ui_program`.

Current legitimate non-UiProgram owners include:

- `ui_math` for primitive values;
- `ui_layout` for reusable foundation layout contracts/facts;
- `ui_tree`, `ui_widgets`, and `ui_runtime` as retained compatibility substrates;
- `ui_graph_editor` as the current graph-editor compatibility layer;
- `ui_render_data` / `ui_render_primitives` for renderer-neutral render products.

`ui_definition` currently carries explicit transitional dependencies into retained
preview/formation paths. `ui_tree` and `ui_runtime` also carry explicit transitional
edges documented in the mechanical owner map. These edges are bounded migration
facts, not permission to add new arbitrary cross-layer coupling.

No lifecycle promotion of UiProgram changes those owners by itself.

## Program ownership boundary

`ui_program` owns semantic program representation. It does not own:

- application/editor/game domain state;
- GPU resources or backend submission;
- OS/window lifecycle;
- retained-tree implementation details;
- all layout math/foundation contracts;
- future standalone framework/repository adoption.

`ui_program_lowering`, `ui_compiler`, `ui_artifacts`, and `ui_evaluator` each own a
separate stage so formation, validation, optimized runtime representation, and
execution remain inspectable rather than becoming one monolithic runtime crate.

## Host and render boundary

`ui_hosts` provides generic host contracts. Concrete applications and engine hosts
remain responsible for domain mutation and scheduling.

`ui_runtime_view` plus render-data/primitive owners expose renderer-neutral facts.
Renderer/backend crates may consume those facts but do not become canonical owners
of UI semantic state.

## Proof boundary

`ui_testing`, `ui_story`, headless render owners, and source-owned unit/integration
tests provide proof utilities. They do not become architecture owners merely
because they validate architecture contracts.

Historical closeouts and execution evidence prove past delivery. They are not a
substitute for current source/ownership truth.

## create an owner only when a proven slice needs that owner

Do not create placeholder UI crates merely because a target architecture diagram can
name a future concern. A new owner is justified only when an accepted slice has a
semantic responsibility that cannot truthfully remain with an existing owner and
provides:

- a concrete contract and dependency direction;
- a real producer/consumer path;
- tests or other executable proof;
- diagnostics/inspection where required;
- an explicit migration boundary when replacing existing ownership.

Future product features, standalone RunenUI adoption, broad game/editor/world-space
integration, and new program families remain separate work until those conditions
are met.

## Compatibility and cutover rule

A future cutover may remove or realign retained owners only when the accepted work
identifies all consumers, provides replacement proof, updates the mechanical owner
map, and deletes the obsolete path cleanly. Until then, current compatibility and
foundation owners are part of the supported Runenwerk architecture.
