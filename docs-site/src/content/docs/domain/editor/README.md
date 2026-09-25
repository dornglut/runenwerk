---
title: Editor Domain
description: Current implementation grouping and ownership guidance for Runenwerk editor-domain crates.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-09-16
publication: primary
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

- `editor_core`: current shared editor session, document, command, transaction,
  ratification, shared-change, reconciliation, and coordination-era contracts. Several
  of these are predecessor-shaped implementation to migrate rather than durable
  universal editor ownership.
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

Current `editor_core` still centralizes a broad `DocumentKind` and one
`EditorSession` for active document/mode plus generic document-tab activation
and ordering. Viewport tool activation instead belongs to mounted-unit-scoped
`SurfaceSessionStore` state; it is not generic `EditorSession` authority. The
generic session no longer owns universal document dirty/save/close persistence
authority. Scene selection is owned by `editor_scene` and the Runenwerk runtime
through a scene-scoped selection context; it is no longer generic `editor_core`
session state.

Scene undo/redo is likewise no longer a universal `editor_core` session stack.
The Runenwerk editor integration owns one explicit scene history context that
retains the originating ratified change together with before/after scene
snapshots and supplies the current shell Undo/Redo availability. Material Lab,
`ui_composition`, and self-authoring histories remain separate owner-specific
histories.

Scene persistence is also explicit and owner-specific. The Runenwerk app host
owns one `ScenePersistenceContext`; `editor_persistence` retains the concrete
`SceneFileV2` DTO, normalization, formation, codec, and migration contracts.
Scene cleanliness is derived from the current normalized persistence projection
versus the context's unbound scene origin or, after successful scene IO, the last
successfully loaded/written normalized scene baseline. The persisted target and
baseline advance only after successful scene write or successful load apply.
Failed scene IO preserves the previous persistence-context knowledge.

This comparison is intentionally limited to content represented by the current
`SceneFileV2`. Runtime/reflected state outside that persistence projection does
not become scene persistence state merely because the editor can inspect or
mutate it. Primary-window close admission reads this explicit scene persistence
state; it does not scan generic document descriptors. Generic
`SaveDocumentTab`/`CloseDocumentTab` persistence commands are not retained as
fake authority. Asset/project, Material Lab, editor-definition, and structural
composition persistence remain independent owners.

The remaining `EditorSession` document/mode aggregation is current
implementation truth and must not be hidden. It remains predecessor shape for
later ADR-0025 implementation work to decompose without compatibility aliases or
duplicate authority. E1 scene selection, E2 scene history, E3 scene persistence,
and E4 viewport tool sessions are bounded ownership repairs, not a claim of full
conformance with ADR 0025.

Likewise, existing product docs may still use “document”, “workspace”, “mode”,
“dirty”, and similar user-facing/current-implementation vocabulary. Those terms
do not establish generic semantic ownership unless a concrete owner contract
says they do.

## Integration

- `apps/runenwerk_editor` wires editor crates into the runnable editor app and
  owns concrete providers, host IO, runtime/window integration, app command
  execution, and the current scene persistence context.
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
