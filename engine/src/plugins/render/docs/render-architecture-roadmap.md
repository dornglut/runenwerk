# Render Architecture Roadmap

Status: target roadmap  
Scope: long-term internal architecture plan for `engine/src/plugins/render`  
Audience: engine/runtime/render developers and AI coding agents

Primary goals:
- make render ownership clearer
- prepare for SDF-first rendering
- support graphics + compute workflows
- support debug/inspection tooling
- reduce top-level flatness
- make future growth obvious

Non-goals:
- full renderer rewrite in one pass
- changing stable behavior without validation
- forcing all rendering to become SDF-only immediately

---

## 1. Purpose

This roadmap defines the target architecture for `engine/src/plugins/render`.

The current render plugin already contains useful separation ideas:
- graph description
- graph execution
- shader management
- renderer runtime flow

But it is still flatter and more implementation-shaped than ideal.

This roadmap moves the render plugin toward a structure with clear long-term homes for:
- plugin/composition
- render domain contracts
- backend/wgpu integration
- frame graph specification and execution
- per-frame renderer lifecycle
- shader/pipeline systems
- render resources/transients
- SDF render integration
- debug and inspection workflows

---

## 2. Current Structure Snapshot

Current structure (abridged):

```text
render/
├── README.md
├── docs/
├── domain.rs
├── frame_graph.rs
├── gfx.rs
├── mod.rs
├── pipeline_key.rs
├── plugin.rs
├── render_executor_registry/
├── render_frame_bindings.rs
├── render_graph_registry/
├── renderer/
├── shader_manager/
├── submit.rs
└── wgpu_ctx.rs
```

This is workable, but several important concerns are still too flat or not yet clearly grouped.

## 3. Target End State

### 3.1 Long-term target tree

```text
engine/src/plugins/render/
├── mod.rs
├── plugin.rs
├── README.md
├── docs/
│   ├── README.md
│   ├── architecture.md
│   ├── frame-graph.md
│   ├── sdf-rendering.md
│   └── debug-views.md
├── domain/
│   ├── mod.rs
│   ├── frame.rs
│   ├── gfx.rs
│   ├── pipeline.rs
│   ├── material.rs
│   ├── pass.rs
│   ├── timings.rs
│   └── view.rs
├── backend/
│   ├── mod.rs
│   ├── wgpu_ctx.rs
│   ├── device.rs
│   ├── surface.rs
│   └── formats.rs
├── frame_graph/
│   ├── mod.rs
│   ├── ids.rs
│   ├── spec.rs
│   ├── builders.rs
│   ├── registry.rs
│   ├── resources.rs
│   ├── executor.rs
│   └── validation.rs
├── renderer/
│   ├── mod.rs
│   ├── extract.rs
│   ├── prepare.rs
│   ├── render_flow.rs
│   ├── submit.rs
│   ├── frame_bindings.rs
│   ├── graph_execution.rs
│   └── setup.rs
├── shader/
│   ├── mod.rs
│   ├── registry.rs
│   ├── types.rs
│   ├── helpers.rs
│   └── hot_reload.rs
├── pipelines/
│   ├── mod.rs
│   ├── keys.rs
│   ├── cache.rs
│   └── specialization.rs
├── resources/
│   ├── mod.rs
│   ├── textures.rs
│   ├── buffers.rs
│   ├── transient.rs
│   └── bindings.rs
├── sdf/
│   ├── mod.rs
│   ├── extract.rs
│   ├── bindings.rs
│   ├── fields.rs
│   ├── raymarch.rs
│   ├── materials.rs
│   └── debug_views.rs
└── debug/
    ├── mod.rs
    ├── overlays.rs
    ├── texture_inspector.rs
    ├── timings.rs
    └── graph_dump.rs
```

## 4. Architectural Principles

### 4.1 Domain-first ownership

Render code should be grouped by responsibility, not by historical implementation accident.

### 4.2 Explicit backend boundary

`wgpu` infrastructure should live in a backend-oriented subdomain, not dominate the root.

### 4.3 Frame graph as a true subdomain

The frame graph should own:
- ids
- spec
- builders
- registry
- graph resources
- validation
- execution planning/execution helpers

### 4.4 Renderer lifecycle should be obvious

Per-frame rendering should be easy to scan:
- extract
- prepare
- execute
- submit

### 4.5 SDF integration must be first-class

Since the engine direction is SDF-first, render needs an explicit long-term home for:
- SDF extraction
- SDF bindings
- raymarch/compute passes
- debug visualization

### 4.6 Debugging is not optional

Render inspection needs dedicated architecture for:
- texture viewers
- pass/resource inspection
- timings
- field/influence-map views

## 5. Ownership Model

### 5.1 `plugin.rs`

Owns:
- plugin composition
- app/system/resource registration
- top-level integration entry

Should not own:
- renderer internals
- graph spec logic
- backend details

### 5.2 `domain/`

Owns stable render-domain types and contracts:
- frame data
- pipeline-facing identities
- material/view/pass/timing types
- shared render concepts

### 5.3 `backend/`

Owns backend-specific GPU plumbing:
- wgpu context
- device/surface abstractions
- format policy

### 5.4 `frame_graph/`

Owns graph modeling and graph execution support:
- ids
- specs
- builders
- registries
- graph resources
- validation
- execution helpers

### 5.5 `renderer/`

Owns per-frame runtime flow:
- extract
- prepare
- graph execution
- submit/present
- setup/init
- frame bindings

### 5.6 `shader/`

Owns shader registry/types/helpers and future hot-reload support.

### 5.7 `pipelines/`

Owns:
- pipeline keys
- pipeline cache
- specialization policy

### 5.8 `resources/`

Owns:
- textures
- buffers
- transient resources
- binding/resource helper structures

### 5.9 `sdf/`

Owns render-side integration of `foundation/sdf`:
- extracting field data
- GPU bindings
- SDF render passes
- raymarch features
- SDF-specific debug views

### 5.10 `debug/`

Owns inspection and overlay tools:
- timings
- graph dump/inspection
- texture viewers
- render debug overlays

## 6. Current-to-Target Mapping

### 6.1 Root files

`engine/src/plugins/render/domain.rs`  
Target: `engine/src/plugins/render/domain/mod.rs`  
Later split into:
- `domain/frame.rs`
- `domain/gfx.rs`
- `domain/pipeline.rs`
- `domain/timings.rs`
- `domain/view.rs`

`engine/src/plugins/render/frame_graph.rs`  
Target: `engine/src/plugins/render/frame_graph/mod.rs`  
Later split into focused modules if needed.

`engine/src/plugins/render/gfx.rs`  
Target: likely `engine/src/plugins/render/domain/gfx.rs`  
Unless parts are backend-specific, then move those to `backend/`.

`engine/src/plugins/render/pipeline_key.rs`  
Target: `engine/src/plugins/render/pipelines/keys.rs`

`engine/src/plugins/render/submit.rs`  
Target: `engine/src/plugins/render/renderer/submit.rs`

`engine/src/plugins/render/render_frame_bindings.rs`  
Target: `engine/src/plugins/render/renderer/frame_bindings.rs`

`engine/src/plugins/render/wgpu_ctx.rs`  
Target: `engine/src/plugins/render/backend/wgpu_ctx.rs`

### 6.2 Existing subfolders

`engine/src/plugins/render/render_graph_registry/`  
Target: fold into `engine/src/plugins/render/frame_graph/`

Likely mappings:
- `builders.rs` -> `frame_graph/builders.rs`
- `registry.rs` -> `frame_graph/registry.rs`
- `spec.rs` -> `frame_graph/spec.rs`
- `ids_and_registrations.rs` -> `frame_graph/ids.rs` or `frame_graph/registry.rs`

`engine/src/plugins/render/render_executor_registry/`  
Target: split between:
- `frame_graph/executor.rs`
- `renderer/graph_execution.rs`
- maybe `renderer/setup.rs`

Reason:
- if it is graph execution support, it belongs with graph/runtime execution
- if it is frame lifecycle/runtime flow, it belongs in `renderer/`

`engine/src/plugins/render/shader_manager/`  
Target: `engine/src/plugins/render/shader/`

Likely mappings:
- `registry_impl.rs` -> `shader/registry.rs`
- `types.rs` -> `shader/types.rs`
- `helpers.rs` -> `shader/helpers.rs`

## 7. Roadmap Phases

### Phase R0 - Architecture lock and migration map

Goal: document the target architecture before moving implementation around.

Deliverables:
- `engine/src/plugins/render/docs/architecture.md`
- target tree
- ownership notes
- migration map
- basic boundaries for:
  - graph vs renderer
  - backend vs domain
  - sdf vs generic render
  - debug vs runtime flow

Exit criteria:
- future file placement is obvious
- contributors know where new code belongs

### Phase R1 - Structural cleanup only

Goal: reshape folders and module boundaries without significant behavior changes.

Exact files/modules to create:
- `engine/src/plugins/render/domain/mod.rs`
- `engine/src/plugins/render/backend/mod.rs`
- `engine/src/plugins/render/frame_graph/mod.rs`
- `engine/src/plugins/render/shader/mod.rs`
- `engine/src/plugins/render/pipelines/mod.rs`
- `engine/src/plugins/render/resources/mod.rs`
- `engine/src/plugins/render/sdf/mod.rs`
- `engine/src/plugins/render/debug/mod.rs`

Exact files/modules to move:
- `engine/src/plugins/render/domain.rs` -> `engine/src/plugins/render/domain/mod.rs`
- `engine/src/plugins/render/wgpu_ctx.rs` -> `engine/src/plugins/render/backend/wgpu_ctx.rs`
- `engine/src/plugins/render/frame_graph.rs` -> `engine/src/plugins/render/frame_graph/mod.rs`
- `engine/src/plugins/render/submit.rs` -> `engine/src/plugins/render/renderer/submit.rs`
- `engine/src/plugins/render/render_frame_bindings.rs` -> `engine/src/plugins/render/renderer/frame_bindings.rs`
- `engine/src/plugins/render/pipeline_key.rs` -> `engine/src/plugins/render/pipelines/keys.rs`
- `engine/src/plugins/render/shader_manager/*` -> `engine/src/plugins/render/shader/*`

Exit criteria:
- compile passes
- no major behavior change
- root becomes less flat
- future homes for SDF/debug/resources now exist

### Phase R2 - Frame graph unification

Goal: make the frame graph a coherent subdomain.

Target files/modules:
- `engine/src/plugins/render/frame_graph/ids.rs`
- `engine/src/plugins/render/frame_graph/spec.rs`
- `engine/src/plugins/render/frame_graph/builders.rs`
- `engine/src/plugins/render/frame_graph/registry.rs`
- `engine/src/plugins/render/frame_graph/resources.rs`
- `engine/src/plugins/render/frame_graph/executor.rs`
- `engine/src/plugins/render/frame_graph/validation.rs`

Work:
- fold and rename content from:
  - `render_graph_registry/*`
  - graph-oriented parts of `render_executor_registry/*`

Exit criteria:
- graph ownership is centralized
- graph structure and graph execution helpers are discoverable in one place

### Phase R3 - Renderer lifecycle cleanup

Goal: make render frame flow readable and scalable.

Target files/modules:
- `engine/src/plugins/render/renderer/extract.rs`
- `engine/src/plugins/render/renderer/prepare.rs`
- `engine/src/plugins/render/renderer/render_flow.rs`
- `engine/src/plugins/render/renderer/graph_execution.rs`
- `engine/src/plugins/render/renderer/submit.rs`
- `engine/src/plugins/render/renderer/setup.rs`
- `engine/src/plugins/render/renderer/frame_bindings.rs`

Work:
- make these phases explicit:
  - extract CPU/world data
  - prepare GPU-side state/resources
  - execute graph/passes
  - submit/present

Exit criteria:
- someone can understand frame flow quickly
- setup vs runtime execution is clean

### Phase R4 - Backend isolation

Goal: separate backend plumbing from render-domain logic.

Target files/modules:
- `engine/src/plugins/render/backend/wgpu_ctx.rs`
- `engine/src/plugins/render/backend/device.rs`
- `engine/src/plugins/render/backend/surface.rs`
- `engine/src/plugins/render/backend/formats.rs`

Work:
- move backend-specific logic under `backend/`.

Exit criteria:
- wgpu concerns are isolated
- backend code no longer bleeds across the root

### Phase R5 - Shader and pipeline architecture cleanup

Goal: prepare render for graphics + compute growth.

Target files/modules:
- `engine/src/plugins/render/shader/registry.rs`
- `engine/src/plugins/render/shader/types.rs`
- `engine/src/plugins/render/shader/helpers.rs`
- optional later: `engine/src/plugins/render/shader/hot_reload.rs`
- `engine/src/plugins/render/pipelines/keys.rs`
- `engine/src/plugins/render/pipelines/cache.rs`
- `engine/src/plugins/render/pipelines/specialization.rs`

Work:
- separate:
  - shader registry/types/helpers
  - pipeline identity
  - pipeline caching and specialization policy

Exit criteria:
- graphics and compute growth path is clearer
- pipeline and shader concerns are not entangled

### Phase R6 - Render resource ownership

Goal: create an explicit home for texture/buffer/transient resource logic.

Target files/modules:
- `engine/src/plugins/render/resources/textures.rs`
- `engine/src/plugins/render/resources/buffers.rs`
- `engine/src/plugins/render/resources/transient.rs`
- `engine/src/plugins/render/resources/bindings.rs`

Work:
- introduce structure for:
  - persistent textures
  - persistent buffers
  - transient/graph-frame resources
  - binding helpers

Exit criteria:
- future compute and debug workflows have a resource home
- transient resources are no longer ad hoc

### Phase R7 - SDF render integration

Goal: create a first-class render home for SDF rendering.

Target files/modules:
- `engine/src/plugins/render/sdf/mod.rs`
- `engine/src/plugins/render/sdf/extract.rs`
- `engine/src/plugins/render/sdf/bindings.rs`
- `engine/src/plugins/render/sdf/fields.rs`
- `engine/src/plugins/render/sdf/raymarch.rs`
- `engine/src/plugins/render/sdf/materials.rs`
- `engine/src/plugins/render/sdf/debug_views.rs`

Work:
- define how render consumes `foundation/sdf`:
  - CPU extraction
  - GPU bindings
  - SDF pass logic
  - debug field visualization
  - later material hookups

Exit criteria:
- SDF rendering is not buried inside generic renderer files
- meshless/raymarch paths have a real architectural home

### Phase R8 - Debug and inspection tooling

Goal: make realtime render debugging first-class.

Target files/modules:
- `engine/src/plugins/render/debug/overlays.rs`
- `engine/src/plugins/render/debug/texture_inspector.rs`
- `engine/src/plugins/render/debug/timings.rs`
- `engine/src/plugins/render/debug/graph_dump.rs`

Work:
- prepare for:
  - texture viewers
  - influence/brick/field views
  - frame graph inspection
  - pass timings
  - overlay diagnostics

Exit criteria:
- debug views are structurally supported
- inspection tools no longer feel bolted on

### Phase R9 - Material and feature expansion path

Goal: prepare for later material/render feature growth.

Candidate files/modules:
- `engine/src/plugins/render/domain/material.rs`
- `engine/src/plugins/render/sdf/materials.rs`
- maybe later a dedicated `materials/` subdomain if justified

Work:
- document and reserve a clean material ownership path for:
  - triplanar workflows
  - procedural materials
  - SDF surface materials
  - future graph-driven material systems

Exit criteria:
- material expansion path is clear
- feature work will not be forced into arbitrary files

### Phase R10 - Documentation closeout

Goal: make render structure understandable for contributors and agents.

Deliverables:
- `engine/src/plugins/render/README.md`
- `engine/src/plugins/render/docs/architecture.md`
- `engine/src/plugins/render/docs/frame-graph.md`
- `engine/src/plugins/render/docs/sdf-rendering.md`
- `engine/src/plugins/render/docs/debug-views.md`

Must document:
- lifecycle
- graph ownership
- backend ownership
- resource ownership
- SDF integration boundary
- where compute features belong
- where debug viewers belong

Exit criteria:
- new contributors can find entry points fast
- docs reflect actual structure

## 8. Package Sequence

### Package A - safe structure

Includes:
- R0
- R1

This is the best immediate slice.

### Package B - graph/runtime clarity

Includes:
- R2
- R3
- R4

This makes the plugin much easier to reason about.

### Package C - growth foundations

Includes:
- R5
- R6

This prepares for graphics/compute/resource-heavy workflows.

### Package D - SDF and debug readiness

Includes:
- R7
- R8

This aligns render with the engine's actual direction.

### Package E - long-term polish

Includes:
- R9
- R10

## 9. Validation Gates

After each package, run at least:
- `cargo test -p engine`

If that is too heavy for a structure-only slice, at minimum run:
- `cargo test -p engine --test runtime_surface_guard`

For behavior-touching graph/runtime refactors, also verify:
- render plugin still initializes
- representative examples still compile
- SDF or render examples still build when applicable

## 10. Exit Criteria

This roadmap is complete when:
- render root is no longer overly flat
- frame graph ownership is unified
- renderer lifecycle is explicit
- backend is isolated
- shader/pipeline/resource growth paths are clear
- SDF integration has a first-class home
- debug/inspection has a first-class home
- docs match the code layout

## 11. Recommended Immediate Next Slice

Implement only:
- R0 - architecture lock
- R1 - structural cleanup only

That means:
- create subdomain folders
- move obvious files
- reserve `sdf/`, `debug/`, `resources/`, `backend/`, `frame_graph/`
- avoid major behavior changes

This is the highest-value, lowest-risk next step.

## 12. What Not To Do In The First Slice

Do not do these yet:
- rewrite graph execution model
- redesign the whole renderer runtime
- invent a full material graph system
- add major new SDF rendering behavior
- add clipmap/brickmap systems
- change working behavior unless required by the move

Keep the first slice architectural and safe.
