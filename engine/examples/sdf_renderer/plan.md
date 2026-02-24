# SDF Renderer Data-Driven Setup Plan

## Date

2026-02-24

## Objective

Refactor `engine/examples/sdf_renderer` so setup is loaded from `.ron` files under `engine/examples/sdf_renderer/assets/`:

1. SDF/world parameters
2. Input mappings
3. Render-graph/resource/pipeline/executor registration

Critical boundary:

- The target SDF setup is self-owned. Avoid default world-renderer pass ids (`world_compute`, `world_compose`) as required wiring.
- Example should demonstrate the abstractions a user needs to build an independent renderer feature.

## Target Usage (What We Want)

The example should follow this startup flow:

1. Load `sdf_params.ron`.
2. Load `input_bindings.ron`.
3. Load `render_graph.ron`.
4. Validate configs and emit concise `tracing` diagnostics on failures.
5. Apply defaults/fallbacks where safe.
6. Register feature graph spec and input mappings from loaded config.

Primary authoring API decision:

1. Use typed builder API for runtime graph construction (`RenderFeatureGraphSpec::builder(...)`).
2. Treat `render_graph.ron` as import format that converts into the same typed spec.

## Proposed Config Schemas

### `sdf_params.ron` (example shape)

```ron
(
  world_scene_label: "gameplay_stub",
  world_bounds: (-18.0, -18.0, 18.0, 18.0),
  camera: (
    target: (0.0, 0.8, 0.0),
    yaw: 0.4,
    pitch: 0.25,
    distance: 9.5,
    pitch_min: -1.2,
    pitch_max: 1.2,
    distance_min: 2.0,
    distance_max: 30.0,
    fov_y_radians: 1.0122909,
  ),
  controls: (
    base_move_speed: 7.5,
    speed_up_multiplier: 2.0,
    speed_down_multiplier: 0.35,
    mouse_rotate_sensitivity: 0.0045,
    scroll_zoom_sensitivity: 0.55,
  ),
  debug_view_mode: 0,
  world_paused: false,
  render_mesh_overlay: false,
)
```

### `input_bindings.ron` (example shape)

```ron
(
  bindings: [
    (action: "sdf.move_up", key: "KeyR"),
    (action: "sdf.move_down", key: "KeyF"),
    (action: "sdf.debug_next", key: "Tab"),
    (action: "sdf.debug_prev", key: "Backquote"),
    (action: "sdf.speed_up", key: "KeyE"),
    (action: "sdf.speed_down", key: "KeyQ"),
  ],
)
```

### `render_graph.ron` (example shape)

```ron
(
  feature: "sdf_renderer",
  resources: [
    (id: "sdf.color"),
    (id: "sdf.params"),
    (id: "surface.color"),
  ],
  pipelines: [
    (
      id: "sdf.compute.raymarch",
      key: sdf_compute_raymarch_3d,
    ),
    (
      id: "sdf.compose.fullscreen",
      key: sdf_compose_fullscreen,
    ),
  ],
  passes: [
    (
      id: "sdf.compute",
      kind: compute,
      pipeline: Some((named: "sdf.compute.raymarch")),
      executor: Some("sdf.compute"),
      reads: ["sdf.params"],
      writes: ["sdf.color"],
      depends_on: [],
    ),
    (
      id: "sdf.compose",
      kind: render,
      pipeline: Some((named: "sdf.compose.fullscreen")),
      executor: Some("sdf.compose"),
      reads: ["sdf.color"],
      writes: ["surface.color"],
      depends_on: ["sdf.compute"],
    ),
  ],
)
```

## Engine Changes Needed

1. Add serializable config DTOs for:
   - pipeline ids/refs
   - pass/resource descriptors
   - feature graph spec payload
2. Add conversion functions from DTOs -> runtime types:
   - `RenderFeatureGraphSpec`
   - resource descriptors (new)
   - pipeline descriptors
   - pass descriptors
3. Add robust key parsing for input mapping config:
   - string key names -> `winit::keyboard::KeyCode`
   - clear errors with offending binding index/action/key
4. Add minimal shared config-loading helper:
   - read + parse + contextual error tracing
   - return default/fallback where appropriate
5. Keep logging concise:
   - one error per failed file load/parse
   - one summary info/warn after apply
6. Remove requirement that plugin passes map onto fixed built-in executor names.
7. Add typed builder API as primary authoring surface for specs.

## Example Changes Needed

1. Add:
   - `engine/examples/sdf_renderer/assets/sdf_params.ron`
   - `engine/examples/sdf_renderer/assets/input_bindings.ron`
   - `engine/examples/sdf_renderer/assets/render_graph.ron`
2. Add local config module in example:
   - DTO structs
   - loaders
   - converters
3. Replace hardcoded setup in `main.rs`:
   - apply feature graph spec from config/builder
   - apply input mappings from config
   - apply world/camera params from config or typed defaults
   - register SDF-owned executor bindings
4. Preserve current behavior as default values when file missing/invalid (for dev continuity).

## Implementation Phases

### Phase 1: Params + Input Mapping

1. Add `sdf_params.ron` and `input_bindings.ron`.
2. Load/apply both at setup.
3. Keep render-graph registration in code.

### Phase 2: Render Graph Config

1. Add `render_graph.ron`.
2. Load/convert into `RenderFeatureGraphSpec`.
3. Remove hardcoded registration from setup.
4. Use SDF-owned pass ids/resources/executors (`sdf.*`) rather than `world_*`.

### Phase 3: Harden + Docs

1. Add tests for loader/conversion edge cases.
2. Ensure tracing output is concise and actionable.
3. Update example docs with final schema and extension points.
4. Add a "custom renderer authoring" section (minimum required descriptors + systems).

## Validation Plan

1. `cargo run -p engine --example sdf_renderer` with valid configs.
2. Corrupt each `.ron` file and verify single concise error + safe fallback.
3. Change pipeline/pass config in `render_graph.ron` and verify behavior changes without Rust edits.
4. Confirm controls remap from `input_bindings.ron` without rebuild.
5. Verify no hard dependency remains on `world_compute`/`world_compose` ids in example config.

## Definition of Done

1. Example setup values are primarily file-driven.
2. Hardcoded setup constants are reduced to defaults/fallbacks only.
3. Engine and example logs stay concise under config errors.
4. Docs accurately describe file-driven setup and schema.
5. Example demonstrates custom-renderer composition via engine abstractions, not built-in world renderer reuse.

`sdf.params` source contract:

1. `sdf.params` is a logical render resource id.
2. Source values come from `sdf_params.ron` and/or typed defaults struct.
3. Values are normalized to typed `SdfParams` before per-frame render prepare.
