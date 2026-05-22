---
title: "Render Flow Usage Guide"
description: "Documentation for Render Flow Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-22
---

# Render Flow Usage Guide

## Happy Path

1. Add `RenderPlugin`.
2. Model frame-owned simulation/render values as ECS `Resource` types.
3. Build a `RenderFlow` with ergonomic declarations:
   - `with_state`, `with_surface_color`, `with_builtin_ui`
   - `with_color_target`, `with_color_target_exact`, `with_depth_target`, `with_history_texture`
   - `double_buffer_storage_array`
   - pass builders (`compute_pass`, `fullscreen_pass`, `graphics_pass`, `copy_pass`, `present_pass`, `builtin_ui_composite_pass`)
4. Validate (`.validate()?`) and register with `App::add_render_flow(...)`.

## Minimal Flow

```rust
use engine::plugins::render::RenderFlow;

let flow = RenderFlow::new("minimal.flow")
    .with_surface_color()
    .fullscreen_pass("minimal.compose")
    .write_surface_color()
    .finish()
    .validate()?;
```

## State-Projected Simulation + Compose

```rust
use engine::plugins::render::{
    GpuStorage, GpuUniform, RenderFlow, RenderVertexBufferLayout, RenderVertexFormat,
};
use engine::prelude::Resource;

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Cell {
    alive: u32,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Instance {
    position: [f32; 2],
    radius: f32,
    _pad: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComputeParams {
    tick: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    surface: [u32; 2],
}

#[derive(Debug, Clone, Resource)]
struct State {
    tick: u32,
}

impl State {
    fn compute_params(&self) -> ComputeParams {
        ComputeParams { tick: self.tick }
    }

    fn compose_params(&self, surface: (u32, u32)) -> ComposeParams {
        ComposeParams {
            surface: [surface.0, surface.1],
        }
    }

    fn dispatch_workgroups(&self) -> [u32; 3] {
        [8, 8, 1]
    }
}

let (flow, instances) = RenderFlow::new("sim.flow")
    .with_state::<State>()
    .with_surface_color()
    .with_color_target("scene.color")
    .with_history_texture("scene.history")
    .double_buffer_storage_array::<Cell>("cells", 1024)
    .storage_array::<Instance>("instances", 128);

let flow = flow
    .compute_pass("simulate")
    .uniform_from_state(State::compute_params)
    .bind_ping_pong_storage("cells")
    .dispatch_from_state(State::dispatch_workgroups)
    .finish()
    .graphics_pass("draw")
    .uniform_from_state_with_surface(State::compose_params)
    .bind_ping_pong_storage("cells")
    .instance_buffer(
        instances,
        RenderVertexBufferLayout::instance(0, 16)
            .attribute(0, 0, RenderVertexFormat::Float32x2)
            .attribute(1, 8, RenderVertexFormat::Float32),
    )
    .write_color_target("scene.color")
    .draw(3, 128)
    .depends_on("simulate")
    .finish()
    .copy_pass("history")
    .source("scene.color")
    .destination("scene.history")
    .depends_on("draw")
    .finish()
    .present_pass("present")
    .source("scene.color")
    .depends_on("history")
    .finish()
    .validate()?;
```

## Procedural Mesh, Quad, And Local SDF Instance Passes

Use `RenderFlow::procedural_pass(...)` when a flow needs bounded local
procedural visuals instead of hand-written fullscreen-plus-instance work. The
procedural API consumes typed storage array handles and emits ordinary graphics
passes with local instance geometry, explicit render policy, and normal
render-flow validation.

```rust
use engine::plugins::render::{
    GpuStorage, ProceduralBufferBinding, ProceduralPassDescriptor,
    ProceduralRenderPolicy, RenderBlendMode, RenderCullMode, RenderDepthPolicy,
    RenderFlow, RenderVertexBufferLayout, RenderVertexFormat,
    SURFACE_COLOR_RESOURCE_LABEL,
};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct SpriteInstance {
    position: [f32; 2],
    radius: f32,
    flags: u32,
}

let (flow, instances) = RenderFlow::new("procedural.flow")
    .with_surface_color()
    .storage_array::<SpriteInstance>("sprites.instances", 512);

let flow = flow
    .procedural_pass(
        ProceduralPassDescriptor::quad_sprites(
            "sprites.draw",
            ProceduralBufferBinding::storage(
                instances,
                RenderVertexBufferLayout::instance(0, 16)
                    .attribute(0, 0, RenderVertexFormat::Float32x2)
                    .attribute(1, 8, RenderVertexFormat::Float32)
                    .attribute(2, 12, RenderVertexFormat::Uint32),
            ),
            512,
        )
        .shader_asset("assets/shaders/sprites.wgsl")
        .write_color_target(SURFACE_COLOR_RESOURCE_LABEL)
        .policy(
            ProceduralRenderPolicy::default()
                .blend_mode(RenderBlendMode::Alpha)
                .depth_policy(RenderDepthPolicy::Default)
                .cull_mode(RenderCullMode::None),
        ),
    )?
    .validate()?;
```

Use `ProceduralPassDescriptor::mesh_sprites(...)` when the pass has an explicit
mesh vertex buffer. Use `ProceduralPassDescriptor::local_sdf_2d_impostors(...)`
for local 2D SDF impostor sprites. The first public SDF impostor path is
intentionally 2D-local only; product-owned SDF authority, 3D raymarching, sparse
residency, freshness, fallback, and rebuild policy stay outside the renderer
procedural API.

The canonical boids example is the reference path for storage-backed procedural
instance rendering. `engine/examples/boids_render_flow/rendering/graph.rs`
keeps simulation in compute passes, publishes the current storage buffer into
the instance buffer consumed by `RenderFlow::procedural_pass(...)`, draws local
2D SDF impostors from `assets/shaders/boids_compose.wgsl`, and presents directly
from the flow-owned color target. It intentionally does not keep a history copy
or use a fullscreen fragment loop over all boids.

For production evidence, run `cargo run -p engine --example boids_render_flow --
--evidence` to print the canonical boids pass-shape, timing-diagnostic, and
benchmark contract. Pair that with `cargo bench -p engine --bench
render_flow_planning`, which includes procedural-boids planning and preflight
cases for the public procedural path.

## Scale Working-Set Residency Evidence

Renderer scale evidence starts with finite working sets, not with unbounded
world size. Product jobs publish selected products and residency requests; the
renderer derives a GPU residency working set with explicit addressable,
selected, requested, accepted, resident, byte, upload, and budget-pressure
counts.

```rust
use engine::plugins::render::{
    RenderGpuResidencyBudgetResource, RenderGpuResidencyResource,
};
use engine::plugins::render::inspect::inspect_render_gpu_residency;
use product::RenderProductSelection;

let mut residency = RenderGpuResidencyResource::default();
let budget = RenderGpuResidencyBudgetResource {
    max_resident_entries: 4096,
    max_resident_bytes: 512 * 1024 * 1024,
    max_upload_bytes_per_frame: 32 * 1024 * 1024,
    ..RenderGpuResidencyBudgetResource::default()
};

let selections: Vec<RenderProductSelection> = Vec::new();
let summary = residency.derive_from_selections(&selections, &budget);
let inspection = inspect_render_gpu_residency(&residency);

assert_eq!(summary.resident_count, inspection.resident_count);
```

`RenderGpuResidencyBudgetResource` is a renderer execution budget. It classifies
resident-entry, resident-byte, and upload-byte pressure and emits diagnostics
when limits are exceeded or byte estimates are invalid. It does not choose
product fallback, streaming, freshness, authority, semantic LOD, or rebuild
policy. Those decisions remain with product owners and later product-specific
tracks. WR-062 adds GPU-driven visibility and indirect submission; WR-063 adds
production examples, benchmarks, and hardware evidence.

## Sparse SDF Brick Page And Clipmap Residency

Sparse SDF renderer residency is derived cache evidence over product-owned SDF
payloads. Product producers publish selected products and residency requests;
the renderer pairs those selections with domain-owned `SdfChunkPayload` sources
and derives page-table, brick-atlas, clipmap-window, invalidation, byte, upload,
and budget-pressure DTOs.

```rust
use engine::plugins::render::features::world::sdf_residency::{
    RenderSdfResidencyBudgetResource, RenderSdfResidencyResource,
    RenderSdfResidencySourceResource,
};
use engine::plugins::render::inspect::inspect_render_sdf_residency;
use product::{ProductIdentity, RenderProductSelection};
use world_sdf::SdfChunkPayload;

let product_id = ProductIdentity::new(7);
let mut sources = RenderSdfResidencySourceResource::default();
sources.upsert_payload(product_id, 3, SdfChunkPayload::default());

let mut residency = RenderSdfResidencyResource::default();
let selections: Vec<RenderProductSelection> = Vec::new();
let summary = residency.derive_from_sources(
    &selections,
    &sources,
    &RenderSdfResidencyBudgetResource::default(),
);
let inspection = inspect_render_sdf_residency(&residency);

assert_eq!(summary.resident_product_count, inspection.resident_product_count);
```

Missing payloads, stale products, generation mismatches, nonresident products,
unsupported query policy, and missing residency requests are diagnostics, not
silent success. `RenderSdfResidencyBudgetResource` reports resident page,
resident brick, resident byte, upload byte, and clipmap window pressure. It
does not choose product fallback, rebuild policy, query authority, collision
truth, gameplay semantics, or SDF authoring behavior. WR-065 owns raymarch
acceleration and candidate lists; WR-066 owns runtime SDF examples, visual
evidence, benchmarks, and production readiness.

## SDF Raymarch Acceleration And Candidate Lists

SDF raymarch acceleration is derived from renderer-owned residency evidence.
The renderer consumes `RenderSdfResidencyResource`, then reports conservative
distance mip safe-step data and screen-tile/depth-slice candidate lists. The
acceleration layer bounds raymarch work and exposes unsafe-overstep or
candidate-explosion diagnostics; it does not read product sources directly,
choose fallback policy, own collision truth, or prove runtime visuals.

```rust
use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfRaymarchAccelerationConfig, RenderSdfRaymarchAccelerationResource,
};
use engine::plugins::render::features::world::sdf_residency::RenderSdfResidencyResource;
use engine::plugins::render::inspect::inspect_last_render_sdf_raymarch_acceleration;

let residency = RenderSdfResidencyResource::default();
let mut acceleration = RenderSdfRaymarchAccelerationResource::default();
let report = acceleration.derive_from_residency(
    &residency,
    RenderSdfRaymarchAccelerationConfig {
        screen_tile_count: 4,
        depth_slice_count: 4,
        max_candidates_per_list: 16,
        ..RenderSdfRaymarchAccelerationConfig::default()
    },
);
let last_report = inspect_last_render_sdf_raymarch_acceleration(&acceleration);

assert_eq!(report.total_candidate_count, last_report.total_candidate_count);
```

Missing SDF residency, zero step or candidate budgets, empty tile/depth
partitions, unsafe empty-space steps, and fullscreen raymarching multiplied per
entity are fail-closed diagnostics. Candidate-list overflow is explicit
rejected-candidate evidence. Residency budget pressure remains visible as a
diagnostic instead of becoming a silent product fallback. WR-066 owns runtime
SDF examples, visual proof, benchmark artifacts, hardware/profile evidence,
and production-readiness claims.

## SDF Runtime Evidence

SDF runtime evidence aggregates the completed SDF residency and raymarch
contracts with executable example, visual, timing, benchmark, and artifact
evidence. Use `inspect_render_sdf_production_evidence(...)` for closeout-ready
reports; it keeps unsupported timestamp queries, missing visual proof, broken
count invariants, and missing benchmark evidence explicit.

```rust
use engine::plugins::render::inspect::{
    RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderSdfProductionEvidenceRequest, RenderSdfProductionHardwareProfile,
    RenderSdfResidencyInspection, RenderSdfRuntimeVisualEvidence,
    inspect_render_sdf_production_evidence,
};
# use engine::plugins::render::features::world::sdf_raymarch::RenderSdfRaymarchAccelerationReport;
# fn example(residency: RenderSdfResidencyInspection, raymarch: RenderSdfRaymarchAccelerationReport) {
let report = inspect_render_sdf_production_evidence(
    RenderSdfProductionEvidenceRequest {
        hardware_profile: RenderSdfProductionHardwareProfile {
            profile_key: "portable-sdf-runtime".to_string(),
            adapter_name: None,
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        residency,
        raymarch,
        timings: RenderDebugTimingsState::default(),
        visual_evidence: vec![RenderSdfRuntimeVisualEvidence {
            view_label: "sdf.lit.near".to_string(),
            coverage_band: "near".to_string(),
            artifact_path: "engine/benchmark-artifacts/render-sdf-runtime-evidence/near.txt".to_string(),
            step_count: 32,
            missed_surface_risk: false,
            overstep_risk: false,
        }],
        benchmark_commands: vec![
            "cargo bench -p engine --bench render_flow_planning".to_string(),
        ],
        artifact_paths: vec![
            "docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md".to_string(),
        ],
    },
);

assert_eq!(report.counts.visual_evidence_count, 1);
# }
```

Run the canonical SDF evidence command with:

```text
cargo run -p engine --example sdf_render_flow -- --evidence
```

Run the standalone evidence report with:

```text
cargo run -p engine --example render_sdf_runtime_evidence -- --evidence
```

Raw SDF runtime artifacts belong under
`engine/benchmark-artifacts/render-sdf-runtime-evidence`. Human-readable
benchmark notes belong in
`docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md`.
The report is runtime evidence only: SDF product truth, query authority,
fallback legality, collision semantics, and rebuild policy remain product-owned.

## Scale Visibility And Indirect Submission Evidence

After residency, renderer scale evidence distinguishes resident candidates from
visible candidates and submitted work. `inspect_render_scale_visibility(...)`
applies renderer-owned bounds, screen-size LOD, compaction budgets, and
capability status to resident candidates and returns explicit visible, culled,
compacted, submitted, and indirect command counts.

```rust
use engine::plugins::render::inspect::{
    RenderScaleVisibilityCandidate, RenderScaleVisibilityCapabilities,
    RenderScaleVisibilityConfig, inspect_render_scale_visibility,
};

let candidates = vec![RenderScaleVisibilityCandidate {
    product_id: 7,
    cache_id: "render-gpu-cache:7".to_string(),
    center: [0.0, 0.0, 0.0],
    radius: 0.1,
    screen_size_px: 128.0,
    resident_bytes: 128,
}];

let visibility = inspect_render_scale_visibility(
    &candidates,
    RenderScaleVisibilityConfig::default(),
    RenderScaleVisibilityCapabilities::supported(),
);

assert_eq!(visibility.visible_count, 1);
assert_eq!(visibility.submitted_draw_count, 1);
assert_eq!(visibility.indirect_command_count, 1);
```

Unsupported storage compaction or indirect submission produces diagnostics and
zero submitted work; it does not fall back to per-entity CPU submission.
Renderer LOD bands are execution buckets only. Product semantic LOD, streaming,
fallback, freshness, authority, and visibility truth remain product-owned.

## Scale Production Evidence

`inspect_render_scale_production_evidence(...)` aggregates the renderer scale
chain into a runtime-readiness report. It consumes residency inspection,
visibility inspection, timing state, a hardware or capability profile, benchmark
commands, and artifact paths. The report keeps addressable, resident, visible,
submitted, and measured costs separate and fails closed when required evidence
is missing.

```rust
use engine::plugins::render::inspect::{
    RenderScaleProductionEvidenceRequest, RenderScaleProductionHardwareProfile,
    inspect_render_scale_production_evidence,
};

# fn example(request: RenderScaleProductionEvidenceRequest) {
let report = inspect_render_scale_production_evidence(request);

assert!(report.is_runtime_ready());
assert_eq!(report.error_count(), 0);
# }
```

Run the canonical evidence example with:

```text
cargo run -p engine --example render_scale_evidence -- --evidence
```

Run the benchmark command with:

```text
cargo bench -p engine --bench render_flow_planning
```

Raw scale benchmark and profile artifacts belong under
`engine/benchmark-artifacts/render-scale-evidence`. Human-readable benchmark
notes belong in `docs-site/src/content/docs/engine/benchmarks`.

## Fullscreen Compose + Present

Fullscreen raymarching and postprocess flows should stay on `fullscreen_pass(...)` when they draw a screen triangle without mesh/instance buffers. Use a flow-owned color target when the frame should end with an explicit terminal present pass:

```rust
use engine::plugins::render::RenderFlow;

let flow = RenderFlow::new("sdf.flow")
    .with_surface_color()
    .with_color_target("sdf.color")
    .fullscreen_pass("sdf.compose")
    .write_color_target("sdf.color")
    .finish()
    .present_pass("sdf.present")
    .source("sdf.color")
    .depends_on("sdf.compose")
    .finish()
    .validate()?;
```

Compatibility path:

- A fullscreen or graphics pass may still write `surface.color` directly with `.write_surface_color()`.
- Use `present_pass(...)` when the flow needs a first-class terminal pass for inspection, ordering, or copy/history work.

## Surface-Format And Exact-Format Color Targets

Use `with_color_target("...")` for presentation-style color targets that should inherit the platform-selected surface/swapchain format. This keeps ordinary scene, viewport, and UI presentation paths aligned with the active adapter and surface capabilities.

Use `with_color_target_exact("...", format)` for surface-sized, flow-owned targets whose byte format is part of the data contract. Exact means exact texture format only; it does not mean fixed size. CPU proof data, deterministic product bytes, and intermediate byte-truth targets should declare the required format explicitly:

```rust
use engine::plugins::render::{RenderFlow, RenderTextureTargetFormat};

let flow = RenderFlow::new("proof.flow")
    .with_color_target_exact("proof.rgba8", RenderTextureTargetFormat::Rgba8Unorm)
    .validate()?;
```

Future fixed-size exact targets should be added as a separate API if a flow needs exact dimensions as well as exact format.

## Product Targets, Aliases, And Prepared Invocations

Use flow-owned targets when one compiled flow writes one local product:

```rust
let flow = RenderFlow::new("product.flow")
    .with_surface_color()
    .with_color_target("product.color")
    .with_history_texture("product.history")
    .fullscreen_pass("product.compose")
    .write_color_target("product.color")
    .finish()
    .copy_pass("product.history")
    .source("product.color")
    .destination("product.history")
    .depends_on("product.compose")
    .finish()
    .present_pass("product.present")
    .source("product.color")
    .depends_on("product.history")
    .finish()
    .validate()?;
```

Use target aliases when authored flow topology should stay static while prepared invocations bind concrete product targets. This is the intended product-surface API shape. Active runtime execution resolves target aliases per prepared invocation and uses the renderer-owned dynamic target cache for requested product targets:

```rust
use engine::plugins::render::{
    PreparedFlowInvocationRequest, PreparedViewFrame, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey, RenderFlow,
    RenderProductSurfaceRequest, RenderProductSurfaceRequestBatch, RenderTextureSampleMode,
    RenderTextureTargetFormat,
};

let flow = RenderFlow::new("viewport.product.flow")
    .with_color_target_alias("viewport.scene_color")
    .fullscreen_pass("viewport.compose")
    .offscreen_products_only()
    .write_color_target("viewport.scene_color")
    .finish();

let target_key = RenderDynamicTextureTargetKey::new("editor.viewport.1", "scene_color");
let target_descriptor = RenderDynamicTextureTargetDescriptor::color_sampled(
    target_key.clone(),
    1280,
    720,
    RenderTextureTargetFormat::Rgba8Unorm,
    RenderTextureSampleMode::FilterableFloat,
    RenderDynamicTextureRetention::RetainWhileRequested,
);

let request = RenderProductSurfaceRequest::new(
    PreparedViewFrame::offscreen_product("viewport.1", (1280, 720))
        .with_history_signature("camera:v1:1280x720"),
    PreparedFlowInvocationRequest::new("viewport.1.scene", flow.id(), "viewport.1")
        .bind_dynamic_texture_alias("viewport.scene_color", target_key)
        .with_history_signature("camera:v1:1280x720"),
)
.with_dynamic_target(target_descriptor);

let batch = RenderProductSurfaceRequestBatch::from_request(request);

dynamic_target_requests.replace_contribution(
    producer_id,
    batch.dynamic_targets().iter().cloned(),
)?;
prepared_frame_requests.replace_contribution(
    producer_id,
    batch.views().iter().cloned(),
    batch.flow_invocations().iter().cloned(),
)?;
```

Upload-backed product surfaces use the same manifest vocabulary, but they opt
in to upload validation explicitly. The renderer can then diagnose missing
uploads without inferring product meaning:

```rust
use engine::plugins::render::{
    RenderDynamicTextureUploadDescriptor, RenderProductSurfaceManifest,
    RenderTextureUploadAlphaMode,
};
use ui_render_data::ProductSurfaceTextureBindingSource;

let upload = RenderDynamicTextureUploadDescriptor::rgba8(
    target_key.clone(),
    0,
    0,
    1280,
    720,
    RenderTextureUploadAlphaMode::Straight,
    product_generation,
    rgba8,
);

let manifest = RenderProductSurfaceManifest::new(producer_id, "editor.texture_preview")
    .with_dynamic_target(target_descriptor)
    .with_dynamic_upload(upload)
    .with_upload_backed_product_surface_binding(
        "texture-preview.primary",
        ProductSurfaceTextureBindingSource::dynamic_texture(
            target_key.namespace.clone(),
            target_key.target_id.clone(),
        ),
    );

let (targets, uploads, views, invocations) = manifest.into_render_parts();
dynamic_target_requests.replace_contribution(producer_id, targets)?;
texture_uploads.replace_contribution(producer_id, uploads)?;
prepared_frame_requests.replace_contribution(producer_id, views, invocations)?;
```

Prepared render frame requests are written before `RenderPrepare`. `RenderPrepare` snapshots requested views, prepared flow invocations, target alias bindings, dynamic target descriptors, projected uniform bytes, dispatch workgroups, and history signatures into `PreparedRenderFrame`. `RenderSubmit` must consume that packet rather than rediscovering product targets from live ECS state.

Current implementation boundary:

- `RenderDynamicTextureTargetRequestRegistryResource` validates producer-scoped dynamic target descriptor contributions and snapshots them into `PreparedRenderFrame`.
- `RenderProductSurfaceRequestBatch` and `RenderProductSurfaceManifest` are return-only. Producers still call `replace_contribution(...)` on `RenderDynamicTextureTargetRequestRegistryResource`, `RenderDynamicTextureUploadRegistryResource`, and `PreparedRenderFrameRequestResource` explicitly.
- `RenderProductSurfaceManifest::diagnostics()` reports producer-scoped duplicate target/upload keys, missing dynamic targets, missing upload descriptors for upload-backed bindings, non-sampleable UI bindings, conflicting history signatures, and producer-owned stale/fallback/rejected/unavailable status.
- `PreparedRenderFrameRequestResource` carries producer-scoped offscreen product views and per-flow invocation requests and exposes typed duplicate diagnostics through `diagnostics()`.
- Target alias execution and renderer-owned dynamic texture cache work are implemented foundation behavior; do not model dynamic products by cloning flows or suffixing static flow resource labels.

History retention should be expressed through explicit history resources or dynamic target retention policy:

- flow-owned history textures use `with_history_texture(...)` plus a `copy_pass(...)` and are scoped per prepared invocation;
- dynamic product targets use `RenderDynamicTextureRetention` and prepared view/invocation history signatures;
- prepared views and invocations carry history signatures so resize, camera, product, or descriptor changes can invalidate only the affected product/history scope.

## Copy Pass Raw Transfer Policy

`copy_pass(...)` is a raw texture transfer. It allows identical color formats and color formats that are identical after removing the sRGB suffix, such as `Rgba8Unorm <-> Rgba8UnormSrgb` and `Bgra8Unorm <-> Bgra8UnormSrgb`.

No color-space conversion happens during `copy_pass(...)`. Unrelated color formats and depth/stencil formats are rejected. Any actual color-space conversion must be modeled as a future explicit shader blit/convert pass family, never hidden inside `copy_pass(...)`.

## UI Composite After Direct Surface Writes

`builtin_ui_composite_pass(...)` writes to the surface color import. It is appropriate after flows that render directly to `surface.color`:

```rust
let flow = RenderFlow::new("ui.flow")
    .with_surface_color()
    .with_builtin_ui()
    .fullscreen_pass("compose")
    .write_surface_color()
    .finish()
    .builtin_ui_composite_pass("ui")
    .depends_on("compose")
    .finish()
    .validate()?;
```

## Validation and Contract Inspection

`RenderFlow` keeps contracts inspectable:

- `flow.validation_report()` returns pass order and validation result details.
- `compile_flow_plan_checked(&flow, &RenderBackendCapabilityProfile::runtime_default())` returns typed compiler diagnostics for static validation failures, resource lifetime windows, and backend-neutral capability mismatches.
- `flow.graph()` exposes declared pass/resource topology for tests and tooling.
- `flow.project_uniforms(frame_data, surface_size)` verifies state projection at frame time.

Prepared-frame preflight is submit-adjacent and runs before backend command encoding. The active renderer uses cached strict preflight by default: full structural validation runs when the prepared-frame structure, compiled flow revision, shader revision, dynamic target signatures, alias bindings, feature gates, history signatures, or uniform/dispatch shape changes. Cheap runtime guards still run every frame.

Full structural preflight validates the compiled flow against the prepared frame packet:

- target alias bindings are present for invocations that execute alias-using passes;
- dynamic target descriptors are valid and compatible with color, depth, sampled, storage, copy, or present roles;
- non-sampleable dynamic targets are rejected before a sampled pass or UI binding tries to use them;
- compute dispatch and uniform bytes are prepared for passes that require them;
- history signatures remain unambiguous for dynamic targets and invocation history scopes;
- feature-gated passes have prepared contribution status and fallback policy;
- backend capabilities are checked through `RenderBackendCapabilityProfile` without exposing WGPU handles.

Use `validate_prepared_render_frame(...)` for tooling/tests that want a report and `preflight_prepared_render_frame(...)` for fail-fast full validation. Runtime submit should go through the renderer-owned cached preflight path. `RenderPreflightValidationConfigResource::strict_every_frame()` or `RUNENWERK_RENDER_STRICT_PREFLIGHT=1` forces full preflight every frame for audits and debugging.

`Renderer::last_preflight_report()`, `Renderer::last_preflight_cache_state()`, `inspect_render_execution_graph_preflight(...)`, and `inspect_render_execution_graph_preflight_with_cache(...)` expose diagnostics and cache source for tooling. `RendererFrameTimings` reports `preflight_ms`, `flow_encode_ms`, and `encode_submit_ms` separately. `RenderDebugTimingsState` also records shader reload poll time/status, diagnostics report time/mode, preflight cache status/source, and the runtime frame pacing mode so steady-state slow frames can be attributed without constructing full JSON diagnostics every frame.

The compiler/preflight path owns render execution correctness only. Product jobs and producers still own product truth, selection, freshness, authority, fallback legality, rebuild policy, and residency intent.

Import-model contract:

- Use typed imports from the public flow API (`with_surface_color`, `with_builtin_ui`) for active runtime flows.
- Use flow-owned `with_depth_target(...)` plus `.depth_target(...)` for runtime-backed graphics depth attachments.
- `with_surface_depth` / `RenderResourceDescriptor::imported_surface_depth` remain declaration compatibility for the typed import model, but imported surface depth is not accepted as a graphics depth target until the renderer exposes a prepared surface-depth texture.
- Avoid generic `RenderResourceDescriptor::imported_texture(...)` / `imported_buffer(...)` for active runtime flows.
- Active validation rejects external/generic import semantics in the runtime path.

Graphics contract:

- `graphics_pass(...)` must declare exactly one color output and explicit draw parameters with `.draw(...)` or `.draw_with_offsets(...)`.
- Vertex and instance buffers are runtime-backed when authored with `RenderVertexBufferLayout::{vertex, instance}` attributes. Layout slots must be dense from `0`, shader locations must be unique, and every buffer must have a matching layout.
- Graphics storage bindings are bind-group storage reads/writes. Raster color/depth writes must use `.write_color_target(...)` / `.depth_target(...)`; storage bindings are not color attachments.
- Graphics passes that draw fullscreen-style generated geometry with `instance_count > 1` are rejected by compiler/preflight diagnostics unless `.allow_instanced_fullscreen(max_instances, reason)` records explicit bounded author intent. Prefer local vertex/instance geometry for procedural sprites and impostors.
- Storage-backed procedural graphics passes without local vertex, index, or instance geometry produce `AmbiguousProceduralShape` diagnostics before submit; this is renderer execution validation, not product truth or fallback policy.

Runtime boundary note:

- `RenderFrameDataRegistry` remains a compatibility helper for projection tests/tools.
- Active frame execution uses `PreparedRenderFrame` produced in `RenderPrepare`.

Advanced feature-tagged pass note:

- `compute_pass(...)`, `fullscreen_pass(...)`, and `graphics_pass(...)` expose optional `.for_feature("feature.id")` tagging.
- Feature-tagged passes execute through the same compiled path but are gated by prepared feature status/fallback policy.
- Only tag passes when the corresponding feature contribution is prepared for the frame; otherwise policy may skip those passes.
- New render features should register typed contribution collectors instead of adding feature-specific central `PreparedFeaturePayload` variants.
- Collectors run in `RenderPrepare`, declare the prepared resources they read, and publish typed diagnostics plus inspectable registered payloads. `RenderSubmit` still consumes only the prepared frame.

Current multi-view scope:

- prepared offscreen product views and per-flow prepared invocations are active through the product-surface path.
- broader native OS multi-window and multi-swapchain presentation remains separate future work.
- finer-grained view-scoped pass subset scheduling should stay explicit through pass scoping and compiled view masks rather than cloned flows.

## Related Examples

- `engine/examples/render_flow_fullscreen_minimal/main.rs`
- `engine/examples/render_flow_postprocess_compositor/main.rs`
- `engine/examples/game_of_life_sdf/main.rs`
- `engine/examples/boids_render_flow/main.rs`
- `engine/examples/sdf_render_flow/main.rs`
- `engine/examples/procedural_sky_sdf_terrain/main.rs`
- `engine/examples/render_flow_debug_inspect/main.rs`
