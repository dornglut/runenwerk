---
title: UI Substrate Architecture
description: Current-state architecture, ownership boundaries, and migration direction for Runenwerk UI.
---

# UI Substrate Architecture

## Purpose
Establish the factual, current-state architecture for Runenwerk UI, define correct ownership boundaries, and document the migration direction from current implementation placement to target domain ownership.

This page is intentionally current-state-first. It does not treat planned ownership extraction as already complete.

## Scope
This document covers:

- `domain/ui/*` crates
- retained UI runtime placement currently under `domain/editor/editor_shell/src/runtime/*`
- workspace/tool-surface host ownership in `editor_shell`
- runtime/app glue in `apps/runenwerk_editor`
- engine render/UI integration paths used to submit and draw UI frames

This document does not define visual design direction, docking product UX, or authored editor-definition workflows.

## Current Reality
As of the audited repository state:

- `domain/ui/*` is a strong primitive/contract layer.
- A real retained tree runtime (tree/layout/input/output/widgets) exists, but it currently lives in `domain/editor/editor_shell/src/runtime/*`.
- Workspace identity/projection/reducer/tool-surface host infrastructure is implemented and belongs to `editor_shell`.
- `apps/runenwerk_editor` owns app runtime bridging and viewport runtime resources/bindings.
- Engine render integration for UI frame submission/extraction is implemented.
- Fallback seams still exist and are not resolved:
  - `first_frame()` usage in editor runtime systems.
  - `ViewportId(0)` fallback in shell viewport adapter.

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

Related non-`domain/ui` owners currently in the runtime path:

- `domain/editor/editor_shell`
  - retained runtime implementation (`src/runtime/*`), shell composition, workspace host model, command routing.
- `apps/runenwerk_editor`
  - app runtime resource wiring, viewport presentation/product runtime resources, tool-surface runtime binding registry.
- `engine/src/plugins/render`
  - UI submission registry, prepared UI payloads, renderer extraction and draw path for UI primitives.

## What `domain/ui/*` Currently Owns

- reusable primitive contracts for UI math, input, layout, text, theme, and render-data payloads
- text atlas and layouter contracts plus concrete atlas layouter implementation
- renderer-facing UI frame/primitive data model consumed by engine render feature code

## What `domain/ui/*` Does Not Yet Own

- retained runtime/tree/layout orchestration implementation
- widget/control runtime behavior (button/label/panel/scroll/split/stack/viewport-embed node runtime)
- tree-level keyboard/text routing and focus-scope orchestration
- invalidation scheduler semantics beyond low-level response flags
- reusable control library beyond current primitive-layer contracts
- dedicated UI testing/gallery harness crates

## Current Runtime Ownership Mismatch
The reusable retained runtime substrate is implemented, but currently owned by `editor_shell`:

- current location: `domain/editor/editor_shell/src/runtime/*`
- expected long-term owner: `domain/ui/*` runtime-oriented crate/module

This mismatch is the central architecture gap between current reality and target ownership.

## Relationship Between `domain/ui`, `editor_shell`, `runenwerk_editor`, and Engine Render Integration

### `domain/ui`
Provides reusable engine-agnostic UI primitives/contracts and renderer-facing frame data contracts.

### `editor_shell`
Correct owner for:

- workspace structural identity and graph state
- host/panel/tab/tool-surface composition logic
- shell command routing from UI interactions

Currently also owns retained runtime implementation that should become domain-owned.

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

1. Generic runtime implementation located under `editor_shell` instead of `domain/ui`.
2. Viewport-specific embed/binding semantics present in generic `ui_render_data` contracts, increasing product-specific coupling in a shared crate.
3. Slot taxonomy duplication across `ui_render_data`, `editor_viewport`, and app runtime viewport resources.
4. Parallel ad hoc UI stacks in engine scene/debug overlay paths instead of shared substrate reuse.
5. Runtime fallback seams (`first_frame`, `ViewportId(0)`) still active despite stricter structural identity direction.

## Target Ownership Model
Target ownership (not yet fully implemented):

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

1. extract retained runtime modules from `editor_shell` into domain-owned UI runtime crate/module without behavior changes.
2. normalize contracts and remove fallback seams (`first_frame` routing assumptions, `ViewportId(0)` fallback).
3. complete interaction substrate behavior (keyboard/text routing, focus scopes, invalidation semantics).
4. expand reusable controls needed by existing editor surfaces.
5. converge duplicated UI paths where practical onto shared substrate.
6. harden with dedicated UI testing/gallery docs and verification model.

This is a migration path; it is not complete in the current repository state.

## Testing and Verification Expectations

- Keep architecture guard tests that enforce structural identity and fail-closed routing behavior.
- Add missing unit coverage in under-tested UI primitive crates.
- Add retained-runtime interaction coverage for keyboard/text/focus paths as those capabilities are implemented.
- Add UI frame snapshot/fixture verification for stable render-data expectations.
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
- [Editor / UI / Workspace / Tool-Surface Architecture](../../guidelines/editor_ui_workspace_tool_surface_architecture.md)
- [Viewport Expression Upgrade Design](../../guidelines/viewport_expression_upgrade_design.md)
- [Workspace Identity Contract and Migration Map](../../guidelines/workspace-identity-contract-and-migration-map.md)
- [UI Substrate Roadmap](./roadmap.md)
