# Feature Request: Data-Driven SDF Renderer Example Configuration

## Date

2026-02-24

## Context

The `sdf_renderer` example currently works, but most setup values are hardcoded in `main.rs`:

- feature graph spec registration (pipelines/passes/executors),
- world and camera defaults,
- input bindings.

This is good for bring-up, but it is not aligned with the plugin-owned, data-driven render graph direction.

## Goal

Move example setup data into `.ron` files under `engine/examples/sdf_renderer/assets/`, and make the example load/apply that data at startup.

Hard requirement:

- The SDF example must not depend on default world-renderer pass naming (`world_compute`, `world_compose`) as its target design.
- The example should be a proving ground for user-authored renderers built from engine abstractions.

## Requested Data-Driven Areas

1. SDF/world parameters
2. Input mappings
3. Render graph + pipeline + resource + executor descriptors

## Why This Is Worth Doing

1. The example becomes a true reference for plugin-driven configuration.
2. Tuning no longer requires recompiling Rust code.
3. It exercises engine APIs needed for broader render-graph refactor phases and custom renderer authoring.
4. It reduces boilerplate in the example plugin setup.

## Proposed Config Files (Example)

- `engine/examples/sdf_renderer/assets/sdf_params.ron`
- `engine/examples/sdf_renderer/assets/input_bindings.ron`
- `engine/examples/sdf_renderer/assets/render_graph.ron`

## Non-Goals (This Scope)

1. Full runtime hot-reload for all files (optional follow-up).
2. Replacing all engine frame-graph config immediately.
3. Building a UI editor for these configs.

## Engine Work Likely Required

1. Helper to read `.ron` assets from example-local paths robustly.
2. A serializable descriptor for feature graph specs:
   - resources (not only passes),
   - named pipelines,
   - pass descriptors,
   - optional compatibility fields during migration,
   - executor id per pass.
3. Input-binding config loader that maps `action -> KeyCode`.
4. Validation and concise tracing diagnostics on config load/apply failures.
5. Typed builder API as the primary authoring surface, with optional RON import to the same runtime types.

## Acceptance Criteria

1. Example runs with equivalent behavior using config-driven setup.
2. `main.rs` no longer hardcodes most setup constants.
3. Invalid config produces actionable, concise `tracing` errors.
4. Render-graph registration for the example can be changed in `.ron` without editing Rust code.
5. Example-defined pass/resource/pipeline ids are sufficient; no required dependence on built-in world pass ids.
