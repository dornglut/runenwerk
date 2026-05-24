---
title: Editor Domain
description: Current documentation for engine-agnostic editor domain crates.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-09
---

# Editor Domain

`domain/editor/*` contains engine-agnostic editor contracts used by the
Runenwerk editor app and by future editor-facing integrations.

## Current Crates

- `editor_core`: shared editor session and core state contracts.
- `editor_definition`: durable editor definition document schemas, validation,
  and pure formation helpers for editor-owned UI/workspace/theme/menu/shortcut
  definition families.
- `editor_shell`: shell composition, observation frames, view models, commands,
  workspace identity, persistence-facing state, and surface projection
  ratification.
- `editor_viewport`: viewport camera, overlay, hit, snap, expression, and
  surface-set contracts.
- `editor_inspector`: inspector target resolution, editable value models,
  validation, sessions, and ECS bridge adapters.
- `editor_scene`: editor-facing scene contracts, including P1 SDF operation
  documents, source-backed SDF graph documents, operation/graph ratification,
  deterministic lowering to `world_ops` records, and CPU field-preview
  formation requests.
- `editor_persistence`: project/scene files, RON codec, migrations,
  normalization, and change-log persistence.

## Ownership Boundary

Editor domain crates own editor data contracts and engine-agnostic shell logic.
They do not own native window startup, engine plugin wiring, renderer backend
behavior, or app-specific persistence locations.

## Integration

- `apps/runenwerk_editor` wires these crates into the runnable editor app.
- `domain/ui/*` owns retained UI substrate and surface contracts.
- `scene`, `world_ops`, and `world_sdf` provide domain data consumed by editor
  tooling.

## Current Architecture Docs

- [`editor-definition-usage.md`](./editor-definition-usage.md)
- [`editor-definition/current-architecture.md`](./editor-definition/current-architecture.md)
