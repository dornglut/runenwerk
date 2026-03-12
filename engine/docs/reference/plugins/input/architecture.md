# Input Plugin Architecture

## Ownership Boundary

- Owns: Input action and pulse lifecycle.
- Does not own: Gameplay systems consuming input.

## Module Layout

- Primary module: engine/src/plugins/input/mod.rs
- Entry surface: InputFinalizePlugin
- Runtime schedule touchpoints: FrameEnd (CoreSet::FrameEnd)

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
