# Architecture Target

Target architecture for the Runenwerk workspace.

This document describes the intended long-term structure for shared foundations, engine runtime domains, render infrastructure, game-owned logic, and tooling.

It is a **target structure**, not a claim that the repository already matches it fully.

---

## 1. Goals

This target architecture is intended to support:

- strong domain ownership
- easy public usage and discoverability
- reusable foundation primitives
- engine-level runtime ownership where appropriate
- render as a focused GPU frame-production domain, not a catch-all
- game-specific logic remaining game-owned until clearly reusable
- future editor and DCC tooling without collapsing everything into `engine`
- scalable support for:
  - AABB and geometric primitives
  - BVH and other spatial structures
  - LOD
  - chunking
  - clipmaps
  - caches
  - materials
  - meshes and runtime model content
  - animation
  - lighting
  - compute
  - frame graphs
  - material and compositor graphs
  - SDF rendering and SDF modelling
  - editor and asset tooling

---

## 2. Ownership Principles

### Foundation owns

Reusable, engine-agnostic primitives and data structures.

Examples:

- math
- geometry
- spatial structures
- generic graph primitives
- generic asset handles and ids

### Engine owns

Runtime composition, scene/content/streaming/animation systems, and engine-level plugins.

### Render owns

GPU frame-production infrastructure and render feature systems.

### Games own

Vertical-slice-specific gameplay, content, world logic, and feature experiments until they are clearly reusable.

### Tools own

Editor and DCC workflows.

---

## 3. Top-Level Target Structure

```text
repo/
├── apps/
│   ├── grotto_client/
│   ├── grotto_server/
│   ├── grotto_online/
│   ├── grotto_fleet_control/
│   └── editor/                         # future editor app if added
│
├── assets/
│   ├── editor/
│   ├── gameplay/
│   ├── models/
│   ├── render/
│   ├── scenes/
│   ├── shaders/
│   └── ui/
│
├── docs/
│   ├── index.md
│   ├── current-state.md
│   ├── guidelines/
│   ├── roadmaps/
│   └── visuals/
│
├── foundation/
│   ├── ecs/
│   ├── ecs_macros/
│   ├── scheduler/
│   ├── grid/
│   ├── geometry/                       # new
│   ├── spatial/                        # new
│   ├── graphs/                         # optional future
│   └── assets/                         # optional future
│
├── engine/
│   ├── src/
│   │   ├── app/
│   │   ├── runtime/
│   │   ├── plugins/
│   │   ├── scene/                      # future stronger engine-owned scene domain
│   │   ├── content/                    # future
│   │   ├── streaming/                  # future
│   │   ├── animation/                  # future
│   │   ├── lib.rs
│   │   ├── prelude.rs
│   │   └── state.rs
│   ├── docs/
│   ├── examples/
│   └── tests/
│
├── games/
│   └── cavern_hunt/
│
├── net/
│   ├── engine_net/
│   ├── engine_net_quic/
│   ├── engine_sim/
│   ├── engine_history/
│   └── engine_net_macros/
│
├── ops/
│   ├── docker/
│   ├── fleet/
│   └── helm/
│
└── tools/
    ├── editor/                         # optional if not under apps/
    └── dcc/                            # future
```

---

## 4. Foundation Target Structure

### 4.1 `foundation/geometry/`

Purpose:

- reusable geometric primitives and intersection logic
- no engine, render, or runtime assumptions

```text
foundation/geometry/
├── Cargo.toml
├── README.md
├── src/
│   ├── mod.rs
│   ├── aabb.rs
│   ├── sphere.rs
│   ├── ray.rs
│   ├── frustum.rs
│   ├── plane.rs
│   ├── triangle.rs
│   └── intersection.rs
└── tests/
```

Owns:

- AABB
- sphere
- ray
- frustum
- plane
- triangle
- geometric intersection helpers

Does not own:

- scene BVHs
- render extraction
- engine chunking logic
- gameplay collision policy

### 4.2 `foundation/spatial/`

Purpose:

- reusable spatial indexing, culling, LOD, clipmap, and chunk-addressing primitives

```text
foundation/spatial/
├── Cargo.toml
├── README.md
├── src/
│   ├── mod.rs
│   ├── bvh/
│   │   ├── mod.rs
│   │   ├── bounds.rs
│   │   ├── builder.rs
│   │   ├── node.rs
│   │   └── traversal.rs
│   ├── lod/
│   │   ├── mod.rs
│   │   ├── metrics.rs
│   │   ├── policy.rs
│   │   └── selection.rs
│   ├── clipmaps/
│   │   ├── mod.rs
│   │   ├── indexing.rs
│   │   ├── levels.rs
│   │   └── regions.rs
│   └── chunking/
│       ├── mod.rs
│       ├── coords.rs
│       ├── keys.rs
│       └── regions.rs
└── tests/
```

Owns:

- BVH primitives
- generic LOD metrics and selection helpers
- clipmap indexing math
- chunk coordinate and key math
- generic spatial traversal helpers

Does not own:

- engine scene residency
- render pass execution
- game-specific worldgen
- feature-specific runtime policies

### 4.3 `foundation/graphs/` (optional future)

Purpose:

- shared graph primitives only

```text
foundation/graphs/
├── Cargo.toml
├── README.md
├── src/
│   ├── mod.rs
│   ├── ids.rs
│   ├── pins.rs
│   ├── values.rs
│   ├── edges.rs
│   └── validation.rs
└── tests/
```

Owns:

- shared node, pin, edge, and typed-value graph building blocks
- reusable validation helpers

Does not own:

- frame graph semantics
- material graph semantics
- compositor graph semantics
- animation graph semantics

### 4.4 `foundation/assets/` (optional future)

Purpose:

- shared asset ids, handles, cache keys, and versioning primitives

Potential ownership:

- asset identifiers
- handle semantics
- versioning helpers
- cache-key primitives

This should remain generic and should not own engine-specific loading flows or editor tooling.

---

## 5. Engine Target Structure

### 5.1 Core engine ownership remains

These domains already make sense and should remain central:

```text
engine/src/
├── app/
├── runtime/
├── plugins/
├── lib.rs
├── prelude.rs
└── state.rs
```

### 5.2 Future `engine/src/scene/`

Purpose:

- engine-owned scene and runtime scene contracts if scene becomes more core than plugin-shaped

```text
engine/src/scene/
├── mod.rs
├── domain/
│   ├── mod.rs
│   ├── scene.rs
│   ├── instances.rs
│   ├── visibility.rs
│   └── bounds.rs
├── extraction/
│   ├── mod.rs
│   ├── render.rs
│   ├── lighting.rs
│   └── animation.rs
└── runtime/
    ├── mod.rs
    ├── state.rs
    └── updates.rs
```

Owns:

- scene instance and runtime scene ownership
- scene-level visibility/runtime contracts
- engine scene boundaries

Migration note:

Current `engine/src/plugins/scene/` can remain until enough pressure exists to promote parts of it.

### 5.3 Future `engine/src/content/`

Purpose:

- engine-level runtime content models, separate from raw render infrastructure

```text
engine/src/content/
├── mod.rs
├── meshes/
│   ├── mod.rs
│   ├── assets.rs
│   ├── layout.rs
│   ├── bounds.rs
│   └── skinning.rs
├── materials/
│   ├── mod.rs
│   ├── descriptors.rs
│   ├── instances.rs
│   ├── parameters.rs
│   └── bindings.rs
├── animation/
│   ├── mod.rs
│   ├── clips.rs
│   ├── skeleton.rs
│   ├── pose.rs
│   └── state.rs
├── sdf/
│   ├── mod.rs
│   ├── assets.rs
│   ├── fields.rs
│   └── parameters.rs
└── textures/
    ├── mod.rs
    ├── assets.rs
    └── formats.rs
```

Owns:

- runtime mesh and model content
- material descriptors and instances
- animation clips, skeletons, poses, and runtime content state
- SDF content assets
- texture and content-facing asset metadata

Does not own:

- frame graph execution
- pass scheduling
- editor graph tooling

### 5.4 Future `engine/src/streaming/`

Purpose:

- engine runtime residency, chunking, clipmaps, and streaming caches

```text
engine/src/streaming/
├── mod.rs
├── chunking/
│   ├── mod.rs
│   ├── residency.rs
│   ├── requests.rs
│   └── scheduler.rs
├── clipmaps/
│   ├── mod.rs
│   ├── state.rs
│   ├── updates.rs
│   └── residency.rs
├── caches/
│   ├── mod.rs
│   ├── pages.rs
│   ├── bricks.rs
│   ├── eviction.rs
│   └── keys.rs
└── runtime/
    ├── mod.rs
    ├── state.rs
    └── metrics.rs
```

Owns:

- chunk loading and unloading
- clipmap residency and update logic
- page and brick caches
- eviction policy
- streaming runtime state and metrics

Does not own:

- generic clipmap math
- generic chunk coordinates
- render pass execution

### 5.5 Future `engine/src/animation/`

Purpose:

- engine runtime animation evaluation beyond render-only skinning

```text
engine/src/animation/
├── mod.rs
├── graphs/
│   ├── mod.rs
│   ├── state_machine.rs
│   ├── blend_tree.rs
│   └── evaluation.rs
├── runtime/
│   ├── mod.rs
│   ├── state.rs
│   ├── evaluation.rs
│   └── events.rs
└── extraction/
    ├── mod.rs
    └── render_pose.rs
```

Owns:

- runtime animation logic
- blend and state evaluation
- animation events
- render-facing pose extraction

Does not own:

- frame graph
- shader manager
- editor graph UI

---

## 6. Render Plugin Target Structure

### 6.1 Current owner remains

Render ownership remains under:

- `engine/src/plugins/render/`

### 6.2 Target structure

```text
engine/src/plugins/render/
├── mod.rs
├── README.md
├── domain.rs
├── plugin.rs
├── gfx.rs
├── wgpu_ctx.rs
│
├── frame_graph/
│   ├── mod.rs
│   ├── spec.rs
│   ├── builders.rs
│   ├── registry.rs
│   ├── resources.rs
│   └── executor.rs
│
├── renderer/
│   ├── mod.rs
│   ├── render_flow.rs
│   ├── setup.rs
│   └── graph_execution.rs
│
├── shader_manager/
│   ├── mod.rs
│   ├── registry.rs
│   ├── types.rs
│   └── helpers.rs
│
├── pipelines/
│   ├── mod.rs
│   ├── keys.rs
│   ├── cache.rs
│   └── specialization.rs
│
├── extract/
│   ├── mod.rs
│   ├── views.rs
│   ├── scene.rs
│   ├── materials.rs
│   ├── lighting.rs
│   └── animation.rs
│
├── resources/
│   ├── mod.rs
│   ├── textures.rs
│   ├── buffers.rs
│   └── transient.rs
│
├── submission/
│   ├── mod.rs
│   ├── encoder.rs
│   └── present.rs
│
├── compute/
│   ├── mod.rs
│   ├── dispatch.rs
│   ├── kernels.rs
│   ├── resources.rs
│   └── readback.rs
│
├── graphs/
│   ├── mod.rs
│   ├── material_graph/
│   │   ├── mod.rs
│   │   ├── nodes.rs
│   │   ├── registry.rs
│   │   ├── compiler.rs
│   │   └── types.rs
│   ├── compositor_graph/
│   │   ├── mod.rs
│   │   ├── nodes.rs
│   │   ├── registry.rs
│   │   ├── compiler.rs
│   │   └── types.rs
│   └── shared/
│       ├── mod.rs
│       ├── ids.rs
│       ├── pins.rs
│       └── values.rs
│
└── features/
    ├── mod.rs
    ├── compositor/
    │   ├── mod.rs
    │   ├── plugin.rs
    │   └── runtime.rs
    ├── sdf/
    │   ├── mod.rs
    │   ├── plugin.rs
    │   └── runtime.rs
    ├── lighting/
    │   ├── mod.rs
    │   ├── plugin.rs
    │   └── runtime.rs
    ├── debug_views/
    │   ├── mod.rs
    │   ├── registry.rs
    │   ├── plugin.rs
    │   └── runtime.rs
    └── postprocess/
        ├── mod.rs
        ├── plugin.rs
        └── runtime.rs
```

### 6.3 Render ownership summary

`frame_graph/`

Owns:

- GPU pass and resource dependency graph
- render, compute, and copy pass orchestration model

`renderer/`

Owns:

- top-level render frame flow
- graph execution orchestration

`shader_manager/`

Owns:

- shader registration and lookup
- shader metadata

`pipelines/`

Owns:

- pipeline keys
- specialization
- pipeline cache

`extract/`

Owns:

- ECS and scene to render extraction

`resources/`

Owns:

- textures, buffers, and transient GPU resource model

`submission/`

Owns:

- command encoding, submission, and present flow

`compute/`

Owns:

- compute dispatch helpers
- readback and compute-specific runtime support

`graphs/`

Owns:

- material graph authoring and runtime compilation
- compositor graph authoring and runtime compilation

`features/`

Owns:

- concrete render features built on the render infrastructure

---

## 7. Game Ownership Rules

### 7.1 `games/cavern_hunt/`

Keep game-specific systems here unless proven reusable.

Likely remain game-owned:

- gameplay
- game-specific worldgen
- game-specific geometry graph
- game-specific collision field behavior
- game-specific SDF content logic
- game-specific material graph nodes unless generalized

Extract only when:

- clearly engine-agnostic
- reused or obviously reusable
- stable enough to justify a shared abstraction

---

## 8. Tooling Target Structure

### 8.1 `tools/editor/` or `apps/editor/`

Purpose:

- editor UI and workflows
- runtime inspection
- scene, material, and graph authoring
- debug tooling

```text
tools/editor/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── app/
    ├── panels/
    ├── inspectors/
    ├── graph_editor/
    ├── scene_tools/
    └── asset_tools/
```

### 8.2 `tools/dcc/`

Purpose:

- import and export
- conversion and validation
- offline processing for assets, graphs, materials, and models

```text
tools/dcc/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── import/
    ├── export/
    ├── materials/
    ├── meshes/
    └── validation/
```

---

## 9. Migration Guidance

### 9.1 Immediate additions that fit current repo well

- `foundation/geometry/`
- `foundation/spatial/`
- clearer subdomains under `engine/src/plugins/render/`

### 9.2 Medium-term additions

- `engine/src/content/`
- `engine/src/streaming/`
- `engine/src/animation/`

### 9.3 Later additions

- `tools/editor/` or `apps/editor/`
- `tools/dcc/`

---

## 10. Current-to-Target Mapping Notes

### 10.1 Current render registry and executor areas

Current:

- `engine/src/plugins/render/render_graph_registry/`
- `engine/src/plugins/render/render_executor_registry/`

Target:

- likely fold into `frame_graph/` and `renderer/` depending on exact responsibility

### 10.2 Current `frame_graph.rs`

Current:

- `engine/src/plugins/render/frame_graph.rs`

Target:

- `engine/src/plugins/render/frame_graph/mod.rs` plus split files

### 10.3 Current `pipeline_key.rs`

Current:

- `engine/src/plugins/render/pipeline_key.rs`

Target:

- `engine/src/plugins/render/pipelines/keys.rs`

### 10.4 Current game material graph

Current:

- `games/cavern_hunt/src/domain/material_graph/...`

Target:

- keep game-owned unless generalized enough for `engine/src/plugins/render/graphs/material_graph/`

### 10.5 Current game geometry graph

Current:

- `games/cavern_hunt/src/domain/world/geometry_graph/...`

Target:

- keep game-owned unless it becomes a reusable engine or world-authoring abstraction

---

## 11. Naming Rules

Prefer:

- domain names by responsibility
- subdomain folders with `mod.rs`
- explicit names such as:
  - `frame_graph`
  - `shader_manager`
  - `streaming`
  - `content`
  - `animation`
  - `debug_views`

Avoid:

- `utils`
- `helpers`
- `misc`
- `core`
- `_internal`
- one giant generic graph for everything

---

## 12. Recommended Documentation Sections For This Plan

When documenting this structure elsewhere, prefer these sections:

- Goals
- Ownership Principles
- Top-Level Target Structure
- Foundation Target Structure
- Engine Target Structure
- Render Plugin Target Structure
- Game Ownership Rules
- Tooling Target Structure
- Migration Guidance
- Current-to-Target Mapping Notes
- Naming Rules

---

## 13. Summary

This target architecture is based on these core rules:

- foundation owns reusable primitives
- engine owns runtime domains
- render owns GPU frame production
- games own vertical-slice-specific logic until clearly reusable
- tools own editor and DCC workflows

This structure is intended to let the workspace evolve without forcing:

- render to own everything
- games to accumulate shared engine infrastructure forever
- one generic graph system to model every problem
- tooling concerns to pollute runtime domains
