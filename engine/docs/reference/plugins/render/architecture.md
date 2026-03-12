# Render Plugin Architecture

## Ownership Boundary

- Owns: Render runtime resource and schedule wiring.
- Does not own: Scene lifecycle ownership.

## Module Layout

- Primary module: engine/src/plugins/render/plugin.rs
- Entry surface: RenderPlugin
- Runtime schedule touchpoints: RenderPrepare, RenderSubmit

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
