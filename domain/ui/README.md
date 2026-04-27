# Runenwerk UI Domain (Local Entry)

## What this subtree is
`domain/ui` contains engine-agnostic UI primitives and contracts that are reusable across editor/runtime contexts.

This subtree now includes both primitive crates and retained runtime ownership (`ui_tree`, `ui_runtime`, `ui_widgets`).

## Crate map

- `ui_math`
  - UI geometry/math primitives (`UiRect`, `UiSize`, `UiPoint`, `UiVector`, `UiInsets`, axis types).
- `ui_input`
  - input event/focus/routing contract types (pointer/keyboard/text events, focus ids, shortcut contracts).
- `ui_layout`
  - stateless layout algorithms/contracts (`StackLayout`, `SplitLayout`, constraints, alignment, size policy).
- `ui_text`
  - text primitives and atlas contracts (text style, text buffer/cursor/selection, text layouter contracts).
- `ui_theme`
  - theme token scales and defaults (color, spacing, radius, typography).
- `ui_render_data`
  - renderer-facing UI frame/surface/layer/primitive data contracts.
- `ui_surface`
  - surface-semantic contracts (definitions, mounted lifecycle, capability/session/presentation/intent/ratification boundaries).
- `ui_tree`
  - retained tree contracts (`WidgetId`, node kinds/payloads, tree traversal, computed layout records).
- `ui_runtime`
  - retained runtime orchestration (layout, input routing, frame generation, runtime interaction state).
- `ui_widgets`
  - ergonomic control constructors/builders over `ui_tree` node contracts.

## What belongs here

- reusable engine-agnostic UI primitives and contracts
- reusable retained UI runtime layers
- shared UI testing helpers/harnesses and gallery scenarios

## What does not belong here

- editor-shell-specific workspace host composition and shell command semantics
- app-local runtime wiring and bootstrapping glue
- engine renderer execution policy implementation
- tool/product semantics specific to one editor surface unless intentionally abstracted at domain level

## Current status note
Retained tree/runtime/widget constructor ownership is under `domain/ui`.

`editor_shell` still owns workspace/tool-surface host semantics and re-exports substrate types for compatibility.

Viewport routing no longer uses `first_frame` or `ViewportId(0)` fallback seams; runtime selection is structural, with an explicit single-viewport bootstrap path before first structural binding is established.

Engine debug metrics and scene overlay paths now build frames through substrate contracts instead of ad hoc primitive assembly.

Broad shell-surface migration to all reusable controls (`TextInput`, `Toggle`, `NumericInput`, `Tabs`) is still in progress.

`ui_surface` now owns the base contracts for surface definitions and mounted-surface lifecycle records; host shells mount surfaces through these contracts while retaining shell-specific workspace semantics.

Viewport slot semantics stay in `editor_viewport`; `ui_render_data` now carries only opaque embed slot IDs for renderer-facing payload shape, with mapping handled at integration edges.

Core shell command flows now use surface contracts end-to-end:
- observation -> prepared `SurfacePresentationModel`
- typed `SurfaceIntent` emission
- host-side ratification adapter dispatch

## Canonical docs and architecture links

- Canonical UI architecture:
  - [UI Substrate Architecture](../../docs-site/src/content/docs/domain/ui/architecture.md)
- UI roadmap:
  - [UI Substrate Roadmap](../../docs-site/src/content/docs/domain/ui/roadmap.md)
- Substrate gallery harness example:
  - [`domain/ui/ui_runtime/examples/substrate_gallery.rs`](./ui_runtime/examples/substrate_gallery.rs)
- Workspace/domain architecture boundaries:
  - [Architecture](../../docs-site/src/content/docs/guidelines/architecture.md)
- Module structure guidelines:
  - [Module Structure Guidelines](../../docs-site/src/content/docs/guidelines/module-structure-guidelines.md)
- Editor/UI/workspace/tool-surface architecture:
  - [Editor / UI / Workspace / Tool-Surface Architecture](../../docs-site/src/content/docs/guidelines/editor-ui-workspace-tool-surface-architecture.md)
- Workspace identity contract:
  - [Workspace Identity Contract and Migration Map](../../docs-site/src/content/docs/guidelines/workspace-identity-contract-and-migration-map.md)
