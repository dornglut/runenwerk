---
title: Viewport Dynamic Product Target Allocation Design
description: V5 implementation design for replacing shared static viewport render products with viewport-scoped dynamic product targets.
status: implemented
owner: editor
layer: engine
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ./render-product-surface-foundation-bundle-design.md
  - ./workspace-viewport-expression-upgrade-design.md
  - ./editor-ui-workspace-tool-surface-architecture.md
related_roadmaps:
  - ../../apps/runenwerk-editor/viewport-expression-implementation-roadmap.md
related:
  - ../../apps/runenwerk-editor/current-architecture.md
---

# Viewport Dynamic Product Target Allocation Design

## Status

Implemented V5/V6 cutover design for the viewport expression architecture roadmap.

V5 made viewport product ownership real by replacing shared static scene, picking, and overlay render resources with viewport-scoped dynamic targets. V6 made the scene, picking, and overlay producer render every `ViewportRenderJob` into those targets through prepared render views and flow invocations.

This design was implemented as part of `docs-site/src/content/docs/design/implemented/render-product-surface-foundation-bundle-design.md`. Dynamic targets are allocated, writable pass attachments, sampleable UI resources when descriptors allow it, and scoped by viewport/product/slot ownership.

## Goal

Create the long-term render target contract for viewport expression products:

```text
ViewportRenderJob
  -> ViewportProductTargetRegistryResource
  -> RenderDynamicTextureTargetRequestRegistryResource
  -> Renderer dynamic texture target cache
  -> ViewportSurfaceBindingRegistry
  -> UI ViewportSurfaceEmbed
```

The final V5 system must guarantee that a visible viewport's presentation surface resolves to that viewport's own product target, not to a global scene texture, a first observed viewport, a flow-sized compatibility target, or a shader-side sub-rectangle inside another target.

## Non-Negotiable Outcomes

- Every visible viewport has separate target keys for scene color, picking ids, overlay, and later depth or diagnostic products.
- Target identity is derived from explicit viewport/product ownership, not from static render-flow labels.
- Render target allocation lives in the runtime and engine layer; expression product semantics remain engine-agnostic.
- UI embeds bind to an explicit source describing which target to sample.
- The renderer never falls back to another viewport's product if a binding is missing or stale.
- Resize reallocates only the affected viewport targets.
- Closed viewport instances retire their product targets through an explicit lifecycle path.
- V5 must not introduce one render flow per viewport as the final mechanism.

## Prerequisites

V5 depends on these earlier roadmap contracts:

- V1: `apps/runenwerk_editor/src/runtime/viewport/instance_registry.rs` owns explicit viewport instances.
- V2: viewport lifecycle runs before shell projection and before frame submission.
- V3: `apps/runenwerk_editor/src/runtime/viewport/render_state.rs::ViewportRenderStateResource` owns viewport-local dimensions, scale policy, and camera state.
- V4: `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::ViewportRenderJobResource` exposes one render job per visible viewport.

The preferred active path is to implement V5 with the larger render product surface foundation bundle and land V5+V6 as one user-visible cutover. A standalone V5 infrastructure patch is acceptable only if it remains internal, guarded, and does not introduce a visible path that copies or reuses the old global product target.

## Current Constraints

The current render path is static:

- `apps/runenwerk_editor/src/runtime/app.rs::register_editor_render_flow` registers one `RenderFlow` with static color targets.
- `apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs` exposes `VIEWPORT_RESOURCE_SCENE_COLOR`, `VIEWPORT_RESOURCE_PICKING_IDS`, and `VIEWPORT_RESOURCE_OVERLAY`.
- `engine/src/plugins/render/api/flow.rs::RenderFlow::with_color_target` declares static flow-owned resources.
- `engine/src/plugins/render/renderer/render_flow/runtime_resources/realize.rs::FlowRuntimeResources::realize_for_frame` realizes flow resources at swapchain size and surface format.
- `domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs::ViewportSurfaceBinding` contains only `flow_id` and `resource_id`.
- `engine/src/plugins/render/renderer/setup.rs::encode_ui_pass` resolves UI viewport embeds through the current flow's runtime resource map.

Those constraints mean distinct viewport ids can still sample the same product texture. V5 removes that coupling.

## Alternatives Considered

### One Render Flow Per Viewport

Rejected.

Creating one `RenderFlow` per viewport would make static labels distinct, but it would move viewport lifecycle into flow registry churn. It would duplicate compiled-flow state, complicate pipeline and resource caches, and make dynamic product families depend on shell surface count. It also does not solve the expression product contract because product ownership would be encoded indirectly through flow instances.

### Static Labels With Viewport Id Suffixes

Rejected.

Generating labels such as `editor.viewport.42.scene_color` keeps the old static-resource model and makes flow compilation depend on dynamic viewport instances. It avoids the immediate shared texture bug but leaves product lifecycle, allocation policy, and UI binding source semantics implicit.

### Store Backend Texture Handles In UI Data

Rejected.

`domain/ui/ui_render_data` is a retained UI data crate. It must stay backend-neutral and serializable enough for UI projection, tests, and future non-WGPU render backends. Passing WGPU texture handles into UI bindings would leak renderer internals across the domain boundary.

### Chosen: Dynamic Target Requests And Opaque UI Binding Sources

Accepted.

The app owns viewport product target identity and requests concrete render textures from the engine through a dynamic target request registry. The renderer owns actual backend texture allocation and publishing. UI embeds reference an opaque binding source that can point either to a legacy flow resource or to a dynamic texture target.

This keeps the final contracts separated:

- app viewport runtime owns `ViewportId`, product selection, product target records, lifecycle, and presentation slots;
- engine render runtime owns dynamic texture allocation, cache invalidation, backend usage flags, and texture views;
- UI render data owns only stable binding source data for composition.

## Engine Contract

### Dynamic Target Descriptor Module

Add a render resource module:

```text
engine/src/plugins/render/resource/dynamic_target.rs
```

The module should define backend-neutral request data:

```rust
pub struct RenderDynamicTextureTargetKey {
    pub namespace: String,
    pub target_id: String,
}

pub enum RenderTextureTargetFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    R32Uint,
    Depth32Float,
}

pub struct RenderTextureTargetUsage {
    pub color_attachment: bool,
    pub depth_attachment: bool,
    pub sampled: bool,
    pub storage: bool,
    pub copy_src: bool,
    pub copy_dst: bool,
}

pub enum RenderTextureSampleMode {
    FilterableFloat,
    NonFilterableFloat,
    Uint,
    Depth,
    NotSampled,
}

pub enum RenderDynamicTextureRetention {
    RetainWhileRequested,
    RetainUntilViewportClose,
    RetainForFrames(u32),
}

pub struct RenderDynamicTextureTargetDescriptor {
    pub key: RenderDynamicTextureTargetKey,
    pub width: u32,
    pub height: u32,
    pub format: RenderTextureTargetFormat,
    pub usage: RenderTextureTargetUsage,
    pub sample_mode: RenderTextureSampleMode,
    pub retention: RenderDynamicTextureRetention,
}
```

Implementation may use bitflags for usage if that matches nearby render code better, but the public request must stay explicit and typed. Width and height must reject zero. Unsupported usage/format combinations must produce diagnostics instead of silently allocating a different format.

### Dynamic Target Request Resource

Add an engine runtime resource:

```text
engine/src/plugins/render/runtime/dynamic_targets.rs
```

Responsibilities:

- store the prepared-frame snapshot of requested dynamic texture targets;
- validate duplicate keys;
- expose target descriptors in deterministic key order;
- preserve request diagnostics for app/editor inspection.

The app writes requests before render preparation. `engine/src/plugins/render/runtime/frame_prepare.rs::frame_render_prepare_system` copies the request snapshot into `engine/src/plugins/render/frame/packet.rs::PreparedRenderFrame` so render submission operates on the same viewport/product set as the submitted UI frame.

### Renderer Dynamic Target Cache

Add renderer-owned allocation code:

```text
engine/src/plugins/render/renderer/dynamic_targets.rs
```

Extend `engine/src/plugins/render/renderer/mod.rs::Renderer` with a dynamic texture cache keyed by `RenderDynamicTextureTargetKey`.

Responsibilities:

- allocate WGPU textures for valid requested descriptors;
- reuse existing textures when key, dimensions, format, usage, and sample mode match;
- reallocate only the changed target when a descriptor changes;
- retain or retire unrequested targets according to `RenderDynamicTextureRetention`;
- expose texture views for render passes and UI embedding;
- keep generation counters for diagnostics and tests;
- preserve the previous valid texture if a later request for the same key is invalid.

No editor-specific `ViewportId` type should enter this module. The renderer sees only dynamic target keys and descriptors.

## UI Binding Contract

Update:

```text
domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs
```

Replace the old flow-resource binding shape with a dynamic-texture-only source:

```rust
pub enum ViewportSurfaceBindingSource {
    DynamicTexture {
        namespace: String,
        target_id: String,
    },
}

pub struct ViewportSurfaceBinding {
    pub source: ViewportSurfaceBindingSource,
}
```

Normal viewport presentation uses the dynamic constructor only:

```rust
impl ViewportSurfaceBinding {
    pub fn dynamic_texture(namespace: impl Into<String>, target_id: impl Into<String>) -> Self;
}
```

`engine/src/plugins/render/renderer/setup.rs::encode_ui_pass` resolves UI viewport embeds through the renderer dynamic target cache. Static flow-resource viewport embed compatibility was removed with the V5/V6 cutover so there is no second sampling path to preserve.

The UI bind-group cache key must include the full binding source, not only `resource_id`, because two dynamic targets may share a product slot name while belonging to different viewports.

Non-sampleable products, such as picking id targets, must not bind to normal visual viewport embeds. They can be read by picking systems or diagnostic views that explicitly support their format.

## App Contract

### Viewport Product Target Registry

Add:

```text
apps/runenwerk_editor/src/runtime/viewport/product_targets.rs
```

Responsibilities:

- map `(ViewportId, ViewportSurfacePresentationSlot, ExpressionProductId)` to a stable dynamic target key;
- record dimensions, format, sample mode, product freshness, generation, lifecycle status, and producer availability;
- request engine dynamic texture targets for all visible or retained viewport products;
- retire target records when viewport instances close;
- expose target records to the presentation resolver.

Suggested records:

```rust
pub struct ViewportProductTargetKey {
    pub viewport_id: ViewportId,
    pub slot: ViewportSurfacePresentationSlot,
    pub product_id: ExpressionProductId,
}

pub struct ViewportProductTargetRecord {
    pub key: ViewportProductTargetKey,
    pub namespace: String,
    pub target_id: String,
    pub width: u32,
    pub height: u32,
    pub format: ExpressionProductFormat,
    pub sample_mode: ViewportProductSampleMode,
    pub status: ViewportProductTargetStatus,
    pub generation: u64,
}

pub struct ViewportProductTargetRegistryResource {
    records: BTreeMap<ViewportProductTargetKey, ViewportProductTargetRecord>,
}
```

The target id should be deterministic and stable for the lifetime of the viewport instance, for example:

```text
editor.viewport.{viewport_id}.{product_id}.{slot}
```

This string is an opaque runtime address, not the semantic identity source. The semantic identity remains the typed key.

### Target Request System

Add a viewport runtime system:

```text
apps/runenwerk_editor/src/runtime/viewport/product_targets.rs::sync_viewport_product_targets_system
```

Inputs:

- `apps/runenwerk_editor/src/runtime/viewport/instance_registry.rs::ViewportInstanceRegistryResource`
- `apps/runenwerk_editor/src/runtime/viewport/render_state.rs::ViewportRenderStateResource`
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs::ViewportRenderJobResource`
- `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs::ViewportProductRegistryResource`
- `apps/runenwerk_editor/src/runtime/viewport/presentation_state.rs` once presentation state is split out

Outputs:

- `ViewportProductTargetRegistryResource`
- `engine/src/plugins/render/runtime/dynamic_targets.rs::RenderDynamicTextureTargetRequestRegistryResource`
- `apps/runenwerk_editor/src/runtime/viewport/surface_set.rs::ViewportSurfaceSetResource`

The system runs after render jobs are derived and before render frame preparation.

### Surface Set Cutover

Update:

```text
apps/runenwerk_editor/src/runtime/viewport/surface_set.rs
```

`ViewportSurfaceHandle` should carry a binding source or dynamic target address instead of static `flow_id` and `resource_id` fields. The surface set remains app-owned because it represents viewport presentation slots, not renderer internals.

Update:

```text
apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs::build_surface_binding_registry
```

The resolver maps each `(ViewportId, ViewportSurfacePresentationSlot)` to `ViewportSurfaceBinding::dynamic_texture(namespace, target_id)` when a valid target record exists. Missing records produce an explicit unavailable binding state or no binding for that slot; they must not fall back to another viewport or to the old static scene resource.

## Product Format Policy

Initial V5 product targets:

| Product | Format | Usage | UI sample mode |
| --- | --- | --- | --- |
| Scene color | `Rgba8Unorm` or `Rgba8UnormSrgb` by descriptor policy | color attachment, sampled, copy src | filterable float |
| Picking ids | `R32Uint` | color attachment, copy src | not sampled |
| Overlay | `Rgba8Unorm` or `Rgba8UnormSrgb` by descriptor policy | color attachment, sampled, copy src | filterable float |
| Depth | `Depth32Float` when introduced | depth attachment, copy src | not sampled by default |

Current product descriptors already carry expression-level format and presentation hints. V5 must translate those hints into explicit render target descriptors at the app/engine boundary. The renderer must not infer viewport product format from the swapchain surface format.

## Lifecycle Policy

### Visible Viewport

Visible viewport jobs request all required product targets. The renderer allocates or resizes those targets before producer execution.

### Hidden Retained Viewport

Hidden viewport instances may retain product targets if the viewport state marks them as retained. Retained targets are not rendered unless a job requests refresh, but the previous valid product remains available for fast remount or diagnostics.

### Closed Viewport

Closing a viewport retires its target records from `ViewportProductTargetRegistryResource`. The dynamic request registry stops requesting those keys. The renderer releases them according to the descriptor retention policy and cache retirement sweep.

### Invalid Dimensions

Zero-sized or otherwise invalid viewport dimensions do not allocate new targets. Existing valid targets remain available only if the viewport lifecycle policy allows stale presentation. The product record must mark the target stale or unavailable.

### Producer Failure

If a producer fails after a target exists, the target record preserves the last valid target and marks product freshness/producer health as stale. It must not bind another viewport's product or recreate a blank success-shaped product without diagnostics.

## Implementation Phases

### V5.1 - Engine Descriptor Types

Change:

- `engine/src/plugins/render/resource/dynamic_target.rs`
- `engine/src/plugins/render/resource/mod.rs`
- `engine/src/plugins/render/runtime/dynamic_targets.rs`
- `engine/src/plugins/render/runtime/mod.rs`

Exit criteria:

- dynamic target descriptors are typed and backend-neutral;
- zero-sized and invalid usage/format combinations produce diagnostics;
- app code can request targets without importing WGPU types.

Tests:

- descriptor validation accepts scene color, picking ids, overlay, and depth descriptors;
- descriptor validation rejects zero dimensions;
- descriptor validation rejects sampled UI binding for `R32Uint` unless a supported uint sampling path is explicitly added.

### V5.2 - Prepared Frame Snapshot

Change:

- `engine/src/plugins/render/frame/packet.rs::PreparedRenderFrame`
- `engine/src/plugins/render/runtime/frame_prepare.rs::frame_render_prepare_system`
- `engine/src/plugins/render/runtime/frame_submit.rs::frame_render_submit_system`

Exit criteria:

- every submitted frame carries the exact dynamic target request snapshot used for that frame;
- render submission does not read mutable app request state after preparation.

Tests:

- prepared frame snapshots dynamic target requests in deterministic order;
- late mutations to the request registry after prepare do not affect the submitted prepared frame.

### V5.3 - Renderer Dynamic Texture Cache

Change:

- `engine/src/plugins/render/renderer/mod.rs::Renderer`
- `engine/src/plugins/render/renderer/dynamic_targets.rs`
- `engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs`

Exit criteria:

- renderer allocates distinct textures for distinct keys;
- resize reallocates only the changed target;
- invalid later requests preserve the previous valid target and emit diagnostics;
- cache exposes target generation, dimensions, format, and sample mode.

Tests:

- two keys with identical descriptors allocate distinct records;
- changing one target size increments only that target generation;
- removing a request retires the target according to retention policy.

### V5.4 - UI Binding Source Cutover

Change:

- `domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs`
- `engine/src/plugins/render/renderer/setup.rs::encode_ui_pass`

Exit criteria:

- `ViewportSurfaceBinding` supports both compatibility flow resources and dynamic texture sources;
- viewport UI embeds can sample a dynamic texture source;
- bind-group cache keys include the full binding source;
- unsupported target sample modes produce explicit skipped-bind diagnostics.

Tests:

- dynamic texture binding source roundtrips through the registry;
- two viewport embeds with different dynamic target ids do not share a bind group keyed only by product slot;
- picking id target bindings are rejected for normal visual embed sampling.

### V5.5 - App Product Target Registry

Change:

- `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs`
- `apps/runenwerk_editor/src/runtime/viewport/mod.rs`
- `apps/runenwerk_editor/src/runtime/plugin.rs::EditorAppPlugin::build`

Exit criteria:

- app runtime derives one target record per requested viewport product;
- target ids are stable for the lifetime of the viewport instance;
- target descriptors are translated into engine dynamic target requests;
- closed viewport records are retired.

Tests:

- two viewport instances produce different scene color, picking id, and overlay target ids;
- resizing one viewport changes only that viewport's target descriptors;
- closing one viewport removes only that viewport's target requests.

### V5.6 - Surface Set And Presentation Resolver Integration

Change:

- `apps/runenwerk_editor/src/runtime/viewport/surface_set.rs`
- `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs::build_surface_binding_registry`
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs::submit_editor_frame_system`

Exit criteria:

- surface handles point at dynamic target binding sources;
- the presentation resolver emits dynamic texture bindings for viewport product slots;
- frame submit consumes already-resolved binding registries and does not allocate product targets.

Tests:

- primary viewport slot resolves to the owning viewport's scene color target;
- missing target record yields an unavailable state, not another viewport's binding;
- architecture guard fails if normal viewport surface binding uses `VIEWPORT_RESOURCE_SCENE_COLOR`.

### V5.7 - Static Resource Containment

Change:

- `apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs`
- `apps/runenwerk_editor/src/runtime/app.rs::register_editor_render_flow`
- `apps/runenwerk_editor/tests/viewport_architecture_guards.rs`

Exit criteria:

- static viewport resource ids are isolated to the migration compatibility path or removed if V6 lands in the same branch;
- new viewport product bindings cannot reference the static ids;
- docs and tests label any remaining static ids as migration-only.

Tests:

- architecture guard rejects static scene color use in normal viewport surface bindings;
- architecture guard rejects adding new static viewport product ids.

### V5.8 - Validation And Documentation Closeout

Change:

- `docs-site/src/content/docs/apps/runenwerk-editor/viewport-expression-implementation-roadmap.md`
- `docs-site/src/content/docs/apps/runenwerk-editor/current-architecture.md`
- `docs-site/src/content/docs/engine/plugins/render/docs/roadmap.md` if render dynamic target APIs change public engine behavior

Exit criteria:

- V5 docs describe the implemented modules and acceptance evidence;
- engine render roadmap acknowledges dynamic target allocation if it becomes a reusable renderer capability;
- documentation validation passes.

Validation:

```text
cargo fmt --all -- --check
cargo test -p runenwerk_editor viewport_surface
cargo test -p runenwerk_editor --test viewport_architecture_guards
cargo test -p runenwerk_editor --test startup_render_smoke
python3 tools/docs/validate_docs.py
```

GPU visual truth belongs to V6 unless V5 and V6 land together:

```text
RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored
```

## Final V5 Acceptance Criteria

V5 is complete only when all of these are true:

- `domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs::ViewportSurfaceBinding` can address dynamic texture targets without renderer backend handles.
- `engine/src/plugins/render/renderer/mod.rs::Renderer` owns a dynamic texture target cache keyed by opaque target keys.
- `engine/src/plugins/render/frame/packet.rs::PreparedRenderFrame` carries a prepared dynamic target request snapshot.
- `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs::ViewportProductTargetRegistryResource` maps viewport/product/slot tuples to stable target records.
- `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs::build_surface_binding_registry` emits dynamic texture bindings for normal viewport presentation.
- Two visible viewports cannot accidentally bind the same scene color target.
- Resize and close operations affect only the target records for the relevant viewport.
- Static viewport resource ids are removed from normal presentation or guarded as migration-only until V6 deletes them.

## Handoff To V6

V5 ends with allocated, viewport-scoped targets and dynamic UI binding support. V6 starts by changing producer execution:

- `apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs` iterates `ViewportRenderJob` records.
- scene color, picking ids, overlay, and later depth are rendered into the targets allocated by V5.
- `assets/shaders/editor_viewport_scene_product.wgsl` becomes target-local and no longer receives multiple panel rectangles.
- `apps/runenwerk_editor/src/runtime/resources.rs::EditorViewportSceneProductUniform` becomes a single-job uniform or equivalent per-job parameter block.

Under the active render product surface foundation bundle, V5 and V6 should land together as one user-visible cutover. If they land separately for reviewability, V5 must be internal infrastructure only and must not introduce a visible path that copies the old global scene product into dynamic targets.
