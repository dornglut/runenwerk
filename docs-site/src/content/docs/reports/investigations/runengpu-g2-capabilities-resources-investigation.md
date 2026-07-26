---
title: RunenGPU G2 Capabilities and Resources Investigation
description: Current-main capability, resource, typed-data, RenderFlow, backend, consumer, and proof-workload census for the decision-complete G2 specification.
status: completed
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-26
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ./runengpu-render-s0-inventory.md
  - ./runengpu-render-s0-file-disposition.md
  - ./runengpu-render-s0-identity-consumer-lifecycle.md
  - ./runengpu-industry-comparison.md
  - ./runengpu-public-api-ergonomics-review.md
  - ./runengpu-proof-workload-strategy.md
  - ../closeouts/pt-runengpu-g1a-closeout.md
---

# RunenGPU G2 Capabilities and Resources Investigation

## Outcome

G2 can proceed without an ADR-level stop.

The verified source baseline is:

```text
repository: dornglut/runenwerk
branch inspected: main
commit: d1dfd2518c988ed1ffa9ff40f2d6df5fe7f5a9b1
planning branch: docs/runengpu-g2-capabilities-resources
```

The planning branch existed at the same commit and contained no prior changes. This investigation changes no Rust behavior and does not create `dornglut/runen-gpu`.

The current implementation contains enough evidence to bind G2, but it must not be copied as the target API. The present capability profile is renderer-shaped, the resource model combines unrelated dimensions, the typed-data layer assumes one broad raw representation, direct ECS projection lives inside the framework-facing facade, and WGPU realization is coupled to Runenwerk and Winit. These are decomposition inputs.

## Binding ownership

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image planning
        -> RunenGPU generic GPU execution
            -> WGPU backend
```

RunenGPU owns normalized capabilities, logical resource descriptions, typed logical handles, prepared GPU-data contracts, generic work inputs, deterministic validation, backend admission and realization, execution lifecycle facts, readback facts, and low-level surface facts.

RunenRender owns views, logical targets, materials, visibility, lighting, transport, reconstruction, overlays, image-formation semantics, and lowering render plans into generic GPU work.

Runenwerk owns ECS and domain projection, scheduling, windows, shader-file discovery, hot reload, fixed-time policy, capture and artifact policy, offline batch jobs, image encoding, external video-encoder integration, recovery, and diagnostics presentation.

## Investigation method

The census read declarations and representative direct and transitive consumers rather than classifying filenames alone. It reviewed:

- repository workflow authority in `AGENTS.md` and `TESTING.md`;
- the canonical roadmap and active-work state;
- ADR 0015;
- the RunenGPU architecture and GPU/renderer decomposition plan;
- S0 inventory, file disposition, and identity/lifecycle findings;
- the G1A specification and closeout;
- industry and public-API ergonomics reports;
- the accepted proof workload strategy;
- issues `#167` and `#168`, including their accepted planning comments;
- current capability, resource, typed-data, handle, RenderFlow, graph, backend, primitive, capture, example, test, and macro source.

## Current declaration census

### Existing future-transferable GPU boundary

| File | Current declaration | Verdict |
|---|---|---|
| `engine/src/plugins/gpu/api/work_resource_id.rs` | owner-scoped `GpuWorkResourceId` and allocator | Retain as G1A authority. Typed G2 handles contain this logical identity without exposing raw construction. |
| `engine/src/plugins/gpu/api/mod.rs` | G1A exports | Extend with G2 modules. |
| `engine/src/plugins/gpu/mod.rs` | future-transferable module and import guards | Extend dependency guards to every G2 module. |

G1A is complete. The crate-private bridge that seeds the allocator from `RenderFlowId` is temporary and is removed through G3/G4 when GPU-owned work/context authority exists.

### Capabilities

| File | Current declaration or use | Current owner | Target disposition |
|---|---|---|---|
| `engine/src/plugins/render/graph/capabilities.rs` | `RenderBackendCapabilityProfile`, render-pass booleans, render-format sets, hard-coded `runtime_default`, compiled-flow validation | mixed renderer/GPU | Replace GPU facts and requirements with normalized RunenGPU contracts; retain render-policy selection above them. |
| `engine/src/plugins/render/backend/device.rs` | adapter feature query, timestamp feature request, default limits, device and queue request | mixed backend/product default | G4 RunenGPU WGPU realization; G2 only binds normalized facts and requirements. |
| `engine/src/plugins/render/backend/formats.rs` | WGPU surface-format choice | mixed surface/product policy | G7 low-level format facts plus Runenwerk/RunenRender policy; not G2 implementation. |
| `engine/src/plugins/render/backend/wgpu_ctx.rs` | instance, adapter, public device/queue, Winit window, primary surface, timing facts | mixed GPU/host/surface | Split in G4/G7. G2 inventories it but creates no context. |
| `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs` | timestamp-query consumption and timing realization | mixed backend/diagnostics | G4/G5 realization, RunenGPU timing facts, Runenwerk presentation. |

The current profile is not authoritative because:

- `supports_fullscreen`, `supports_builtin_ui_composite`, and pass kinds are renderer or product vocabulary;
- the string profile key is diagnostic, not capability authority;
- `runtime_default()` claims support without querying an admitted adapter;
- format support is represented through render target types;
- required, preferred, and disabled behavior is not represented;
- fallback and degradation are not explicit.

### Resources

| File | Current declaration or use | Defect | Target disposition |
|---|---|---|---|
| `engine/src/plugins/render/resource/descriptors.rs` | buffer, texture, target, history, alias, and import variants | kind is combined with render role; `TypeId` and `GpuParams` determine size; surface policy is embedded | Replace with kind-specific backend-neutral RunenGPU descriptors and higher-level RunenRender target/history declarations. |
| `engine/src/plugins/render/resource/lifetime.rs` | `Imported`, `Persistent`, `Transient` | ownership and lifetime are combined | Replace with independent lifetime and ownership dimensions. |
| `engine/src/plugins/render/resource/import.rs` | surface/history/external import semantics | imported ownership, surface acquisition, history semantics, and external compatibility are combined | Split into RunenGPU import facts, G7 surface acquisition, and RunenRender history meaning. |
| `engine/src/plugins/render/resource/usages.rs` | read/write plus sampled/storage/target/vertex/index/instance/indirect kinds | access, binding role, render role, and transfer intent are incomplete and combined | G2 binds permitted uses; G3 binds per-work access and hazard semantics. |
| `engine/src/plugins/render/api/handles.rs` | typed uniform/storage handles and a named double buffer | logical typing is useful; handles are not RAII backend ownership | Recreate typed logical handles in RunenGPU; backend retirement semantics arrive with G5. Double-buffer policy belongs to the contributing workload/adapter. |
| `engine/src/plugins/render/backend/resource_allocator.rs` | ECS resource maps, labels, pass-owned transient claims | not an allocator or RAII registry; imports ECS and render pass IDs | Replace incrementally through G3-G5; do not move. |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs` and `runtime_resources/**` | WGPU realization and resolution of current descriptors | mixed renderer, backend, product target, and diagnostics authority | Redesign in G4/G5 after G2/G3 contracts. |
| `engine/src/plugins/render/graph/resource_lifetimes.rs` | current lifetime validation | transferable invariant mixed with old descriptor model | Re-express on G2 descriptors in G3; delete old authority in that slice. |

The initial resource kinds justified by current consumers are:

```text
buffer
texture
texture view
sampler
query set
```

No separate `Readback`, `Exported`, `ExternalResource`, or `SurfaceImage` resource kind is accepted. Readback and export are operations or relationships. Surface acquisition is ownership. Import is ownership. None is a lifetime.

The independent dimensions are:

```text
kind
    buffer, texture, texture view, sampler, query set

lifetime
    transient, retained

ownership
    RunenGPU-owned, imported, surface-acquired

transfer and observation
    initial data, update/upload, copy, readback request, export relationship

reconstruction
    source-backed, externally reconstructed, non-reconstructable
```

Current evidence also justifies separate memory intent for ordinary device use, host-to-device transfer, and device-to-host observation. It does not justify public backend heap or residency classes.

### Typed data and parameter projection

| File | Current declaration or use | Verdict |
|---|---|---|
| `engine/src/plugins/render/params/gpu_params.rs` | one `GpuParams::Raw`, marker `GpuUniform`, marker `GpuStorage` | One raw representation cannot imply valid uniform, storage, vertex, indirect, transfer, and readback layouts. Transitional only. |
| `engine/src/plugins/render/params/gpu_value.rs` | scalar/vector conversion and uniform ABI alignment | Useful evidence, not universal layout authority. Silent out-of-range byte writes are not accepted. |
| `engine_render_macros/src/lib.rs` | `GpuUniform` and `GpuStorage` derives hard-coded to `engine::plugins::render` | G4 decides replacement or deletion after compile-pass/fail and layout proof. Do not move in G2. |
| `engine/src/plugins/render/api/bindings.rs` | `TypeId`, `Any`, ECS projection, prepared uniform bytes, fixed-step uniform | Projection belongs to Runenwerk adapters. Prepared bytes and layout facts cross into RunenGPU. |
| `engine/src/plugins/render/api/flow.rs` | `with_state`, `project_uniforms`, resource registration | Split. ECS state and projection remain outside RunenGPU; generic descriptor/handle authority moves. |
| `engine/src/plugins/render/api/passes.rs` | `uniform_from_state`, `dispatch_from_state`, surface-dependent projection | Runenwerk adapter authority; not RunenGPU API. |

Binding boundary:

```text
Runenwerk or source-domain adapter
    ECS or domain state
        -> explicit prepared value or prepared bytes
            -> RunenGPU descriptor and later upload/update contract
```

G2 must distinguish at least:

```text
uniform data
storage data
vertex data
indirect data
transfer data
readback decoding
```

The distinction is semantic and typed. G4 binds backend layout, binding-interface realization, and derive/macro behavior. G5 performs uploads, updates, staging, completion, and readback. A `TypeId` or type name may support in-process diagnostics and adapter checks, but it is not a stable layout, binding, persistence, wire, replay, cache, or shader-interface key.

### RenderFlow

`RenderFlow` is a transitional combined facade. It is neither moved nor wrapped wholesale.

| Current surface | Target owner or action |
|---|---|
| `RenderFlow` resource identity and generic buffer/texture descriptions | RunenGPU, replaced incrementally |
| `with_state`, `uniform_from_state*`, `dispatch_from_state`, `project_uniforms` | Runenwerk/source-domain adapter |
| `with_surface_color`, `with_surface_depth`, color/depth target aliases, history texture | RunenRender semantic targets plus Runenwerk/G7 surface adapter |
| `shader_asset` | Runenwerk source discovery/revision; G4 receives an admitted shader source/interface key |
| `compute_pass`, generic copy, resource binding | lexical generic RunenGPU work authoring in G3/G4 |
| fullscreen/graphics/procedural render meaning | RunenRender lowering into generic work |
| `with_builtin_ui`, built-in UI composite | Runenwerk UI adapter and later RunenRender overlay |
| `fixed_step_region` and fixed-time uniforms | Runenwerk scheduling adapter |
| string maps for resources and passes | diagnostic labels only; typed references replace authority |
| broad `.depends_on` chains | infer data order in G3; keep explicit order only for non-data constraints |
| repeated `.finish()` builders | replace with lexical or closure scope |
| explicit `validate()` common path | ordinary `submit` validates automatically; advanced `prepare` exposes the same authority |
| `RenderFlowId`-seeded resource scope | temporary bridge removed through G3/G4 |

Useful readability is retained, but mixed ownership is not.

## Consumer census

### Direct declaration and validation consumers

```text
engine/src/plugins/render/api/bindings.rs
engine/src/plugins/render/api/dispatch.rs
engine/src/plugins/render/api/flow.rs
engine/src/plugins/render/api/handles.rs
engine/src/plugins/render/api/passes.rs
engine/src/plugins/render/api/resources.rs
engine/src/plugins/render/graph/capabilities.rs
engine/src/plugins/render/graph/diagnostics.rs
engine/src/plugins/render/graph/execution_plan.rs
engine/src/plugins/render/graph/flow_graph.rs
engine/src/plugins/render/graph/merge.rs
engine/src/plugins/render/graph/pass_graph.rs
engine/src/plugins/render/graph/planning.rs
engine/src/plugins/render/graph/prepared_validation.rs
engine/src/plugins/render/graph/resource_graph.rs
engine/src/plugins/render/graph/resource_lifetimes.rs
engine/src/plugins/render/graph/validation.rs
engine/src/plugins/render/resource/descriptors.rs
engine/src/plugins/render/resource/import.rs
engine/src/plugins/render/resource/lifetime.rs
engine/src/plugins/render/resource/usages.rs
```

### Backend and runtime consumers

```text
engine/src/plugins/render/backend/device.rs
engine/src/plugins/render/backend/execution.rs
engine/src/plugins/render/backend/formats.rs
engine/src/plugins/render/backend/pipeline_cache.rs
engine/src/plugins/render/backend/resource_allocator.rs
engine/src/plugins/render/backend/surface.rs
engine/src/plugins/render/backend/wgpu_ctx.rs
engine/src/plugins/render/renderer/frame_bindings.rs
engine/src/plugins/render/renderer/prepare.rs
engine/src/plugins/render/renderer/render_flow/bindings.rs
engine/src/plugins/render/renderer/render_flow/execute.rs
engine/src/plugins/render/renderer/render_flow/execute_passes.rs
engine/src/plugins/render/renderer/render_flow/gpu_timing.rs
engine/src/plugins/render/renderer/render_flow/preflight_cache.rs
engine/src/plugins/render/renderer/render_flow/runtime_resources.rs
engine/src/plugins/render/renderer/render_flow/runtime_resources/inspect.rs
engine/src/plugins/render/renderer/render_flow/runtime_resources/realize.rs
engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs
engine/src/plugins/render/runtime/dynamic_texture_uploads.rs
engine/src/plugins/render/runtime/frame_prepare.rs
engine/src/plugins/render/runtime/frame_submit.rs
```

### Applications and transitive product callers

```text
apps/runenwerk_draw/src/runtime/app.rs
apps/runenwerk_draw/src/runtime/gpu_ink.rs
apps/runenwerk_editor/src/runtime/app.rs
apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs
apps/runenwerk_draw/tests/app_shell.rs
apps/runenwerk_editor/tests/region_compass_visual_capture.rs
apps/runenwerk_editor/tests/startup_render_smoke.rs
apps/runenwerk_editor/tests/viewport_architecture_guards.rs
apps/runenwerk_editor/tests/viewport_branch_truth_smoke.rs
apps/runenwerk_editor/tests/viewport_gpu_truth_smoke.rs
```

### Generic GPU primitive consumers

```text
engine/src/plugins/render/gpu_primitives/compaction.rs
engine/src/plugins/render/gpu_primitives/counters.rs
engine/src/plugins/render/gpu_primitives/draw_args.rs
engine/src/plugins/render/gpu_primitives/plan.rs
engine/src/plugins/render/gpu_primitives/scan.rs
engine/src/plugins/render/procedural/population/uniform_grid.rs
```

### Examples

```text
engine/examples/game_of_life_sdf/rendering/graph.rs
engine/examples/game_of_life_sdf/rendering/state.rs
engine/examples/boids_render_flow/rendering/graph.rs
engine/examples/procedural_sky_sdf_terrain/rendering/graph.rs
engine/examples/render_flow_fullscreen_minimal/main.rs
engine/examples/render_flow_postprocess_compositor/main.rs
engine/examples/sdf_render_flow/rendering/graph.rs
engine/examples/render_flow_debug_inspect/main.rs
```

### Tests and benchmark pressure

```text
engine/benches/render_flow_planning.rs
engine/tests/render_dynamic_targets.rs
engine/tests/render_flow_v2.rs
engine/tests/render_import_contract.rs
engine/tests/render_resource_model.rs
engine/tests/render_runtime_inspect.rs
engine/tests/procedural_instance.rs
```

The complete historical S0 test/example/benchmark census remains authoritative for the wider migration surface. The listed files are the direct G2 pressure and required implementation review set.

### Diagnostics, capture, and artifacts

```text
engine/src/plugins/render/inspect/artifacts.rs
engine/src/plugins/render/inspect/capture.rs
engine/src/plugins/render/inspect/graph_dump.rs
engine/src/plugins/render/inspect/prepared_frame.rs
engine/src/plugins/render/inspect/query_snapshot.rs
engine/src/plugins/render/inspect/resource_inspector.rs
engine/src/plugins/render/inspect/texture_view.rs
engine/src/plugins/render/inspect/timings.rs
engine/src/plugins/render/renderer/render_flow/capture.rs
```

Current capture evidence includes WGPU texture-to-buffer copies, 256-byte row alignment, row-padding removal, RGBA/BGRA normalization, mapping, PNG/manifest handoff, and terminal diagnostics. The current path blocks on `device.poll`; it is evidence, not the accepted G5 asynchronous readback design.

### Stable-format census

No current `GpuWorkResourceId`, typed resource handle, capability profile, or render resource descriptor is accepted as a persisted, replay, network, wire, cache, or cross-process identity. `TypeId` and type names are process-local diagnostics. Capture identities and artifact manifests are Runenwerk product formats and do not make GPU logical resource IDs stable.

No stable-format stop condition is triggered.

## Capability decision

G2 selects the smallest requirement vocabulary that represents current consumers:

```text
Required
Preferred { degradation }
Disabled
```

An unmentioned capability is irrelevant. `Optional` is therefore not a fourth state. `Disabled` is an explicit admission constraint, not a synonym for unsupported.

Capability profiles are convenience recipes that produce ordinary requirement entries. They are not a second authority.

Initial normalized feature facts justified by current consumers are:

```text
compute
render pipelines
copy operations
indirect draw
storage textures
depth attachments
timestamp queries
presentation
```

Initial normalized limits justified by current validation are:

```text
maximum uniform-buffer binding size
maximum storage-buffer binding size
maximum color attachments
maximum vertex buffers
maximum binding entries per group
```

Initial format facts cover only formats currently represented by the renderer and capture path. G2 does not claim universal format support. Subgroups, external-resource interop, hardware ray queries, hardware ray pipelines, sparse resources, mesh shaders, video, and multiple hardware queues remain deferred because current consumers do not require them.

## Resource decision

G2 descriptors are immutable values. Construction validates dimensions, sizes, permitted uses, ownership combinations, initial data length, view-parent relationships, reconstruction facts, and provenance before a value can enter work authoring.

Owned, imported, and surface-acquired resources use different constructors so invalid combinations are difficult to express. Examples:

- imported resources cannot claim RunenGPU-owned initial allocation;
- surface-acquired ownership is valid only for texture/view contracts and has no retained lifetime;
- non-reconstructable retained resources require explicit acceptance and cannot be silently recreated after device loss;
- initial buffer bytes must exactly match the declared initialized range;
- a texture view references a typed texture handle and a validated subresource description;
- labels are retained for diagnostics but never resolve bindings or identity.

Transfer, update, copy, readback, and export execution remain G5. G2 binds the descriptor and relationship vocabulary only.

## Typed-data decision

G2 introduces an opaque prepared-data boundary parameterized by semantic purpose. It records checked byte length, alignment, stride, element count, and provenance. It does not infer backend layout from arbitrary Rust memory.

The initial purpose classes are:

```text
uniform
storage
vertex
indirect
transfer
```

Readback is represented by an explicit decoding contract rather than pretending mapped bytes are the same as an upload representation.

Current `GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField`, and both derives remain transitional Runenwerk/render adapter mechanisms until G4 decides concrete layout and macro realization. G2 must remove them from new RunenGPU public signatures and from new descriptor size authority. G2 may use explicit adapters at the old boundary; it may not add a universal replacement derive.

## Public API pressure

The intended common path remains:

```rust
let simulation = simulation.gpu_work(&gpu, &state)?;
let rendering = renderer.gpu_work(&gpu, &scene, request)?;
let submission = gpu.submit("frame 42", [simulation, rendering])?;
```

The advanced path remains:

```rust
let prepared = gpu.prepare("frame 42", [simulation, rendering])?;
inspect(prepared.diagnostics());
let submission = gpu.submit_prepared(prepared)?;
```

G2 binds the resource/capability/data vocabulary used by both paths. G3-G5 implement generic work, preparation, admission, realization, and execution. The graph is the shared internal correctness authority, not mandatory common-path ceremony.

Required error presentation includes:

```text
operation label
resource label and typed identity
machine-readable category
cause
provenance
correction
```

## Proof workload binding

### G5 deterministic compute conformance

Current authority:

```text
engine/src/plugins/render/gpu_primitives/scan.rs
engine/src/plugins/render/gpu_primitives/plan.rs
engine/src/plugins/render/gpu_primitives/counters.rs
engine/src/plugins/render/gpu_primitives/compaction.rs
engine/src/plugins/render/gpu_primitives/draw_args.rs
assets/shaders/gpu_primitive_counter_reset.wgsl
assets/shaders/gpu_primitive_prefix_scan.wgsl
assets/shaders/gpu_primitive_prefix_scan_apply_offsets.wgsl
assets/shaders/gpu_primitive_u32_scatter.wgsl
assets/shaders/gpu_primitive_indirect_draw_args.wgsl
assets/shaders/gpu_primitive_indexed_indirect_draw_args.wgsl
```

Required fixture:

```text
input: 4,097 u32 values, every value equal to 1
workgroup size evidence: 64
exclusive expected output: output[i] == i for 0 <= i < 4,097
inclusive expected output: output[i] == i + 1 for 0 <= i < 4,097
expected total: 4,097
```

This crosses workgroup boundaries and requires more than one scan hierarchy level. The test compares every output element, not only a digest. It proves upload, temporary multi-level storage, multiple dispatches, completion, asynchronous readback, and exact integer results without renderer, ECS, surface, window, or product types.

Counter reset, scatter/compaction, and indirect-argument generation remain intended RunenGPU authority and are separately covered by focused exact tests before G6 composition.

### G5 stateful integration

Current authority:

```text
engine/examples/game_of_life_sdf/rendering/state.rs
engine/examples/game_of_life_sdf/rendering/graph.rs
assets/shaders/game_of_life_compute.wgsl
assets/shaders/game_of_life_compose.wgsl
```

Headless G5 uses only the state preparation and compute algorithm. Fullscreen composition, UI, surface, and product timing are excluded.

Bound fixture derived from the current compute shader:

```text
grid: 160 x 90
seed: 0xC0FF_EE11
evolution steps after seeding: 16
boundary: toroidal
expected live cells: 2,063
checksum: FNV-1a-64 over little-endian u32 cell values
expected checksum: 0xBD710B88594CD584
selected cells:
    (0, 0) = 1
    (10, 12) = 1
    (11, 12) = 1
    (12, 12) = 0
    (24, 12) = 0
    (80, 45) = 0
    (159, 89) = 1
```

A CPU reference implementation in the test owns the oracle and also compares the full grid. Simulation state is prepared outside RunenGPU. The proof uses ping-pong buffers, repeated submissions, completion, and asynchronous readback.

### G5 conditional texture proof

When storage-texture readback is in accepted G5 scope, add an integer compute-to-texture fixture under the G5 conformance test tree. It writes a deterministic `Rgba8Unorm` or `R32Uint` pattern, copies to a buffer, removes backend row padding, validates every selected texel and total byte length, and hands normalized bytes to a Runenwerk test/tool adapter for PNG encoding.

The current row-padding and format evidence is `engine/src/plugins/render/renderer/render_flow/capture.rs`. That file is not moved wholesale.

### G6 graphics conformance

Current starting point:

```text
engine/examples/render_flow_fullscreen_minimal/main.rs
```

The current example proves planning only and is insufficient. G6 replaces it with an offscreen test fixture and a dedicated known-pattern shader. The test has no surface, performs a known clear plus draw, copies the texture to a buffer, normalizes padding, and asserts selected pixels. Exact values are required for integer-friendly regions; documented channel tolerance is permitted only where normalized rasterization or format conversion requires it.

### G6 GPU-driven composition

Current authority:

```text
engine/src/plugins/render/gpu_primitives/counters.rs
engine/src/plugins/render/gpu_primitives/scan.rs
engine/src/plugins/render/gpu_primitives/compaction.rs
engine/src/plugins/render/gpu_primitives/draw_args.rs
engine/src/plugins/render/gpu_primitives/plan.rs
```

Compute generates count, compacted data, or indirect arguments. Graphics consumes that resource through one shared context. Ordering is inferred from resource access. Validation checks structure, generated argument values, and selected pixels.

### G6 representative showcase

Current authority:

```text
engine/examples/boids_render_flow/rendering/graph.rs
assets/shaders/boids_compute.wgsl
assets/shaders/boids_compose.wgsl
```

Offscreen boids proves shared compute/graphics use, bounded-grid count/scan/scatter, ping-pong state, indirect or generated draw data, and artifact production. Acceptance uses pass/resource structure, agent-count invariants, finite values, bounded position/range checks, overflow checks, and successful image generation. It does not require exact cross-backend boid state or pixel equality.

### G7 surface proof

G7 reuses the accepted G6 known-pattern and boids workloads. It proves presentation, resize, reconfiguration, surface generations, multi-surface behavior where supported, thread affinity, and structured device/surface outcomes. It creates no parallel surface-only execution architecture.

### First RunenRender proof

Current authority:

```text
engine/examples/procedural_sky_sdf_terrain/rendering/graph.rs
assets/shaders/procedural_sky_sdf_terrain_compose.wgsl
```

This becomes the first semantic image-formation proof after standalone RunenGPU is accepted. Boids follows as simulation-to-render integration. The history flow in `engine/examples/sdf_render_flow/rendering/graph.rs` and `assets/shaders/sdf_render_flow_3d_compose.wgsl` remains a later temporal/history ownership proof.

### Offline output

Preferred order:

1. Game of Life PNG sequence after G5.
2. Offscreen boids PNG sequence after G6.
3. Procedural sky, SDF, or scene sequences after the corresponding RunenRender phase.

Runenwerk owns fixed output clock, seed/job configuration, bounded in-flight readbacks, ordered filenames, manifests, retries, failure policy, PNG/EXR encoding, and FFmpeg or other codec integration. RunenGPU owns completion/readback facts. RunenRender owns image formation. Neither framework owns MP4/WebM codecs.

## Alternative workload ranking

| Candidate | What it proves | Determinism | Current reuse | Complexity | Portability | Classification |
|---|---|---|---|---|---|---|
| Vector addition | minimal upload/dispatch/readback | exact | none | very low | high | tutorial only; not a gate |
| Reduction | multi-stage integer compute | exact when integer | primitive-adjacent | low | high | optional G5 conformance |
| Histogram | atomics and readback | exact for integer bins | none | low-medium | high | optional G5 conformance |
| Convolution | texture upload/process/readback | exact for integer kernel; tolerant for float | postprocess candidate | medium | high | optional G5/G6 integration |
| Sobel filtering | texture processing and edges | exact or bounded by format | no accepted executable proof | medium | high | optional integration |
| Mandelbrot or Julia | compute-to-image | boundary pixels vary with float | none | medium | medium | showcase, not conformance |
| Reaction-diffusion | repeated texture compute | float-tolerant | none | medium | medium | later showcase |
| N-body | compute/render composition | float-tolerant | none | medium-high | medium | defer; boids has stronger reuse |
| Fluids or smoke | many-pass simulation and stress | low | none | high | medium-low | deferred stress workload |
| FFT ocean | spectral compute/render | low | none | very high | medium-low | deferred RunenRender/stress work |
| Marching cubes | compute generation and draw | count may be exact; geometry backend-sensitive | no accepted implementation | high | medium | later domain-to-GPU integration |
| GPU procedural mesh generation | compute-to-graphics | workload-dependent | procedural concepts only | high | medium | later integration |
| SDF rendering | semantic image formation | float-tolerant | strong current example | medium | high | first RunenRender work, not GPU conformance |
| Path tracing | transport, accumulation, advanced resources | statistical/tolerant | no accepted execution proof | very high | medium-low | deferred advanced RunenRender work |

No candidate is added merely because another engine uses it.

## Migration and deletion order

1. Add G2 capability, resource, typed-handle, prepared-data, error, and dependency-guard modules under `engine/src/plugins/gpu`.
2. Add focused tests for descriptor construction, invalid combinations, capability requirement merging, typed-handle separation, prepared-data metadata, and human-readable errors.
3. Introduce explicit Runenwerk/render adapters from current `GpuParams` output and render resource intent into the new contracts.
4. Migrate capability consumers away from `RenderBackendCapabilityProfile` where G2 semantics are sufficient.
5. Migrate generic resource descriptors and handles away from render-owned declaration authority.
6. Keep render target, history, surface, fixed-step, ECS projection, shader-file, and product policy in their owners.
7. Delete replaced capability/resource declaration authority in the same implementation slice; do not leave aliases or forwarding modules.
8. Leave access/hazard/work graph, WGPU realization, execution, readback, and surfaces for G3-G7, with explicit adapters only where required by the current runtime.

Exact deletion candidates after all current consumers migrate:

```text
RenderBackendCapabilityProfile and RenderBackendCapabilityInspection
old capability default/test constructors
ResourceLifetime::Imported/Persistent/Transient authority
RenderResourceDescriptor as generic GPU resource authority
ImportedResourceKind and compatibility import helpers as generic authority
uniform/storage descriptor size authority based on TypeId and GpuParams::Raw
render-owned UniformHandle and StorageArrayHandle authority where replaced
```

Render-semantic target/history aliases may remain temporarily only in RunenRender/Runenwerk-facing adapters; they must not be re-exported as RunenGPU descriptors.

## Allowed dependencies

The future-transferable G2 boundary may depend on:

```text
Rust standard library
small no-std-compatible collection/value utilities already accepted by the workspace
thiserror only if already accepted for structured errors
bytemuck only behind explicit prepared-byte constructors whose safety contract is proven
```

The implementation issue must justify every dependency. No new package is presumed.

## Forbidden dependencies

New RunenGPU G2 modules must not import or expose:

```text
Runenwerk application or engine lifecycle
RunenRender
ECS
UI
SDF or world domains
Winit
WGPU public types
scene, view, material, lighting, transport, overlay, or presentation semantics
shader filesystem paths or hot reload
capture, PNG, EXR, FFmpeg, or artifact policy
fixed-time scheduling
editor or product types
```

## Risks

| Risk | Required control |
|---|---|
| Recreating WGPU one-for-one | Admit only normalized semantics with current consumer value. |
| Treating labels as identity | Private typed IDs and handles; labels diagnostic only. |
| Universal data derive | Separate prepared-data purposes; G4 layout proof; compile-pass/fail tests. |
| Keeping duplicate descriptor authority | Migrate and delete in the same G2 implementation. |
| Moving RenderFlow wholesale | Use the explicit responsibility table and adapters. |
| Premature execution contracts | G2 creates no device, queue, pipeline, submission, readback, surface, or retirement implementation. |
| False portability claims | WGPU is the first backend; no second backend or universal shader IR. |
| Visual tests masking correctness | Preserve exact prefix-scan and Game of Life oracles. |
| Performance thresholds without environment | Record measurements only; no acceptance threshold until an environment is bound. |

## Stop conditions for implementation

Stop and return to planning if implementation discovers:

- a stable persisted, replay, network, wire, cache, or external format containing an affected identity or descriptor;
- an owner split that requires a new ADR;
- a need to implement G3 hazards/work graphs, G4 devices/pipelines, G5 execution/readback, or G7 surfaces to make G2 types coherent;
- a need for compatibility aliases, forwarding modules, or duplicate descriptor authority;
- an inability to establish typed-layout safety without claiming arbitrary Rust layout is GPU-safe;
- a materially incomplete direct or transitive consumer census;
- a proof workload that requires unrelated renderer/domain architecture as a G2 gate;
- an unrelated failure of `cargo validate` on current `main`.

## Unresolved evidence deliberately deferred

G2 does not resolve:

- exact WGPU feature/limit mapping;
- adapter selection and context admission execution;
- shader interface reflection and validated binding-key realization;
- hazard ranges, texture subresources, scheduling, or work-graph internals;
- staging implementation, partial update encoding, asynchronous map polling, cancellation, or retirement;
- offscreen pipeline and raster tolerances;
- surface raw-handle safety, generations, thread affinity, or device-loss recovery;
- macro package retention;
- performance targets;
- external repository creation.

These are assigned to G3-G8 and GX. Their deferral is bounded ownership, not an architectural gap.

## Final verdict

The G2 implementation may be specified as one bounded internal cutover. It must create normalized capability and logical resource authority under `engine::plugins::gpu`, establish the prepared-data seam, migrate the current declaration consumers that G2 replaces, and delete the replaced authority without implementing backend execution.
