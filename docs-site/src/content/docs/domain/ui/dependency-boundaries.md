---
title: UI Dependency Boundaries
description: Dependency direction and enforcement policy for Runenwerk UI crates.
status: active
---

# UI Dependency Boundaries

This document defines how UI crates may depend on each other.

It exists to prevent accidental architecture drift while the UI system evolves from retained/editor-specific proof into a long-term domain program family.

## Boundary model

UI crates are grouped by responsibility:

```text
foundation
definition
definition_adapter
program
render
proof
surface
retained
testing
```

Each crate has:

```text
layer
owned responsibility
allowed dependencies
temporary exceptions
forbidden dependencies
```

The machine-readable map is:

```text
domain/ui/ui-crate-ownership.toml
```

The checker is:

```text
tools/checks/check_ui_layer_dependencies.py
```

## Enforcement principle

The checker is intentionally conservative.

It should catch obvious forbidden crossings, not replace design review. A green check means "no known forbidden dependency crossing was detected." It does not mean the architecture is complete.

## Hard forbidden dependencies

These should not be allowed without an accepted design change:

```text
domain/ui/* -> apps/*
domain/ui/* -> editor app crate internals
ui_story -> runenwerk_editor
ui_story -> editor_* crates
ui_program -> ui_definition
ui_program -> ui_story
ui_compiler -> ui_story
ui_artifacts -> ui_story
ui_runtime_view -> ui_story
ui_surface -> game/editor/world concrete semantic crates
```

## Transitional dependencies

Some current dependencies may exist for migration reasons, especially around retained UI and first-button proof.

A transitional exception must include:

```text
why it exists
owner
target removal phase
replacement owner
validation that prevents it from growing
```

No exception is permanent by default.

## Domain source-of-truth boundaries

### Authored UI definition

Owned by:

```text
ui_definition
```

May not own long-term retained runtime or render proof.

### UI program

Owned by:

```text
ui_program
```

May not load authored assets directly. Lowering from definitions belongs to:

```text
ui_program_lowering
```

### Compiler and artifact

Owned by:

```text
ui_compiler
ui_artifacts
```

They may produce diagnostics and artifact manifests, but they must not depend on story/gallery/app orchestration.

### Runtime view

Owned by:

```text
ui_runtime_view
```

It should consume runtime artifacts and expose read models/facts. Control-specific reports should move to control-owned adapters once the component platform expands.

### Render proof

Owned by:

```text
ui_render_primitives
ui_headless_render_data
ui_static_mount
```

These crates produce renderer-neutral proof facts. They do not own editor/game host semantics.

### Story proof

Owned by:

```text
ui_story
```

It aggregates proof evidence and determines verdict/mount eligibility. It does not own the semantic meaning of compiler/runtime/render stages.

### App orchestration

Owned by:

```text
apps/runenwerk_editor
future game/editor/world-space tracks
```

Apps may wire many domain crates together. Domain crates must not depend upward on apps.

## Required validation commands

Run after any dependency or ownership-map change:

```sh
python3 tools/checks/check_ui_layer_dependencies.py --root .
cargo check --workspace
cargo test -p ui_story
cargo test -p runenwerk_editor story
task docs:validate
task planning:validate
```
