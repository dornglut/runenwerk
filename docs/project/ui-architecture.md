# UI Architecture (Retained ECS + SDF/MSDF)

## Goal
Build an integrated engine UI stack where ECS is the source of UI state and wgpu renderer submits SDF/MSDF draw data via explicit extraction stages.

## Active Direction
- Approach B (retained ECS UI) is active.
- UI state is ECS-owned and renderer-agnostic.
- Renderer resources/pipelines remain isolated from ECS internals.

## Stage Graph
- `ui_input`
- `ui_layout`
- `ui_build_batches`
- `ui_render_extract`
- `ui_render_submit`

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

## Current Implemented Scope
- Console panel + scrollback + input + confirm button.
- Shared submit path for Enter and button click.
- Text clipping/scissor.
- Hot-reloadable RON templates with component tree and keyed patch behavior.
- Multiline input editing with viewport and vertical caret movement.

## Current Constraints
- Selection/copy-paste not complete.
- IME composition not implemented.
- Advanced rich text/layout not in scope yet.

## Near-Term Implementation Priorities
1. Selection model and interactions.
2. Clipboard copy/cut/paste.
3. Editor behavior polish and additional tests.
4. Lightweight dev diagnostics for template reload failures.

## Risks And Mitigations
- Risk: ECS/render coupling drift.
  - Mitigation: keep extraction boundary strict and typed.
- Risk: editor complexity regressions.
  - Mitigation: behavior-focused unit tests and small incremental merges.
- Risk: hidden perf regressions.
  - Mitigation: basic profiling and targeted allocation cleanup in hot paths.
