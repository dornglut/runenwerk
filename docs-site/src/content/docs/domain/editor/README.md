---
title: Editor Domain
description: Current implementation grouping and ownership guidance for Runenwerk editor-domain crates.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-09-14
related_designs:
  - ../../design/accepted/runenwerk-editor-coordination-semantic-model.md
  - ../../design/implemented/editor-tool-suite-registry-and-workbench-host-design.md
related_adrs:
  - ../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md
---

# Editor Domain

`domain/editor/*` is the current repository grouping for editor-facing crates.
Directory placement is implementation evidence, not proof that every contract in
that directory is a reusable engine-agnostic editor framework semantic.

Code and tests own current behavior. ADR 0025 and the accepted
[editor coordination semantic model](../../design/accepted/runenwerk-editor-coordination-semantic-model.md)
define the normalized target ownership model for later boundary repair; they do
not claim current Rust already conforms.

## Current Crates

- `editor_core`: current shared editor session, document, selection, history,
  command, transaction, and coordination-era contracts. Several of these are
  predecessor-shaped implementation to migrate rather than durable universal
  editor ownership.
- `editor_definition`: durable editor-definition schemas, validation, and pure
  formation helpers for editor-owned UI/theme/menu/shortcut/binding/catalog
  definition families.
- `editor_inspector`: inspector target resolution, editable value models,
  validation, sessions, and ECS bridge adapters.
- `editor_persistence`: current project/scene file DTOs, codecs, migrations,
  normalization, and formation. These are concrete persistence owners, not a
  universal editor save model.
- `editor_preview`: current external runtime-preview protocol, mode, reload,
  product-reference, and preview-session vocabulary.
- `editor_scene`: editor-facing scene contracts, scene command/proposal paths,
  SDF operation/graph authoring, ratification, deterministic lowering, and
  field-preview formation requests.
- `editor_shell`: current shell composition adapters, tool-suite/Workbench host
  contracts, provider-facing DTOs, and editor-local integration machinery.
  Generic structural topology remains owned by `domain/ui/ui_composition`.
- `editor_viewport`: viewport camera, overlay, hit, snap, expression,
  presentation, and surface-set contracts.

## Ownership Boundary

The normalized rule is not “editor owns everything edited.”

Editor coordination owns local coordination invariants such as explicit
bindings, surface sessions, activation/invocation routing, projection
publication, and relationships to owner-defined selection/history/persistence
contexts.

Semantic owners retain:

- their authored/runtime source truth;
- domain commands and mutation semantics;
- ratification/validation;
- owner-native concurrency/preconditions;
- selection-address meaning;
- history semantics;
- persistence formats and persisted-state cuts.

`domain/ui/ui_composition` owns structural targets, roots, regions, mounted
units, structural transactions/history, and composition persistence. App/engine
layers own native windows, host execution, renderer integration, and concrete
IO/runtime wiring.

## Current Migration Pressure

Current `editor_core` still centralizes a broad `DocumentKind`, one
`EditorSession` active document/tool/mode, one generic `SelectionSet`, and one
`HistoryStack`. That is current implementation truth and must not be hidden.
It is also the primary semantic boundary that later ADR-0025 implementation work
must decompose without compatibility aliases or duplicate authority.

Likewise, existing product docs may still use “document”, “workspace”, “mode”,
“dirty”, and similar user-facing/current-implementation vocabulary. Those terms
do not establish generic semantic ownership unless a concrete owner contract
says they do.

## Integration

- `apps/runenwerk_editor` wires editor crates into the runnable editor app and
  owns concrete providers, host IO, runtime/window integration, and app command
  execution.
- `domain/ui/*` owns reusable UI substrate and app-neutral structural
  composition.
- scene, asset, graph, material, drawing, world, runtime, and future domain
  owners retain their own semantic truth and expose explicit editor adapters or
  contracts as needed.

## Current Architecture Docs

- [Normalized editor coordination semantic model](../../design/accepted/runenwerk-editor-coordination-semantic-model.md)
- [ADR 0025: Normalize Editor Coordination and Semantic Ownership](../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md)
- [`editor-definition-usage.md`](./editor-definition-usage.md)
- [`editor-definition/current-architecture.md`](./editor-definition/current-architecture.md)
