---
title: "Render Remaining Features Roadmap"
description: "Documentation for Render Remaining Features Roadmap."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
related_designs:
  - ../../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../../../../design/active/editor-native-multi-window-presentation-design.md
  - ../../../../design/active/render-fragment-data-driven-maturity-design.md
  - ../../../../design/active/render-product-surface-foundation-bundle-design.md
  - ../../../../design/active/viewport-dynamic-product-target-allocation-design.md
related_roadmaps:
  - ../../../../workspace/sdf-first-execution-roadmap.md
---

# Render Remaining Features Roadmap

Concrete implementation roadmap for the remaining render features after the hard cutover.

This document assumes the hard cutover is complete:

- normal render path uses compiled builtin execution
- legacy render architecture trees are removed
- builtin pass kinds fail loudly if unimplemented
- examples no longer require custom executors for normal flows

This roadmap is focused on the next major stage:

- complete builtin pass coverage
- prove the architecture with real feature workloads
- improve ergonomics
- strengthen persistent/history resource support
- mature fragments and inspection tooling

---

## SDF-First Roadmap Constraint

SDF renderer and GPU residency work must follow the SDF-first open-world
substrate roadmap. Renderer code consumes producer-built
`RenderProductSelection` and derived GPU residency requests; it must not become
world truth, product authority, or a fallback path around strict product policy.

Phase 3 product selection producers and Phase 4 logical derived GPU residency
are complete. Procgen, SDF terrain rendering, and material/SDF preview GPU work
must consume those contracts and should still wait for their owning product
family upload/render designs unless the work is explicitly documentation-only.

## Status Baseline

Already complete:

- canonical render architecture is in place
- `RenderFlow` v2 is the normal authoring path
- compiled planning exists
- builtin execution exists for `compute_pass`, `fullscreen_pass`, `graphics_pass`, `copy_pass`, `present_pass`, and `builtin_ui_composite_pass`
- hard cutover removed duplicate legacy ownership trees
- examples and tests validate the new architecture

Still missing or incomplete:

- broader graphics/resource-heavy workflow coverage beyond the current proof examples
- additional serious feature proofs on top of the new architecture
- persistent/history resource workflows beyond invocation-scoped history and dynamic target history signatures
- fragment/data-driven maturation
- better inspection/tooling polish
- final public API ergonomics polish

---

## End Goal

After this roadmap, the render system should cleanly support:

- compute pipelines
- fullscreen compositors
- raster/mesh graphics pipelines
- hybrid compute + graphics flows
- boids and particle simulation/rendering
- SDF/raymarch flows
- geometry generation and GPU-driven drawing
- history/persistent resources
- dynamic viewport/product texture targets
- multi-plugin flow composition
- fragment/data-driven flow authoring
- hot reload
- inspection/debug tooling

---

# Roadmap Overview

## Active Bundle

The next large render update should be the [Render Product Surface Foundation Bundle](../../../../design/active/render-product-surface-foundation-bundle-design.md).

That bundle intentionally pulls together binding model closeout, dynamic target allocation, target aliases, prepared render views, history retention, UI sampling, inspection, and proof workloads. It is the no-compromise path for editor viewports and future product preview surfaces; implementing only dynamic allocation without dynamic write targets would leave another bridge.

## Recommended implementation order

### Wave 1 — Core missing builtin execution
1. `graphics_pass`
2. `copy_pass`
3. `present_pass`

### Wave 2 — Real workflow support
4. binding model expansion
4a. dynamic texture target allocation for viewport/product surfaces
5. boids feature proof
6. SDF renderer rebuilt on the new path

### Wave 3 — Usability and persistence
7. gold-path ergonomics polish
8. persistent/history resource support
9. inspection/tooling expansion

### Wave 4 — Data-driven maturity
10. fragment/data-driven maturation

This order minimizes churn and proves real capability as early as possible.

---

# Phase R1 — Builtin `graphics_pass`

Status: Complete (March 12, 2026). Builtin runtime graphics pass execution is wired through compiled flow execution, with loud-failure guards for unsupported advanced graphics bindings.

## Goal

Implement first-class builtin raster/graphics execution.

## Why this matters

This is the biggest missing builtin pass kind.

Without it, the architecture still cannot fully support:

- mesh rendering
- terrain/world rendering
- instanced boids draw
- hybrid mesh + SDF pipelines
- debug line/shape draw flows
- future character rendering paths that are not fullscreen-only

## Domains

- API
- graph
- backend
- renderer
- resource

## Target files

- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/graph/planning.rs`
- `engine/src/plugins/render/backend/execution.rs`
- `engine/src/plugins/render/backend/pipeline_cache.rs`
- `engine/src/plugins/render/backend/resource_allocator.rs`
- `engine/src/plugins/render/renderer/graph_execution.rs`
- `engine/src/plugins/render/resource/usages.rs`

## Required implementation

### 1. Public API support
Ensure `graphics_pass(...)` is fully supported by the public authoring model.

### 2. Compiled pass model
Add or complete a compiled descriptor for graphics passes, such as:

- `CompiledGraphicsPass`

It must include:
- pass ID
- shader/pipeline identity
- target color attachments
- optional depth target
- required buffer bindings
- required sampled/storage resources
- load/store metadata
- dependency metadata

### 3. Pipeline support
Add builtin pipeline support for graphics passes through backend pipeline cache.

### 4. Buffer binding support
Add support for:
- vertex buffers
- index buffers
- instance buffers
- optional indirect draw later

### 5. Depth support
Support depth target binding and validation.

### 6. Loud failure
If unsupported graphics features are requested, fail loudly.
Do not silently skip.

## Verification

### Tests
- graphics pass planning test
- graphics pass validation test
- graphics pass execution smoke test
- compute -> graphics resource handoff test
- depth target attachment test

### Example proof
A visible graphics example must render without custom executors.

## Exit criteria

You can write a declarative `graphics_pass(...)` and get visible geometry with builtin execution only.

---

# Phase R2 — Builtin `copy_pass`

Status: Complete (March 12, 2026). Builtin runtime copy pass execution is wired through compiled flow execution for supported texture-like copies, with loud failures for unsupported combinations.

## Goal

Implement first-class copy execution.

## Why this matters

This is needed for:

- ping-pong workflows
- history preservation
- explicit copy-style graph flows
- texture/buffer copy utilities
- later temporal effects and cache maintenance

## Domains

- API
- graph
- backend
- resource

## Target files

- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/graph/planning.rs`
- `engine/src/plugins/render/backend/execution.rs`
- `engine/src/plugins/render/resource/usages.rs`
- `engine/src/plugins/render/resource/descriptors.rs`

## Required implementation

### 1. Public API support
Support `copy_pass(...)`.

### 2. Compiled pass model
Add compiled copy pass descriptor, such as:

- `CompiledCopyPass`

### 3. Resource validation
Validate legal copy source/destination combinations:
- texture -> texture
- buffer -> buffer
- possibly staged restrictions initially

### 4. Backend execution
Perform actual copy command encoding.

### 5. Loud failure
Unsupported copy combinations must fail loudly.

## Verification

### Tests
- texture copy test
- buffer copy test
- copy validation error test

### Example proof
A compositor or history example must use `copy_pass(...)`.

## Exit criteria

A declarative `copy_pass(...)` executes through builtin backend execution.

---

# Phase R3 — Builtin `present_pass`

Status: Complete (March 12, 2026). Builtin runtime present pass execution is wired through compiled flow execution with explicit terminal present-pass validation semantics.

## Goal

Implement explicit final present support.

## Why this matters

This makes the frame model cleaner and is important for:

- explicit surface handoff
- future editor viewports
- future multi-view/multi-surface flows
- explicit end-of-frame semantics

## Domains

- API
- graph
- backend
- surface

## Target files

- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/graph/planning.rs`
- `engine/src/plugins/render/backend/execution.rs`
- `engine/src/plugins/render/backend/surface.rs`

## Required implementation

### 1. Public API support
Support `present_pass(...)`.

### 2. Compiled pass model
Add compiled present pass descriptor, such as:

- `CompiledPresentPass`

### 3. Validation
Validate legal present target usage.
Prefer a clear rule for how many present passes are allowed per flow.

### 4. Backend execution
Perform explicit final surface presentation handoff.

## Verification

### Tests
- present pass planning test
- present pass validation test
- one-flow present success test

### Example proof
At least one compositor example uses explicit `present_pass(...)`.

## Exit criteria

The frame can end in a first-class declarative `present_pass(...)`.

---

# Phase R4 — Binding Model Expansion

Status: In progress. Landed so far: explicit buffer size metadata on uniform/storage descriptors, first-class buffer-to-buffer `copy_pass` execution, and graphics-pass builder parity (`write_texture`, `storage_state`).

## Goal

Expand the binding model so the architecture cleanly supports real GPU workloads beyond uniform upload.

## Why this matters

This is required for:

- boids instance buffers
- geometry generation
- raster pipelines
- indirect drawing
- sampled/storage texture workflows
- field-based simulation and compose passes

## Domains

- API
- resource
- backend
- renderer

## Target files

- `engine/src/plugins/render/api/bindings.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/resource/descriptors.rs`
- `engine/src/plugins/render/resource/usages.rs`
- `engine/src/plugins/render/backend/resource_allocator.rs`
- `engine/src/plugins/render/backend/execution.rs`

## Required implementation

### Buffers
Support or complete:
- `.vertex_buffer(...)`
- `.index_buffer(...)`
- `.instance_buffer(...)`
- `.indirect_buffer(...)`

### Textures
Support or complete:
- `.sample_texture(...)`
- `.storage_texture(...)`

### State projection
Support or complete:
- `.uniform_state(...)`
- `.uniform_state_with_surface(...)`

### Validation
Validate resource role compatibility:
- sampled texture in legal sampled contexts
- storage texture in legal storage contexts
- vertex/index/instance/indirect usage correctness

## Verification

### Tests
- vertex buffer binding test
- instance buffer binding test
- sampled texture binding test
- storage texture binding test
- indirect buffer validation test

## Exit criteria

The API can express both compute-heavy and graphics-heavy real workflows cleanly.

---

# Phase R-DT — Dynamic Texture Target Allocation

Status: Implemented for the foundation bundle. Descriptor types, producer-scoped request registry, prepared-frame snapshots, renderer-owned dynamic target cache allocation, UI dynamic texture resolution, and target-alias execution are in place.

## Goal

Add renderer-owned dynamic texture targets that can be requested per prepared frame without making dynamic viewport/product surfaces part of static `RenderFlow` declarations.

## Why this matters

This is needed for:

- editor viewport scene-color, picking, overlay, and depth products;
- future asset/material/field preview surfaces;
- debug texture viewers and product inspectors;
- per-viewport resize without reallocating unrelated flow targets;
- avoiding one render flow per viewport or string-suffixed static resource labels.

## Domains

- resource
- runtime
- renderer
- UI composite integration

## Target files

- `engine/src/plugins/render/resource/dynamic_target.rs`
- `engine/src/plugins/render/runtime/dynamic_targets.rs`
- `engine/src/plugins/render/frame/packet.rs`
- `engine/src/plugins/render/runtime/frame_prepare.rs`
- `engine/src/plugins/render/renderer/dynamic_targets.rs`
- `engine/src/plugins/render/renderer/setup.rs`

## Required implementation

### 1. Dynamic target descriptors

Add backend-neutral target keys, formats, usage flags, sample modes, and retention policy.

### 2. Prepared-frame request snapshot

Copy dynamic target requests into `PreparedRenderFrame` so render submission uses the same target set as the submitted UI frame.

### 3. Renderer-owned cache

Allocate, reuse, resize, retain, and retire backend textures by dynamic target key. Preserve previous valid targets when later requests are invalid.

### 4. UI embed resolution

Allow UI viewport embeds to resolve dynamic texture target sources without storing backend texture handles in UI data.

### 5. Inspection hooks

Expose target key, dimensions, format, sample mode, retention state, and generation for diagnostics.

Current proof/inspection notes:

- `engine/src/plugins/render/resource/dynamic_target.rs` owns backend-neutral keys, descriptors, usage flags, sample modes, retention policy, signatures, and validation.
- `engine/src/plugins/render/runtime/dynamic_targets.rs` owns deterministic request snapshots and request diagnostics.
- `engine/src/plugins/render/frame/packet.rs::PreparedRenderFrame` carries `dynamic_texture_targets`, `views`, and `flow_invocations`.
- `engine::plugins::render::inspect::inspect_prepared_render_frame` exposes dynamic targets, prepared views, target alias bindings, flow invocations, and history signatures without reading renderer internals.
- Renderer dynamic target cache allocation and pass attachment resolution are not complete.

## Verification

### Tests

- dynamic target descriptor validation test
- prepared-frame dynamic target snapshot test
- renderer dynamic target cache allocation/reuse/resize test
- UI embed dynamic texture source resolution test

## Exit criteria

The renderer can allocate distinct dynamic texture targets for separate viewport/product keys, resize only the affected target, and expose sampleable targets to UI composition without editor-specific types entering the engine render API.

---

# Phase R5 — Gold-Path Ergonomics Polish

## Goal

Make the architecture feel good to use, not just correct.

## Why this matters

The hard cutover made the system real.
This phase makes it pleasant.

## Domains

- API
- docs
- examples

## Target files

- `engine/src/plugins/render/api/flow.rs`
- `engine/src/plugins/render/api/passes.rs`
- `engine/src/plugins/render/api/resources.rs`
- `engine/src/plugins/render/api/bindings.rs`
- `engine/src/plugins/render/README.md`
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`

## Required implementation

### 1. Naming audit
Review:
- `ecs_resource::<T>()`
- `uniform_state(...)`
- `uniform_state_with_surface(...)`
- resource declaration names
- pass builder naming consistency

### 2. Boilerplate audit
Remove unnecessary repetition in examples and common usage paths.

### 3. Error message polish
Improve diagnostics for:
- missing resources
- missing ECS state
- unsupported pass features
- invalid bindings
- invalid dependencies

### 4. Gold-path examples
Make three canonical examples excellent:
- minimal fullscreen flow
- Game of Life
- contribution/compositor flow

## Verification

### Criteria
A new user should be able to understand and copy the gold-path examples without reading internals.

## Exit criteria

The public API feels deliberate and low-friction.

---

# Phase R6 — Boids Feature Proof

Status: Proof updated. `engine/examples/boids_render_flow/` is the canonical builtin compiled boids-shaped flow declaration (compute + storage + graphics + instance draw-buffer binding + copy + present, no custom executors).

## Goal

Prove the architecture with a serious compute + draw workload.

## Why this matters

Boids is the best realistic next proof because it uses:

- ECS-driven simulation settings
- compute simulation
- storage buffers
- graphics or fullscreen rendering
- explicit pass dependencies

## Domains

- example / feature proof
- API
- backend
- renderer

## Target files

Suggested example tree:

- `engine/examples/boids_render_flow/main.rs`
- `engine/examples/boids_render_flow/README.md`
- `engine/examples/boids_render_flow/assets/boids_sim.wgsl`
- `engine/examples/boids_render_flow/assets/boids_draw.wgsl`
- additional example modules as needed

## Implementation options

### Preferred version
Compute + instanced graphics draw.

Why:
- validates `graphics_pass`
- validates storage/instance buffer handling
- proves a very important real workflow

### Alternative version
Compute field + fullscreen compose.

This is still useful, but less important if `graphics_pass` is the current main gap.

## Verification

### Example behavior
- visible boids motion
- no custom executors
- no manual registry wiring
- ECS settings drive simulation/render params

## Exit criteria

Boids works entirely on the builtin compiled flow system.

---

# Phase R7 — Rebuild SDF Renderer on the New Path

Status: Proof updated. `engine/examples/sdf_render_flow/` is the canonical builtin compiled SDF-shaped flow declaration (compute preparation + fullscreen raymarch compose + flow-owned history copy + present, no custom executors).

## Goal

Rebuild a serious SDF/raymarch example on the new architecture.

## Why this matters

You explicitly want this engine to be strong for:

- SDF rendering
- raymarching
- procedural rendering
- hybrid future with `domain/sdf`, `domain/spatial`, and `domain/geometry`

## Domains

- example or feature-owned render support
- API
- backend
- renderer

## Target files

Suggested example tree:

- `engine/examples/sdf_render_flow/main.rs`
- `engine/examples/sdf_render_flow/README.md`
- `engine/examples/sdf_render_flow/assets/...`
- runtime/rendering modules as needed

## Required implementation

- use `RenderFlow`
- use builtin `compute_pass(...)` and/or `fullscreen_pass(...)`
- no custom executors
- UI composite optional but recommended
- debug texture/resource views optional but useful

## Verification

### Example behavior
- visible SDF render output
- no legacy render architecture usage
- no custom executor path

## Exit criteria

A serious SDF example proves the architecture supports your preferred rendering style cleanly.

---

# Phase R8 — Persistent and History Resource Support

Status: Implemented foundation support. History texture declarations and copy-pass proof usage exist in boids and SDF examples. Dynamic target retention descriptors and prepared view/invocation history signatures are inspectable and participate in renderer invalidation. Invocation-scoped flow-owned history textures prevent prepared invocations from sharing stale history.

## Goal

Strengthen support for persistent temporal resources and cache-oriented workflows.

## Why this matters

Needed for:

- history buffers
- temporal AA later
- clipmaps
- persistent simulation buffers
- cached large-world rendering resources
- future open-world compatible workflows

## Domains

- resource
- graph
- backend

## Target files

- `engine/src/plugins/render/resource/lifetime.rs`
- `engine/src/plugins/render/resource/transient.rs`
- `engine/src/plugins/render/backend/resource_allocator.rs`
- `engine/src/plugins/render/graph/planning.rs`
- `engine/src/plugins/render/resource/descriptors.rs`

## Required implementation

### 1. Persistent vs transient clarity
Strengthen the distinction between:
- persistent flow-owned resources
- transient per-frame resources

### 2. History workflows
Support or document patterns for:
- previous-frame textures
- persistent buffers
- temporal resource dependencies

### 3. Validation
Validate correct usage of persistent/history resources.

### 4. Example proof
Use history resources in at least one compositor or debug workflow.

## Verification

### Tests
- persistent resource survival test
- history resource planning test
- transient lifetime separation test

## Exit criteria

The architecture cleanly supports frame-to-frame GPU resource persistence.

---

# Phase R9 — Inspection and Debug Tooling Expansion

Status: Implemented foundation support. Graph/resource/texture/timing inspection exists, and prepared-frame inspection exposes dynamic target descriptors, views, flow invocations, target alias bindings, and history signatures. Runtime resource generation for dynamic targets is renderer-owned.

## Goal

Make complex flows debuggable and inspectable enough for serious development.

## Why this matters

Now that the architecture can express more complex workflows, debugging must keep up.

## Domains

- inspect
- graph
- resource

## Target files

- `engine/src/plugins/render/inspect/graph_dump.rs`
- `engine/src/plugins/render/inspect/resource_inspector.rs`
- `engine/src/plugins/render/inspect/texture_view.rs`
- `engine/src/plugins/render/inspect/timings.rs`

## Required implementation

### 1. Better graph dump
Show:
- passes
- resources
- dependencies
- execution order
- pass kinds

### 2. Better resource inspection
Show:
- declared resource types
- imported/persistent/transient categories
- usage roles
- realization metadata where appropriate

### 3. Better texture view tooling
Support viewing:
- color targets
- storage textures
- history textures
- debug outputs

### 4. Better timings
Support:
- per-pass timing
- total frame timing
- optional debug UI/overlay later

## Verification

### Example proof
The debug/inspect example should become genuinely useful for real debugging.

## Exit criteria

Complex mixed flows are inspectable without digging through internal implementation code.

---

# Phase R10 — Fragment and Data-Driven Maturation

Owning design:

- `docs-site/src/content/docs/design/active/render-fragment-data-driven-maturity-design.md`

## Goal

Make fragments and hot reload feel native to the architecture rather than bridged.

## Why this matters

You already laid the groundwork.
This phase removes the remaining awkwardness.

## Domains

- composition
- API IDs
- graph
- inspect

## Target files

- `engine/src/plugins/render/composition/fragments.rs`
- `engine/src/plugins/render/composition/hot_reload.rs`
- `engine/src/plugins/render/api/ids.rs`
- `engine/src/plugins/render/graph/merge.rs`

## Required implementation

### 1. Better ID model
Reduce reliance on internal string-interning bridges where possible.

Long-term target:
- stronger owned/typed IDs internally
- less friction for data-driven authored fragments

### 2. Better fragment validation
Improve diagnostics for:
- invalid fragment references
- missing resources
- namespace collisions
- incompatible fragment composition

### 3. Better hot reload loop
Improve:
- revision tracking
- reload diagnostics
- failure recovery

### 4. Example proof
At least one fragment-driven flow or contribution example should demonstrate the model clearly.

## Verification

### Tests
- fragment parse/validate test
- fragment merge collision test
- hot reload revision/error state test

## Exit criteria

Fragments and hot reload feel like first-class extensions of the architecture.

---

# Recommended Concrete Order of Attack

## Immediate next steps
1. **Render Product Surface Foundation Bundle** from `docs-site/src/content/docs/design/active/render-product-surface-foundation-bundle-design.md`
2. **R6 — Boids feature proof** as a bundle proof workload
3. **R7 — Rebuild SDF renderer on new path** as a bundle proof workload

Builtin pass completion is done. The highest-value continuation is the product-surface foundation because it removes the target/view/resource limitations blocking editor viewports and future preview surfaces.

## Then
4. **R5 — Gold-path ergonomics polish**
5. **R8 — Persistent/history resources**
6. **R9 — Inspection/tooling expansion**

These make the system robust and debuggable after the main execution and product-surface paths are proven.

## Then
7. **Native multi-window presentation** from `docs-site/src/content/docs/design/active/editor-native-multi-window-presentation-design.md` when second-monitor editor workflows become the active product priority.
8. **R10 — Fragment/data-driven maturation**

These extend the stable product-surface foundation into multi-swapchain editor presentation and authored render-flow composition.

---

# Concrete Milestone Definition

## Milestone M1 — Builtin Pass Completion
Complete when:
- `graphics_pass` works
- `copy_pass` works
- `present_pass` works

## Milestone M2 — Real Feature Proof
Complete when:
- dynamic viewport/product target allocation supports the editor V5 surface contract
- one boids example works
- one serious SDF example works
- both use builtin compiled execution only

## Milestone M3 — Production-Ready Usability
Complete when:
- persistent/history resources are solid
- inspect tooling is genuinely useful
- public API has undergone ergonomics polish

## Milestone M4 — Data-Driven Maturity
Complete when:
- fragments and hot reload feel first-class
- typed/owned ID handling is less bridged and more native

---

# Verification Plan

## Required tests after each major phase

### Core architecture tests
- `cargo test -p engine --test render_flow_graph`
- `cargo test -p engine --test render_flow_bindings`
- `cargo test -p engine --test render_flow_bridge`
- `cargo test -p engine --test render_flow_contributions`
- `cargo test -p engine --test render_resource_model`
- `cargo test -p engine --test render_resource_lifetime`
- `cargo test -p engine --test render_dynamic_targets`
- `cargo test -p engine --test render_inspect`
- `cargo test -p engine --test render_flow_fragments`

### Examples
- `cargo test -p engine --example render_flow_fullscreen_minimal`
- `cargo test -p engine --example render_flow_postprocess_compositor`
- `cargo test -p engine --example render_flow_contributions`
- `cargo test -p engine --example render_flow_debug_inspect`
- `cargo test -p engine --example game_of_life_sdf`

### New examples as phases land
- boids example
- SDF flow example

---

# Risks

## Main risks to watch

### 1. `graphics_pass` becomes too special-case
Keep it generic enough for:
- mesh rendering
- instancing
- hybrid future workloads

### 2. Binding model grows inconsistently
Prefer one coherent model for:
- buffers
- textures
- state projection
- pass bindings

### 3. Feature proofs expose API friction too late
Build boids and SDF proofs early enough to inform API polish.

### 4. Persistent resource model stays vague
History/persistent/transient distinctions must become concrete before large-world and temporal systems.

### 5. Fragments remain second-class
The current fragment bridge is acceptable, but long-term awkwardness should be reduced.

### 6. Dynamic targets become editor-specific
Dynamic target allocation must stay keyed by backend-neutral render target descriptors. Viewport ids, shell surface ids, and expression-product semantics belong in app/domain code.

---

# Final Recommendation

Use this roadmap as the implementation plan for the remaining render features.

## Best next step
Implement:

1. the Render Product Surface Foundation Bundle;
2. editor viewport V5+V6 as the first app proof;
3. boids and SDF examples as engine proofs.

That is the highest-value continuation of the current architecture now that builtin graphics, copy, and present execution are complete.
