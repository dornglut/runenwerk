# Runenwerk UI Domain (Local Entry)

## What this subtree is
`domain/ui` contains engine-agnostic UI primitives and contracts that are reusable across editor/runtime contexts.

This subtree is currently a strong primitive layer, not yet the full retained UI runtime ownership boundary.

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

## What belongs here

- reusable engine-agnostic UI primitives and contracts
- reusable retained UI runtime layers once ownership extraction is performed
- shared UI testing helpers/harnesses (when introduced)

## What does not belong here

- editor-shell-specific workspace host composition and shell command semantics
- app-local runtime wiring and bootstrapping glue
- engine renderer execution policy implementation
- tool/product semantics specific to one editor surface unless intentionally abstracted at domain level

## Current status note
The retained runtime/tree/layout/input/output implementation currently lives under:

- `domain/editor/editor_shell/src/runtime/*`

That placement is a known current-state ownership mismatch. Do not treat runtime extraction to `domain/ui` as already complete.

Fallback seams also still exist in the current app runtime path (`first_frame`/`ViewportId(0)` usage), so docs and plans should not state those are fully resolved yet.

## Canonical docs and architecture links

- Canonical UI architecture:
  - [UI Substrate Architecture](../../docs-site/src/content/docs/domain/ui/architecture.md)
- UI roadmap:
  - [UI Substrate Roadmap](../../docs-site/src/content/docs/domain/ui/roadmap.md)
- Workspace/domain architecture boundaries:
  - [Architecture](../../docs-site/src/content/docs/guidelines/architecture.md)
- Module structure guidelines:
  - [Module Structure Guidelines](../../docs-site/src/content/docs/guidelines/module-structure-guidelines.md)
- Editor/UI/workspace/tool-surface architecture:
  - [Editor / UI / Workspace / Tool-Surface Architecture](../../docs-site/src/content/docs/guidelines/editor_ui_workspace_tool_surface_architecture.md)
- Workspace identity contract:
  - [Workspace Identity Contract and Migration Map](../../docs-site/src/content/docs/guidelines/workspace-identity-contract-and-migration-map.md)
