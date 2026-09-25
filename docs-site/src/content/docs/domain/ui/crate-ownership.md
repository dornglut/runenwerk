---
title: UI Crate Ownership
description: Ownership, layer, and dependency boundaries for Runenwerk UI crates.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-25
---

# UI Crate Ownership

This document defines the ownership map for `domain/ui` crates. It is the first enforcement layer before any physical folder reorganization.

The goal is not to create more folders. The goal is to make ownership, dependency direction, and source-of-truth boundaries explicit enough that future UI work cannot accidentally collapse authored source, program, artifact, runtime view, render proof, and host integration into one crate.

## Non-negotiable architecture rule

Runenwerk UI follows the domain program separation:

```text
authored source
-> normalized definition
-> UiProgram
-> compiler/evaluator
-> runtime artifacts
-> runtime view / output facts
-> render proof / static mount proof
-> story proof verdict
-> host/app/editor/game integration
```

`domain/ui` crates may define UI domain contracts, reports, facts, proof envelopes, and deterministic transformations. They must not move app/editor/game/renderer semantics into lower domain layers.

## Layer taxonomy

| Layer | Meaning |
|---|---|
| `foundation` | Current local primitive UI values. Reusable runtime-shaped primitives are predecessor authority for migrated consumers; retained authored vocabulary must have an independent Runenwerk semantic reason. |
| `definition` | Authored and normalized UI source contracts. |
| `definition_adapter` | Migration/bridge/adaptation from definition to older retained UI or app workflows. |
| `program` | UI semantic program, controls, lowering, compiler, artifacts, evaluator, runtime view, binding, host contracts. |
| `render` | Renderer-neutral render facts and primitive-generation contracts. |
| `proof` | Story/proof/inspection/static-headless proof contracts. |
| `composition` | App-neutral saved/ratified structural composition and transient adaptive-mechanism contracts. No app/editor/provider/native-window semantics. |
| `surface` | Temporary predecessor surface/mount/intent compatibility envelopes. This is not a durable target layer. |
| `retained` | Existing local retained runtime/tree/widget compatibility layer. For migrated consumers, mounted/runtime authority moves to RunenUI; product-specific graph-editor semantics may remain separately owned. |
| `testing` | Test helpers and conformance utilities. |
| `app` | Application/editor orchestration. Not allowed inside `domain/ui`; listed for dependency-direction reasoning only. |

## Current ownership map

The machine-readable source of truth is:

```text
domain/ui/ui-crate-ownership.toml
```

This Markdown document explains the policy. The TOML file is used by:

```text
tools/checks/check_ui_layer_dependencies.py
```

## Dependency direction

Allowed dependency direction is deliberately asymmetric.

General rule:

```text
foundation <- definition <- program <- render/proof <- app
```

Compatibility exceptions are allowed only when explicitly listed in `allowed_dependencies`.

## Forbidden patterns

The checker and reviews should reject these patterns:

```text
ui_definition -> ui_tree / ui_widgets / ui_runtime / ui_render_data
ui_story -> apps/runenwerk_editor or editor_* crates
ui_program -> ui_definition assets or app/editor crates
ui_compiler -> app/editor crates
ui_composition -> any other production UI crate
ui_adaptive_composition -> app/editor/engine/ui_surface crates
ui_surface -> concrete game/editor/world semantic ownership
domain/ui crates -> apps/*
```

## Story ownership rule

`ui_story` owns:

```text
story manifest contract
story proof contract
story report contract
expected diagnostic matching
verdict
mount eligibility
```

`ui_story` does not own:

```text
compiler meaning
runtime artifact meaning
runtime view meaning
render primitive meaning
static mount meaning
editor fixture loading
game HUD semantics
world-space/entity-attached UI semantics
```

## Composition ownership rule

`ui_composition` owns:

```text
versioned saved structural definitions
presentation target and structural root identities
split / stack / overlay / mount-point region algebra
opaque typed mounted-content references
ratified structural state and immutable snapshots
typed policy-authorized structural transactions
structural-only journal and undo/redo
core-only promotion candidates
content liveness and unavailable-content fallback vocabulary
ui_composition.* diagnostics and neutral fixture contracts
```

It does not own app extension documents, providers, sessions, product content,
adaptive projection, native windows, monitor/DPI/restore policy, rendering,
editor workspace product semantics, Draw semantics, or game behavior. The
machine-readable ownership map enforces that it has no production dependency
on another UI crate; `ui_testing` may consume it for conformance fixtures.

`ui_adaptive_composition` owns:

```text
transient projection from immutable CompositionSnapshot values
adaptive target constraints and region policy
immutable hit indexes and bounded preview state
drag and resize sessions with cancel/rollback
typed structural proposals and exact edit classification
explicit named and scoped promotion deltas
semantic input parity and accessibility/inspection metadata
ui_adaptive_composition.* diagnostics, neutral fixtures, and performance probes
```

It may depend only on `ui_composition`, `ui_input`, and `ui_math`. It may not
hold mutable composition state, execute transactions, persist canonical state,
or import app/editor/Draw/game/engine/windowing/provider/session semantics.
Hosts own policy acceptance and any later typed transaction submission.

## Temporary surface and host predecessor rules

`ui_surface` remains a temporary compatibility boundary only while current
consumers still depend on it. ADR 0013 requires responsibility-by-responsibility
supersession:

```text
reusable mounted/runtime/input/accessibility semantics
    -> RunenUI when the named consumer migrates

app/domain capability, ratification, session, content and mutation meaning
    -> actual Runenwerk app/domain owner

renderer/product Surface vocabulary
    -> renderer/product owners, not ui_surface
```

No new composition, mounted-runtime, product, or world-space authority may be
added to `ui_surface`. It is deleted after the exact-source census proves its
last maintained responsibility has moved.

`ui_hosts` is likewise predecessor/mixed authority. Only independently useful
UiProgram-specific lifecycle/event/output contracts may survive under an
owner-accurate boundary. Generic runtime, windowing, input, renderer, and app
mutation authority must not remain there.

Concrete semantic verbs like `SelectEntity`, `ActivateField`, `FocusEntity`,
world-space prompt behavior, nameplates, damage numbers, world anchors, culling,
occlusion, and gameplay binding belong to their editor/game/world owners.

## Standalone RunenUI adoption rule

The machine-readable map describes current Runenwerk dependencies; it does not
grant permanent reusable-framework ownership.

For a consumer accepted onto standalone RunenUI:

- `ui_math`, `ui_geometry`, `ui_input`, `ui_layout`, `ui_text`,
  runtime `ui_theme`, `ui_tree`, `ui_widgets`, `ui_runtime`,
  framework-local `ui_state`, reusable `ui_accessibility` and framework
  testing semantics are predecessor authority;
- `ui_controls`, `ui_binding`, `ui_render_data`, `ui_evaluator`,
  `ui_runtime_view`, `ui_hosts`, and `ui_surface` are mixed and must be
  split by responsibility;
- `ui_definition`, `ui_schema`, `ui_program`, `ui_program_lowering`,
  `ui_compiler`, `ui_artifacts`, `ui_story`, `ui_composition`, and
  `ui_adaptive_composition` remain Runenwerk-owned where their semantics are
  independently required.

A migrated consumer must never write both local and RunenUI mounted/runtime
state. The source-bearing migration issue must delete the local path it replaces.

## Physical folder policy

Do not move crates into grouped folders until this ownership map and dependency guard are green.

A later physical reorganization is optional and must be a movement-only patch. Candidate grouping is:

```text
domain/ui/foundations/
domain/ui/retained_runtime/
domain/ui/definition/
domain/ui/program/
domain/ui/rendering/
domain/ui/proof/
domain/ui/surfaces/
```

This is not the endgame by itself. The endgame is enforceable ownership and clean dependency direction.

## Closeout criteria for this ownership slice

- `domain/ui/ui-crate-ownership.toml` exists.
- `tools/checks/check_ui_layer_dependencies.py` exists.
- The checker can run from repo root.
- All current violations are either fixed or listed as explicit transitional exceptions with rationale.
- Docs explain that physical folder movement is conditional, not architecture by itself.
