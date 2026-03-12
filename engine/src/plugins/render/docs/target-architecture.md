# Render Target Architecture

Endgame target architecture for the `engine` render system.

This document describes the intended long-term design for render flow authoring, GPU resource modeling, pass composition, plugin-owned flow contributions, and ECS-first render data integration.

This is a target design document, not a statement that every API below already exists today.

---

## Goals

The render architecture should:

- make common compute/render flow authoring easy
- scale to complex mixed pipelines without redesign
- stay ECS-first for source data and runtime state
- support multiple rendering styles in the same engine
- support plugin-owned flow contributions
- support compositor and postprocess pipelines cleanly
- support GPU simulation and geometry generation workflows
- support debug inspection and live resource views
- avoid exposing low-level registry and executor plumbing in normal usage

The system should work well for:

- fullscreen compute + compose flows
- mesh/raster renderers
- SDF/raymarch renderers
- boids and particle systems
- procedural texture generation
- clipmaps and cached world resources
- influence maps and field simulations
- hybrid renderer stacks
- UI compositing
- multi-plugin mixed render graphs

---

## Architectural Principles

### ECS-first for source data

Use ECS resources and ECS-driven runtime state as the source of truth.

Examples:

- camera state
- gameplay state
- simulation state
- debug state
- render-facing extracted state

Render flows consume ECS state but do not replace it.

### Pass/resource graph as the main rendering abstraction

The public render API should be built around:

- resource declarations
- pass declarations
- explicit reads/writes
- explicit dependencies

Prefer pass/resource language over lower-level graph terminology in the main API.

### Separate CPU state from GPU resources

CPU-side ECS state and GPU-side flow resources are distinct layers.

Examples:

ECS resources:
- `MainCameraState`
- `BoidSimState`
- `ToneMapState`

Render-flow resources:
- `"boids.instances"`
- `"scene.depth"`
- `"surface.color"`
- `"ui.draw_list"`

This distinction must remain clear.

### Plugin composition must be first-class

The architecture must support multiple plugins contributing render passes and resources into a single final flow.

A large engine cannot depend on one giant central render graph definition.

### Keep macro scope narrow

Macros should help with GPU layout and field conversion, not own all state extraction logic.

Good macro responsibilities:
- generate raw GPU-safe layout
- generate conversion impls
- validate field convertibility

Bad macro responsibilities:
- giant state extraction DSLs
- implicit render-context magic
- hidden ECS lookup behavior

### Low magic, strong validation

The system should be ergonomic, but explicit enough to:
- validate resource compatibility
- validate pass dependencies
- debug flow composition easily
- inspect resource lifetimes
- explain errors clearly

---

## Layered Model

The endgame system has four layers.

### 1. ECS world layer

Owns source-of-truth runtime and gameplay data.

Examples:
- gameplay state
- camera state
- render settings
- debug settings
- CPU-side extracted render state

### 2. Render resource model

Owns GPU-facing resources and resource descriptors.

Examples:
- uniform buffers
- storage buffers
- sampled textures
- storage textures
- color targets
- depth targets
- imported external resources
- transient intermediates
- persistent history resources

### 3. Render flow graph

Owns pass topology and resource dependencies.

Examples:
- compute passes
- graphics passes
- fullscreen passes
- copy passes
- present pass
- UI composite pass

### 4. Backend execution layer

Owns actual GPU object creation and execution.

Examples:
- pipeline compilation and caching
- bind groups
- buffer and texture allocation
- transient aliasing
- pass execution
- synchronization and barriers

---

## Target Module Structure

### File

`engine/src/plugins/render/`

```text
render/
├── mod.rs
├── plugin.rs
├── README.md
│
├── api/
│   ├── mod.rs
│   ├── flow.rs
│   ├── resources.rs
│   ├── passes.rs
│   ├── ids.rs
│   └── bindings.rs
│
├── graph/
│   ├── mod.rs
│   ├── flow_graph.rs
│   ├── resource_graph.rs
│   ├── pass_graph.rs
│   ├── validation.rs
│   ├── merge.rs
│   └── planning.rs
│
├── resource/
│   ├── mod.rs
│   ├── descriptors.rs
│   ├── usages.rs
│   ├── lifetime.rs
│   ├── import.rs
│   ├── transient.rs
│   └── registry.rs
│
├── params/
│   ├── mod.rs
│   ├── gpu_value.rs
│   ├── gpu_params.rs
│   ├── layouts.rs
│   └── derive_support.rs
│
├── backend/
│   ├── mod.rs
│   ├── device.rs
│   ├── surface.rs
│   ├── formats.rs
│   ├── pipeline_cache.rs
│   ├── resource_allocator.rs
│   ├── barriers.rs
│   └── execution.rs
│
├── passes/
│   ├── mod.rs
│   ├── compute.rs
│   ├── graphics.rs
│   ├── fullscreen.rs
│   ├── copy.rs
│   ├── present.rs
│   └── ui_composite.rs
│
├── composition/
│   ├── mod.rs
│   ├── contribution.rs
│   ├── namespaces.rs
│   └── integration.rs
│
├── inspect/
│   ├── mod.rs
│   ├── graph_dump.rs
│   ├── resource_inspector.rs
│   ├── texture_view.rs
│   └── timings.rs
│
└── builtin/
    ├── mod.rs
    ├── surface.rs
    ├── depth.rs
    ├── ui.rs
    ├── fullscreen_triangle.rs
    └── common_resources.rs
```

## Target Public API

### Root flow authoring

#### File

`engine/src/plugins/render/api/flow.rs`

```rust
pub struct RenderFlow { /* ... */ }

impl RenderFlow {
    pub fn new(id: impl Into<String>) -> Self;

    pub fn ecs_resource<T>(self) -> Self
    where
        T: engine::prelude::Component + 'static;

    pub fn uniform_buffer<T>(self, id: &'static str) -> Self
    where
        T: crate::plugins::render::GpuParams + 'static;

    pub fn storage_buffer<T>(self, id: &'static str) -> Self
    where
        T: crate::plugins::render::GpuParams + 'static;

    pub fn sampled_texture(self, id: &'static str) -> Self;

    pub fn storage_texture(self, id: &'static str) -> Self;

    pub fn color_target(self, id: &'static str) -> Self;

    pub fn depth_target(self, id: &'static str) -> Self;

    pub fn import_texture(self, id: &'static str) -> Self;

    pub fn import_buffer(self, id: &'static str) -> Self;

    pub fn compute_pass(self, id: &'static str) -> ComputePassBuilder;

    pub fn graphics_pass(self, id: &'static str) -> GraphicsPassBuilder;

    pub fn fullscreen_pass(self, id: &'static str) -> FullscreenPassBuilder;

    pub fn copy_pass(self, id: &'static str) -> CopyPassBuilder;

    pub fn present_pass(self, id: &'static str) -> PresentPassBuilder;

    pub fn builtin_ui_composite_pass(self, id: &'static str) -> BuiltinUiCompositePassBuilder;

    pub fn merge(self, other: RenderFlow) -> Self;
}
```

### Compute pass target API

#### File

`engine/src/plugins/render/api/passes.rs`

```rust
pub struct ComputePassBuilder { /* ... */ }

impl ComputePassBuilder {
    pub fn shader(self, path: &'static str) -> Self;

    pub fn uniform_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: engine::prelude::Component + 'static,
        P: crate::plugins::render::GpuParams + 'static;

    pub fn storage_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: engine::prelude::Component + 'static,
        P: crate::plugins::render::GpuParams + 'static;

    pub fn reads(self, id: &'static str) -> Self;

    pub fn writes(self, id: &'static str) -> Self;

    pub fn workgroup_size(self, x: u32, y: u32, z: u32) -> Self;

    pub fn depends_on(self, id: &'static str) -> Self;

    pub fn finish(self) -> RenderFlow;
}
```

### Graphics pass target API

#### File

`engine/src/plugins/render/api/passes.rs`

```rust
pub struct GraphicsPassBuilder { /* ... */ }

impl GraphicsPassBuilder {
    pub fn shader(self, path: &'static str) -> Self;

    pub fn uniform_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: engine::prelude::Component + 'static,
        P: crate::plugins::render::GpuParams + 'static;

    pub fn uniform_state_with_surface<S, P>(self, build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: engine::prelude::Component + 'static,
        P: crate::plugins::render::GpuParams + 'static;

    pub fn vertex_buffer(self, id: &'static str) -> Self;

    pub fn index_buffer(self, id: &'static str) -> Self;

    pub fn instance_buffer(self, id: &'static str) -> Self;

    pub fn indirect_buffer(self, id: &'static str) -> Self;

    pub fn sample_texture(self, id: &'static str) -> Self;

    pub fn reads(self, id: &'static str) -> Self;

    pub fn writes(self, id: &'static str) -> Self;

    pub fn depth_target(self, id: &'static str) -> Self;

    pub fn depends_on(self, id: &'static str) -> Self;

    pub fn finish(self) -> RenderFlow;
}
```

### Fullscreen pass target API

#### File

`engine/src/plugins/render/api/passes.rs`

```rust
pub struct FullscreenPassBuilder { /* ... */ }

impl FullscreenPassBuilder {
    pub fn shader(self, path: &'static str) -> Self;

    pub fn uniform_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: engine::prelude::Component + 'static,
        P: crate::plugins::render::GpuParams + 'static;

    pub fn uniform_state_with_surface<S, P>(self, build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: engine::prelude::Component + 'static,
        P: crate::plugins::render::GpuParams + 'static;

    pub fn sample_texture(self, id: &'static str) -> Self;

    pub fn reads(self, id: &'static str) -> Self;

    pub fn writes(self, id: &'static str) -> Self;

    pub fn clear_color(self, rgba: [f32; 4]) -> Self;

    pub fn depends_on(self, id: &'static str) -> Self;

    pub fn finish(self) -> RenderFlow;
}
```

### Render flow contributions

#### File

`engine/src/plugins/render/composition/contribution.rs`

```rust
pub struct RenderFlowContribution { /* ... */ }

impl RenderFlowContribution {
    pub fn new(namespace: impl Into<String>) -> Self;

    pub fn ecs_resource<T>(self) -> Self
    where
        T: engine::prelude::Component + 'static;

    pub fn uniform_buffer<T>(self, id: &'static str) -> Self
    where
        T: crate::plugins::render::GpuParams + 'static;

    pub fn storage_buffer<T>(self, id: &'static str) -> Self
    where
        T: crate::plugins::render::GpuParams + 'static;

    pub fn sampled_texture(self, id: &'static str) -> Self;

    pub fn storage_texture(self, id: &'static str) -> Self;

    pub fn color_target(self, id: &'static str) -> Self;

    pub fn depth_target(self, id: &'static str) -> Self;

    pub fn compute_pass(self, id: &'static str) -> ComputePassBuilder;

    pub fn graphics_pass(self, id: &'static str) -> GraphicsPassBuilder;

    pub fn fullscreen_pass(self, id: &'static str) -> FullscreenPassBuilder;

    pub fn builtin_ui_composite_pass(self, id: &'static str) -> BuiltinUiCompositePassBuilder;
}
```

Contributions are merged into a final flow during planning/validation.

### Target app-level composition

#### File

`engine/src/app/domain/app.rs`

```rust
impl App {
    pub fn add_render_flow(&mut self, flow: RenderFlow) -> &mut Self;

    pub fn add_render_flow_contribution(
        &mut self,
        contribution: RenderFlowContribution,
    ) -> &mut Self;

    pub fn add_input_bindings(
        &mut self,
        bindings: impl IntoIterator<Item = (&'static str, winit::keyboard::KeyCode)>,
    ) -> &mut Self;
}
```

Input binding ownership belongs to the input/app layer, not the render flow layer.

## GPU Param Model

### Field conversion trait

#### File

`engine/src/plugins/render/params/gpu_value.rs`

```rust
use bytemuck::{Pod, Zeroable};

pub trait ToGpuValue {
    type Gpu: Pod + Zeroable + Copy + 'static;

    fn to_gpu_value(&self) -> Self::Gpu;
}
```

### Intended built-in impls

Implement `ToGpuValue` for:

- `bool`
- `u32`
- `i32`
- `f32`
- `[u32; N]`
- `[i32; N]`
- `[f32; N]`

Later also implement for:

- `glam::Vec2`
- `glam::Vec3`
- `glam::Vec4`
- `glam::Mat4`

### Bool handling

`bool` should be supported globally by `ToGpuValue`, not through field annotations.

Example implementation concept:

```rust
#[repr(transparent)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBoolU32(pub u32);

impl ToGpuValue for bool {
    type Gpu = GpuBoolU32;

    fn to_gpu_value(&self) -> Self::Gpu {
        GpuBoolU32(u32::from(*self))
    }
}
```

### GPU param trait

#### File

`engine/src/plugins/render/params/gpu_params.rs`

```rust
use bytemuck::{Pod, Zeroable};

pub trait GpuParams {
    type Raw: Pod + Zeroable + Copy + 'static;

    fn to_gpu(&self) -> Self::Raw;
}
```

### Layout-specific derives

Prefer layout-specific derives instead of `#[derive(Gpu)]` plus a separate layout attribute.

Target derives:

- `GpuUniform`
- `GpuStorage`

Later, possibly:

- `GpuVertex`
- `GpuPushConstant`

Target style:

```rust
use engine::plugins::render::{GpuStorage, GpuUniform};

#[derive(Debug, Clone, Copy, GpuUniform)]
pub struct CameraParams {
    pub view_proj: glam::Mat4,
    pub eye_position: glam::Vec3,
    pub time_seconds: f32,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
pub struct BoidInstance {
    pub position: glam::Vec3,
    pub velocity: glam::Vec3,
    pub color: glam::Vec4,
}
```

### Responsibilities of the derive

The derive should:

- generate a raw GPU-safe struct
- convert every field via `ToGpuValue`
- fail to compile if any field type does not implement `ToGpuValue`
- generate layout-appropriate padding and alignment

### Non-goals

The derive should not:

- fetch ECS resources
- inspect render context
- own state extraction logic
- become a giant annotation DSL

## ECS State to Param Projection

Render parameter projection should live on ECS state/resources.

### Example

#### File

`engine/examples/game_of_life_sdf/runtime/state.rs`

```rust
impl GameOfLifeSdfState {
    pub fn compute_params(&self) -> crate::rendering::GameOfLifeComputeParams {
        crate::rendering::GameOfLifeComputeParams {
            grid_size: self.grid_size,
            step: self.step_simulation,
        }
    }

    pub fn compose_params(
        &self,
        surface: (u32, u32),
    ) -> crate::rendering::GameOfLifeComposeParams {
        crate::rendering::GameOfLifeComposeParams {
            output_size: [surface.0 as f32, surface.1 as f32],
            grid_size: [self.grid_size[0] as f32, self.grid_size[1] as f32],
            cell_radius: self.cell_radius,
            edge_softness: self.edge_softness,
            grid_line_width: self.grid_line_width,
            glow_strength: self.glow_strength,
            alive_color: self.alive_color,
            dead_color: self.dead_color,
            grid_color: self.grid_color,
            background_color: self.background_color,
        }
    }
}
```

### Why this belongs on the state

Because:

- the state already owns the source values
- param structs remain dumb value objects
- flow authoring stays clean
- no manual `FromRenderContext` is needed
- no giant derive-macro state mapping system is needed

## Resource Categories

The endgame model should distinguish these resource kinds explicitly.

### ECS resources

CPU-side runtime state.

Examples:

- `MainCameraState`
- `BoidSimState`
- `ToneMapState`

### Imported GPU resources

GPU resources owned outside the flow.

Examples:

- `"surface.color"`
- `"scene.depth"`
- `"shadow.atlas"`
- `"editor.viewport.color"`

### Flow-owned persistent resources

Long-lived GPU resources owned by a flow or contribution.

Examples:

- `"boids.instances"`
- `"taa.history"`
- `"terrain.clipmap.cache"`

### Flow-owned transient resources

Intermediate GPU resources used within a frame/flow.

Examples:

- `"post.bloom.extract"`
- `"post.bloom.blur_x"`
- `"post.bloom.blur_y"`

This distinction should exist in the internal architecture even if the public API starts simpler.

## Resource IDs and Namespacing

Use namespaced resource IDs and pass IDs.

### Resource examples

- `"boids.instances"`
- `"boids.sim.params"`
- `"scene.depth"`
- `"surface.color"`
- `"post.bloom.extract"`
- `"ui.draw_list"`

### Pass examples

- `"boids.simulate"`
- `"boids.draw"`
- `"scene.depth_prepass"`
- `"post.tonemap"`
- `"ui.composite"`

This avoids collisions in mixed-plugin flows.

Strings are acceptable as the initial target surface. Typed IDs may be introduced later.

## Imported Resource Support

Imported resources must be first-class.

Needed because of:

- swapchain/surface targets
- scene depth
- editor viewports
- shared shadow atlases
- previous-frame history textures
- shared debug resources

### Target methods

- `.import_texture("surface.color")`
- `.import_texture("scene.depth")`
- `.import_buffer("shared.instance_data")`

Without imported resources, real engine composition becomes awkward.

## Validation Model

Validation must be a core feature of the graph layer.

The system should validate:

- resource existence
- duplicate IDs
- pass existence
- cycles
- invalid dependency chains
- incompatible read/write usage
- type/layout mismatches
- plugin contribution namespace collisions
- invalid imported resource usage

Validation should happen before backend execution planning.

## Inspection and Debugging

Inspection should be designed in from the start.

The render system should support:

- graph dumps
- pass order dumps
- resource lifetime views
- transient aliasing views
- live texture inspection
- GPU timings
- per-pass timings
- debug overlays
- resource usage inspection

This is essential for large mixed flows.

## Example Endgame Usage

### Main app composition

```rust
use anyhow::Result;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::*;
use winit::keyboard::KeyCode;

pub fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("Endgame Render Architecture Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);

    app.init_resource::<crate::camera::MainCameraState>();
    app.init_resource::<crate::boids::BoidSimState>();
    app.init_resource::<crate::post::ToneMapState>();

    app.add_input_bindings([
        ("app.pause", KeyCode::Space),
        ("app.debug.next_view", KeyCode::Tab),
        ("app.debug.prev_view", KeyCode::Backquote),
    ]);

    app.add_render_flow(
        RenderFlow::new("main_view")
            .import_texture("surface.color")
            .import_texture("scene.depth")
            .import_texture("ui.draw_list")
    );

    app.add_render_flow_contribution(
        crate::scene::SceneRenderContribution::new()
    );

    app.add_render_flow_contribution(
        crate::boids::BoidsRenderContribution::new()
    );

    app.add_render_flow_contribution(
        crate::post::PostProcessContribution::new()
    );

    app.add_render_flow_contribution(
        crate::debug::DebugInspectContribution::new()
    );

    app.add_render_flow_contribution(
        crate::ui::UiCompositeContribution::new()
    );

    app.run()
}
```

### Boids contribution example

```rust
use engine::plugins::render::{GpuStorage, GpuUniform};
use engine::plugins::render::flow::RenderFlowContribution;

#[derive(Debug, Clone, Copy, GpuUniform)]
pub struct BoidSimParams {
    pub delta_time: f32,
    pub world_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub boid_count: u32,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
pub struct BoidInstance {
    pub position: glam::Vec3,
    pub velocity: glam::Vec3,
    pub color: glam::Vec4,
}

impl BoidSimState {
    pub fn sim_params(&self) -> BoidSimParams {
        BoidSimParams {
            delta_time: self.delta_time,
            world_radius: self.world_radius,
            separation_weight: self.separation_weight,
            alignment_weight: self.alignment_weight,
            cohesion_weight: self.cohesion_weight,
            boid_count: self.boid_count,
        }
    }
}

pub struct BoidsRenderContribution;

impl BoidsRenderContribution {
    pub fn new() -> RenderFlowContribution {
        RenderFlowContribution::new("boids")
            .ecs_resource::<BoidSimState>()
            .uniform_buffer::<BoidSimParams>("boids.sim.params")
            .storage_buffer::<BoidInstance>("boids.instances")
            .compute_pass("boids.simulate")
                .shader("assets/shaders/boids_sim.wgsl")
                .uniform_state(BoidSimState::sim_params)
                .writes("boids.instances")
                .workgroup_size(64, 1, 1)
                .finish()
            .graphics_pass("boids.draw")
                .shader("assets/shaders/boids_draw.wgsl")
                .vertex_buffer("boids.instances")
                .writes("surface.color")
                .depends_on("boids.simulate")
                .finish()
    }
}
```

### Postprocess contribution example

```rust
use engine::plugins::render::GpuUniform;
use engine::plugins::render::flow::RenderFlowContribution;

#[derive(Debug, Clone, Copy, GpuUniform)]
pub struct ToneMapParams {
    pub exposure: f32,
    pub gamma: f32,
}

impl ToneMapState {
    pub fn params(&self) -> ToneMapParams {
        ToneMapParams {
            exposure: self.exposure,
            gamma: self.gamma,
        }
    }
}

pub struct PostProcessContribution;

impl PostProcessContribution {
    pub fn new() -> RenderFlowContribution {
        RenderFlowContribution::new("post")
            .ecs_resource::<ToneMapState>()
            .uniform_buffer::<ToneMapParams>("post.tonemap.params")
            .color_target("post.bloom.extract")
            .color_target("post.bloom.blur_x")
            .color_target("post.bloom.blur_y")
            .fullscreen_pass("post.bloom_extract")
                .shader("assets/shaders/bloom_extract.wgsl")
                .reads("surface.color")
                .writes("post.bloom.extract")
                .finish()
            .fullscreen_pass("post.bloom_blur_x")
                .shader("assets/shaders/blur_x.wgsl")
                .reads("post.bloom.extract")
                .writes("post.bloom.blur_x")
                .depends_on("post.bloom_extract")
                .finish()
            .fullscreen_pass("post.bloom_blur_y")
                .shader("assets/shaders/blur_y.wgsl")
                .reads("post.bloom.blur_x")
                .writes("post.bloom.blur_y")
                .depends_on("post.bloom_blur_x")
                .finish()
            .fullscreen_pass("post.tonemap")
                .shader("assets/shaders/tonemap.wgsl")
                .uniform_state(ToneMapState::params)
                .reads("post.bloom.blur_y")
                .writes("surface.color")
                .depends_on("post.bloom_blur_y")
                .finish()
    }
}
```

### UI contribution example

```rust
use engine::plugins::render::flow::RenderFlowContribution;

pub struct UiCompositeContribution;

impl UiCompositeContribution {
    pub fn new() -> RenderFlowContribution {
        RenderFlowContribution::new("ui")
            .builtin_ui_composite_pass("ui.composite")
                .reads("ui.draw_list")
                .writes("surface.color")
                .depends_on("post.tonemap")
                .finish()
    }
}
```

## Why this is the target to build toward

This architecture is:

- ergonomic for normal users
- expressive for advanced users
- flexible for compute and graphics workloads
- suitable for multiple renderer styles
- scalable to plugin-composed mixed graphs
- explicit enough for validation and debugging
- consistent with ECS-first engine design

It supports:

- boids
- particles
- fluids
- SDF rendering
- mesh rendering
- postprocess
- compositor chains
- geometry generation
- clipmaps
- temporal history systems
- debug overlays and inspection

without forcing low-level render internals into user code.

## Open Questions / Future Extensions

These are not blockers for the target, but should be acknowledged:

- typed resource IDs instead of strings
- `GpuVertex` derive
- `GpuPushConstant` derive
- transient resource aliasing policy exposure
- explicit barrier model exposure or hiding strategy
- multi-view / multi-camera flow composition
- editor-authored graph tooling
- asset-authored graph fragments
- hot reload for shaders and flow definitions

These should be planned, but not required to validate the core direction.

## Roadmap Phases

The target architecture should later be turned into a roadmap.

Recommended implementation phases:

### Phase 1 - Foundations

- introduce `ToGpuValue`
- introduce `GpuParams`
- implement `GpuUniform`
- implement basic `RenderFlow`
- implement `ecs_resource`
- implement `uniform_buffer`, `storage_buffer`, `sampled_texture`, `color_target`
- implement `compute_pass`, `fullscreen_pass`, `builtin_ui_composite_pass`

### Phase 2 - ECS-first param projection

- standardize state -> params projection pattern
- implement `uniform_state`
- implement `uniform_state_with_surface`
- migrate one or two examples

### Phase 3 - Resource model expansion

- add imported resources
- add depth targets
- add storage textures
- add persistent vs transient internal model
- add validation for usage categories

### Phase 4 - Pass model expansion

- add `graphics_pass`
- add vertex/index/instance/indirect bindings
- add `copy_pass`
- add `present_pass`

### Phase 5 - Plugin composition

- add `RenderFlowContribution`
- add flow merge/integration
- add namespace validation
- migrate multiple plugins to contribution model

### Phase 6 - Inspection and tooling

- graph dump
- timings
- live texture/resource inspect
- flow validation diagnostics
- debug overlays

### Phase 7 - Advanced features

- history resources
- clipmap-oriented flows
- temporal effects
- advanced resource aliasing
- optional editor-facing graph tooling