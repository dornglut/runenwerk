# UI Architecture (Retained ECS + SDF/MSDF)

## Goal
Build an integrated engine UI stack where ECS is the source of UI state and wgpu renderer submits SDF/MSDF draw data via explicit extraction stages.

## Active Direction
- Approach B (retained ECS UI) is active.
- UI state is ECS-owned and renderer-agnostic.
- Renderer resources/pipelines remain isolated from ECS internals.
- UI should run as a scene layer (`OverlayUi`) so gameplay/world scenes can coexist under the same frame.

## Stage Graph
- `ui_input`
- `ui_layout`
- `ui_build_batches`
- `ui_render_extract`
- `ui_render_submit`
- Runtime scheduler node names use `overlay_ui_*` prefixes and end at `frame_render_submit`.

## Stage Responsibilities
- `ui_input`: consume input, update focus/interaction/editor state, emit submit intent.
- `ui_layout`: resolve transforms and size-dependent positioning.
- `ui_build_batches`: convert ECS UI state into renderer-agnostic batch commands.
- `ui_render_extract`: map UI batches into renderer-owned frame draw data.
- `ui_render_submit`: record/submit SDF/MSDF draw calls.

## Data Model Requirements
- ECS components for transforms, style, text, input state, interaction, and dirty flags.
- Explicit event bridge for submit flow.
- Template model supports node IDs and controlled patching/hot reload.

## Current Scope
- Console panel + scrollback + input + confirm button.
- Shared submit path for Enter and button click.
- Text clipping/scissor.
- Hot-reloadable RON templates with component tree and keyed patch behavior.
- Multiline input editing with viewport and vertical caret movement.
- Interactive in-engine editor baseline:
  - `F1` toggles editor mode.
  - Click selects UI node (`root`, `scrollback`, `input`, `confirm_button`).
  - Click-drag moves selected node (Shift enables grid snapping).
  - Arrow keys nudge selected node (hold Shift for larger steps).
  - `X` hides selected node, `A` restores hidden nodes.
  - `Cmd/Ctrl+S` exports and saves current UI template.
- Scene-aware UI template switching baseline:
  - Scene template mapping (`console` and `hud`) with runtime switching.
  - Node visibility persisted in template node data.

## Current Constraints
- Selection/copy-paste not complete.
- IME composition not implemented.
- Advanced rich text/layout not in scope yet.

## Near-Term Implementation Priorities
1. Break `ui_input_system` into smaller input subsystems (text edit, pointer/focus, scroll routing) with explicit stage boundaries.
2. Break `ui_build_batches_system` into composable batch builders (console, logs, controls, debug overlays).
3. Consolidate shared viewport/scroll utility logic used by scene + command systems.
4. Keep logs window and scroll UX fully template-driven and editor-friendly.
5. Continue editor improvements (selection model, clipboard copy/cut/paste, diagnostics polish).

## Risks and Mitigations
- Risk: ECS/render coupling drift.
  - Mitigation: keep extraction boundary strict and typed.
- Risk: editor complexity regressions.
  - Mitigation: behavior-focused unit tests and small incremental merges.
- Risk: hidden perf regressions.
  - Mitigation: basic profiling and targeted allocation cleanup in hot paths.
