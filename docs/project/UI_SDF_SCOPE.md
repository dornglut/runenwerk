# UI Scope - Custom SDF UI on wgpu (High Priority)

## Goal
Build a fully integrated, engine-level UI stack based on SDF/MSDF rendering in wgpu, with ECS-driven UI state and scheduler-driven execution stages.

## Active Direction (Locked)
- Approach B is the active implementation direction.
- Use retained-mode ECS UI state as the canonical model.
- Render UI through SDF/MSDF pipelines (not clear-color-only fallback).
- Keep renderer internals isolated from ECS data via explicit extraction/batching boundaries.

## Why This Is High Priority
- UI is needed for core loop usability (inventory, hub prep, skill management, party commands).
- Technical direction (materials/shaders for UI) impacts renderer architecture early.
- Early alignment prevents expensive rewrites once gameplay systems expand.

## Architectural Decision
- Use ECS for UI data/state/layout/input representation.
- Use a dedicated render subsystem for wgpu resource management, pipelines, and draw submission.
- Integrate both through scheduler stages in the main engine loop.

This is a single integrated engine architecture, not a separate disconnected UI engine.

## Ownership Boundaries
- `ecs`: UI components, UI resources, dirty flags, query access.
- `game`/engine runtime: UI systems (input, layout, extraction, high-level widgets).
- renderer module (within engine): SDF/MSDF pipelines, materials, shader bindings, batching.
- `scheduler`: stage ordering and dependencies.

## MVP Milestones
1. SDF rect rendering pipeline (quad instances + SDF edge eval).
2. ECS UI nodes (transform/style/visibility/z/clip).
3. UI input + interaction state (hover/press/focus).
4. Scheduler stages for UI (input -> layout -> build_batches -> render_extract -> render_submit).
5. MSDF text pipeline (atlas + glyph instances).
6. Material/shader extension path for UI nodes.

## Current Implementation Goal (Now)
Deliver a retained ECS console window that proves end-to-end UI architecture:
- Console panel container rendered via SDF rect pass.
- Scrollback text region rendered via MSDF text pass.
- Editable input field with keyboard typing support.
- Clickable confirm button rendered as SDF widget.
- Unified submit behavior: `Enter` and confirm button both trigger the same submit path.
- Submission appends to scrollback and clears/reset input field state.

## Required Stage Graph (Initial)
- `ui_input`
- `ui_layout`
- `ui_build_batches`
- `ui_render_extract`
- `ui_render_submit`

## Stage Responsibilities (Required)
- `ui_input`: read keyboard/mouse events, update `UiInteraction` and submit intent.
- `ui_layout`: resolve panel/input/button/scroller transforms and dirty propagation.
- `ui_build_batches`: build renderer-agnostic UI batch data from ECS state.
- `ui_render_extract`: convert UI batches into renderer-owned frame data.
- `ui_render_submit`: record and submit SDF/MSDF draw calls through wgpu.

## Data Model Requirements (Initial)
- UI entities remain ECS-owned.
- Add explicit interaction/state components for button and field behavior:
  - `UiButton` (enabled/visual state)
  - `UiInteraction` (hovered/pressed/clicked/focused)
  - `UiInputField` (cursor, submit_requested)
  - `UiDirty` flags (layout/style/text)
- Input data must include absolute cursor position for hit-testing.

## Technical Constraints
- Deterministic ordering for UI stages.
- Keep ECS renderer-agnostic while exposing extraction-friendly data.
- Minimize per-frame allocations in UI batching.
- Explicit dirty propagation for layout/style changes.

## Risks and Mitigations
- Risk: UI logic and GPU internals become coupled.
  - Mitigation: enforce ECS-data vs renderer-resource boundaries.
- Risk: too many immediate-mode rebuild costs.
  - Mitigation: retained-mode ECS UI + dirty flags.
- Risk: shader/material complexity early.
  - Mitigation: single SDF pipeline MVP before material variants.

## Success Criteria
- UI overlays render via wgpu with stable frame-time behavior.
- UI is fully driven by ECS state and scheduler order.
- Material/shader path exists without rewriting core UI model.
- Console window supports typed input and explicit button confirmation through one submit pipeline.
