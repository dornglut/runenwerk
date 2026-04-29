---
title: UI Substrate Architecture
description: Current-state architecture, ownership boundaries, and migration direction for Runenwerk UI.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# UI Substrate Architecture

## Purpose
Establish the factual, current-state architecture for Runenwerk UI, define correct ownership boundaries, and document remaining migration direction from implemented substrate extraction to full editor/runtime convergence.

This page is intentionally current-state-first.

## Scope
This document covers:

- `domain/ui/*` crates
- retained UI runtime crates under `domain/ui/*`
- workspace/tool-surface host ownership in `editor_shell`
- runtime/app glue in `apps/runenwerk_editor`
- engine render/UI integration paths used to submit and draw UI frames

This document does not define visual design direction, docking product UX, or authored editor-definition workflows.

## Current Reality
As of the audited repository state:

- `domain/ui/*` now owns both primitive crates and retained runtime crates (`ui_tree`, `ui_runtime`, `ui_widgets`).
- Workspace identity/projection/reducer/tool-surface host infrastructure is implemented and belongs to `editor_shell`.
- `apps/runenwerk_editor` owns app runtime bridging and viewport runtime resources/bindings.
- Engine render integration for UI frame submission/extraction is implemented.
- Viewport slot semantic ownership is in `editor_viewport`; renderer-facing embed payload slots are opaque IDs in `ui_render_data`, mapped through integration adapters.
- Core shell control flows (outliner entity selection, viewport product selection, inspector field activation) now route through prepared `SurfacePresentationModel` + typed `SurfaceIntent` + host-side ratification adapters.
- Engine overlay/debug UI paths now route through substrate frame generation (`ui_runtime::build_ui_frame`) instead of ad hoc primitive assembly.
- Prior fallback seams removed:
  - no `first_frame()`-based routing in editor runtime systems
  - no `ViewportId(0)` fallback in shell viewport adapter
- The runtime still has an explicit bootstrap-only single-viewport selection seam before first structural binding exists.
- Substrate output now has baseline snapshot tests and a lightweight gallery harness example (`domain/ui/ui_runtime/examples/substrate_gallery.rs`).

## Current Crate Map

- `domain/ui/ui_math`
  - UI math/geometry primitives (`UiRect`, `UiSize`, `UiPoint`, `UiVector`, `UiInsets`, axis types).
- `domain/ui/ui_input`
  - input and focus contracts (events, pointer/keyboard/text contract types, focus ids, shortcut contracts).
- `domain/ui/ui_layout`
  - stateless layout algorithms/contracts (`StackLayout`, `SplitLayout`, constraints, alignment, size policy).
- `domain/ui/ui_text`
  - text primitives and atlas-based layout contracts (`TextStyle`, buffer/cursor/selection, `TextLayouter`, `AtlasTextLayouter`).
- `domain/ui/ui_theme`
  - theme token scales and defaults (colors, spacing, radius, typography).
- `domain/ui/ui_render_data`
  - renderer-facing `UiFrame`/surface/layer/primitive contracts used by engine renderer extraction.
- `domain/ui/ui_tree`
  - retained tree contracts (`WidgetId`, node kinds/payloads, tree traversal, computed layout records).
- `domain/ui/ui_runtime`
  - retained runtime orchestration (layout engine, input routing, runtime state, frame output generation).
- `domain/ui/ui_widgets`
  - ergonomic widget/control constructors over `ui_tree` node contracts.

Related non-`domain/ui` owners currently in the runtime path:

- `domain/editor/editor_shell`
  - shell composition, workspace host model, command routing, compatibility re-exports for substrate types.
- `apps/runenwerk_editor`
  - app runtime resource wiring, viewport presentation/product runtime resources, tool-surface runtime binding registry.
- `engine/src/plugins/render`
  - UI submission registry, prepared UI payloads, renderer extraction and draw path for UI primitives.

## What `domain/ui/*` Currently Owns

- reusable primitive contracts for UI math, input, layout, text, theme, and render-data payloads
- text atlas and layouter contracts plus concrete atlas layouter implementation
- renderer-facing UI frame/primitive data model consumed by engine render feature code
- retained runtime ownership via:
  - `ui_tree` (retained nodes/tree/layout records)
  - `ui_runtime` (tree orchestration, input routing, frame generation)
  - `ui_widgets` (control/widget constructors)

## What `domain/ui/*` Does Not Yet Own

- fully converged app-side usage of all reusable controls in editor shell surfaces
- shell workspace host semantics (correctly owned by `editor_shell`)
- app/runtime glue and viewport product orchestration (correctly owned by `runenwerk_editor`)

## Relationship Between `domain/ui`, `editor_shell`, `runenwerk_editor`, and Engine Render Integration

### `domain/ui`
Provides reusable engine-agnostic UI primitives/contracts and renderer-facing frame data contracts.

### `editor_shell`
Correct owner for:

- workspace structural identity and graph state
- host/panel/tab/tool-surface composition logic
- shell command routing from UI interactions

### `runenwerk_editor`
Owns app/runtime glue:

- shell controller and dispatch bridge
- viewport runtime resource orchestration
- tool-surface runtime binding resource and viewport runtime systems

### Engine render integration
Owns:

- UI frame submission registry/resources
- prepared UI contribution payloads
- renderer extraction/render path for `UiFrame` primitives

This layer should continue consuming UI frame contracts, not owning UI semantics.

## Current Boundary Violations

1. Viewport semantic-to-render payload slot mapping is intentionally adapter-based and must remain centralized; avoid introducing parallel semantic taxonomies in runtime or renderer layers.
2. Existing shell surfaces still rely primarily on button/label primitives; newer reusable controls (`TextInput`, `Toggle`, `NumericInput`, `Tabs`) are present but not yet broadly adopted.
3. Bootstrap single-viewport routing exists before first structural tool-surface binding generation.

## Target Ownership Model
Target ownership (partially implemented):

- `domain/ui/*` owns reusable UI substrate runtime layers:
  - retained tree/runtime orchestration
  - reusable control runtime
  - input/focus/invalidation behavior
  - shared testing harness
- `editor_shell` owns workspace host semantics and shell-specific composition/command routing only.
- `runenwerk_editor` owns app/runtime wiring and viewport/editor-specific runtime integrations.
- engine render layer continues to own rendering integration and consumes UI frame contracts as data.

## Migration Direction
The migration direction should remain dependency-aware and incremental:

1. complete broader adoption of reusable controls across editor surfaces where ad hoc assembly remains.
2. keep render-data embed slot IDs opaque and generic while preserving semantic slot taxonomy in `editor_viewport`, with explicit mapping at integration edges.
3. keep substrate docs and tests aligned with implemented behavior per phase completion, including surface-flow tracing from observation to ratification.

## Testing and Verification Expectations

- Keep architecture guard tests that enforce structural identity and fail-closed routing behavior.
- Keep baseline unit coverage in primitive crates (`ui_math`, `ui_input`, `ui_layout`, `ui_theme`, `ui_render_data`, `ui_tree`, `ui_widgets`).
- Keep retained-runtime interaction coverage for keyboard/text/focus/invalidation and control interactions.
- Keep UI frame snapshot/fixture verification for stable render-data expectations (`domain/ui/ui_runtime/src/output/build_ui_frame.rs` tests).
- Keep the lightweight substrate gallery harness runnable (`domain/ui/ui_runtime/examples/substrate_gallery.rs`).
- Preserve smoke/architecture tests proving no fallback regression for viewport/tool-surface binding behavior.

## Explicit Non-Goals

- documenting fallback seam removal as complete before code actually removes it
- full docking/tab UX productization in this architecture document
- authored editor-definition/meta-editor system specification here
- speculative future feature taxonomy beyond current audited constraints

## Related Architecture and Workspace Docs

- [Workspace Architecture Boundaries](../../guidelines/architecture.md)
- [Runenwerk Architecture Doctrine](../../guidelines/runenwerk-architecture.md)
- [Module Structure Guidelines](../../guidelines/module-structure-guidelines.md)
- [Editor / UI / Workspace / Tool-Surface Architecture](../../design/active/editor-ui-workspace-tool-surface-architecture.md)
- [Viewport Expression Upgrade Design](../../design/active/viewport-expression-upgrade-design.md)
- [Workspace Identity Contract and Migration Map](../../design/active/workspace-identity-contract-and-migration-map.md)
- [UI Substrate Roadmap](./roadmap.md)
