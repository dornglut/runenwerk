---
title: UI Crate Ownership
description: Ownership, layer, and dependency boundaries for Runenwerk UI crates.
status: active
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
| `foundation` | Primitive UI value types and schema vocabulary. No dependency on higher UI crates. |
| `definition` | Authored and normalized UI source contracts. |
| `definition_adapter` | Migration/bridge/adaptation from definition to older retained UI or app workflows. |
| `program` | UI semantic program, controls, lowering, compiler, artifacts, evaluator, runtime view, binding, host contracts. |
| `render` | Renderer-neutral render facts and primitive-generation contracts. |
| `proof` | Story/proof/inspection/static-headless proof contracts. |
| `surface` | Generic surface/mount/intent envelopes. No concrete game/editor/world semantics. |
| `retained` | Existing retained UI runtime/widges/tree/graph-editor compatibility layer. |
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

## Surface ownership rule

`ui_surface` owns generic surface envelopes only:

```text
surface definition id
surface instance id
host instance id
capability set
intent envelope
payload schema
observation/presentation envelope
mount/session metadata
diagnostics
```

Concrete semantic verbs like `SelectEntity`, `ActivateField`, `FocusEntity`, world-space prompt behavior, nameplates, damage numbers, world anchors, culling, occlusion, and gameplay binding belong to later editor/game/world-space tracks.

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
