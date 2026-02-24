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

## Current Implementation Status (2026-02-24)

Implemented now:

1. Setup uses typed builder API (`RenderFeatureGraphSpec::builder(...)`) instead of manual owner registration DTOs.
2. SDF render path uses feature-owned pass ids:
   - `sdf.compute`
   - `sdf.compose`
3. UI pass dependency is explicitly declared in example graph (`ui_composite` depends on `sdf.compose`).
4. SDF example now uses feature-owned executor ids in graph config:
   - `sdf.compute`
   - `sdf.compose`
   - `ui_composite`
5. Runtime executor registry now uses `executor_bindings` to register custom executors.
6. `sdf.compute`, `sdf.compose`, and `ui_composite` run through custom executor paths in the example.

Still pending for this plan:

1. Add focused tests for loader/conversion edge cases (especially invalid keys and invalid render graph executor/pipeline ids).
2. Keep schema/docs aligned with the now-implemented `.ron` DTO shape as execution phases continue.
3. Keep builtin label contract strict (`builtin_*`) and remove legacy alias acceptance from config parsing.

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

Current bridge usage in code:

```rust
let cfg = load_config_with_default::<SdfRenderGraphConfig>("render_graph.ron");
let spec = cfg.to_spec()?;
data.render_graph_registry.register_feature_graph(spec);
cfg.register_custom_executors(&mut data.render_executor_registry)?;
```

Intended usage later (no built-in delegation):

```rust
let cfg = load_config_with_default::<SdfRenderGraphConfig>("render_graph.ron");
data.render_graph_registry.register_feature_graph(cfg.to_spec()?);
data.render_executor_registry
    .register_custom("sdf.compute", Arc::new(SdfComputeExecutor::new()));
data.render_executor_registry
    .register_custom("sdf.compose", Arc::new(SdfComposeExecutor::new()));
data.render_executor_registry
    .register_custom("ui_composite", Arc::new(SdfUiCompositeExecutor));
```

## Proposed Config Schemas

### `sdf_params.ron` (example shape)

```ron
(
  world_scene_label: "gameplay_stub",
  overlay_scene_label: "console_ui",
  world_bounds: [-18.0, -18.0, 18.0, 18.0],
  camera: (
    target: [0.0, 0.8, 0.0],
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
    camera_target_y_min: -4.0,
    camera_target_y_max: 8.0,
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
  feature: "sdf_renderer_example",
  resources: [
    "sdf.params",
    "world.agents",
    "sdf.color",
    "surface.color",
    "ui.draw_list",
  ],
  compute_pipelines: [
    (
      id: "sdf.compute.raymarch",
      shader: "assets/shaders/sdf_compute_3d_example.wgsl",
    ),
  ],
  render_builtin_pipelines: [
    (
      id: "sdf.compose.fullscreen",
      builtin: "compose.fullscreen",
    ),
    (
      id: "ui.compose",
      builtin: "ui.composite",
    ),
  ],
  executor_bindings: [
    (id: "sdf.compute", builtin: "builtin_compute"),
    (id: "sdf.compose", builtin: "builtin_compose"),
    (id: "ui_composite", builtin: "builtin_ui_composite"),
  ],
  passes: [
    (
      id: "sdf.compute",
      kind: compute,
      pipeline: "sdf.compute.raymarch",
      executor: "sdf.compute",
      reads: ["sdf.params", "world.agents"],
      writes: ["sdf.color"],
      depends_on: [],
    ),
    (
      id: "sdf.compose",
      kind: render,
      pipeline: "sdf.compose.fullscreen",
      executor: "sdf.compose",
      reads: ["sdf.color"],
      writes: ["surface.color"],
      depends_on: ["sdf.compute"],
    ),
    (
      id: "ui_composite",
      kind: render,
      pipeline: "ui.compose",
      executor: "ui_composite",
      reads: ["ui.draw_list"],
      writes: ["surface.color"],
      depends_on: ["sdf.compose"],
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
6. Keep bridge aliases optional; target end-state is fully feature-owned custom executors.
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
   - apply executor bindings from config to custom executor registrations
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
5. Verify no hard dependency remains on `world_compute`/`world_compose` ids in example config (use `builtin_*` labels instead).

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
