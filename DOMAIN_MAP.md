# Domain Map

## Purpose

This root document answers: where does this concept belong now, and which owner
retains it after repository-family cutovers?

Canonical long-form authority lives under `docs-site/src/content/docs`.

## Governance documents

```text
Human entrypoint                  -> README.md
AI agent entrypoint               -> AGENTS.md
Programming principles            -> docs-site/src/content/docs/guidelines/programming-principles.md
Repository-family architecture    -> docs-site/src/content/docs/architecture/repository-family-architecture.md
Repository extraction ADR         -> docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
GPU/render ownership ADR          -> docs-site/src/content/docs/adr/accepted/0015-separate-gpu-execution-from-rendering.md
Dependency direction              -> DEPENDENCY_RULES.md
Terminology                       -> GLOSSARY.md
Testing doctrine                  -> TESTING.md
Crate inventory                   -> CRATES.md
Workspace workflow                -> docs-site/src/content/docs/workspace/engineering-workflow.md
Current work                      -> docs-site/src/content/docs/workspace/planning/active-work.md
Roadmap                           -> docs-site/src/content/docs/workspace/planning/roadmap.md
```

## Target repository ownership

```text
Reusable SDF mathematics/queries  -> Crystonix/runen-sdf, package runen-sdf
Reusable ECS semantics            -> target Crystonix/runen-ecs; package topology governed separately
General GPU execution/WGPU        -> Crystonix/runen-gpu, package runen-gpu
Image formation/render semantics  -> Crystonix/runen-render, package runen-render
Reusable UI framework             -> Crystonix/runen-ui workspace; current packages include runenui_core and runenui_runtime
Cross-framework integration       -> Crystonix/runenwerk
Applications and product policy   -> Crystonix/runenwerk
```

Accepted dependency:

```text
RunenRender -> RunenGPU
```

Each framework begins with one public package. Do not create speculative core,
backend, facade, macro, testing, or compatibility packages merely to represent
internal architecture.

Until a clean cutover merges, current source stays at its existing Runenwerk path.
Current location is not permission to preserve accidental ownership.

## Current concept map

```text
Typed ID primitives                -> foundation/id
Typed ID macro support             -> foundation/id_macros
Diagnostics vocabulary             -> foundation/diagnostics
Ratification vocabulary            -> foundation/ratification
Command descriptor/proposal vocabulary -> foundation/commands
Schema vocabulary                  -> foundation/schema
Portable resource references       -> foundation/resource_ref
Domain ratification rules          -> owning domain ratifier

ECS world/query/system runtime      -> domain/ecs, future RunenECS
ECS derive macros                   -> domain/ecs_macros, future RunenECS decision
Generic schedule DAG/planning       -> domain/scheduler, future RunenECS decision
Entity-to-spatial adaptation        -> Runenwerk adapter, not ECS core
Spatial coordinates/indexing        -> domain/spatial, domain/spatial_index

Geometry primitives/queries         -> domain/geometry
SDF fields/queries                  -> domain/sdf until Runenwerk clean cutover; external RunenSDF is authoritative standalone source
SDF world payload/collision policy  -> domain/world_sdf, retained by Runenwerk
SDF-to-render preparation           -> Runenwerk adapter
SDF-to-GPU realization              -> Runenwerk adapter until independent reuse is proven

Scene transform contracts           -> domain/scene
Asset identity/import contracts     -> domain/asset
Shared product contracts            -> domain/product
Graph substrate                     -> domain/graph
Texture product descriptors         -> domain/texture
Material graph/source semantics      -> domain/material_graph
Procgen planning/lowering           -> domain/procgen
Drawing documents/strokes/tiles     -> domain/drawing
Chunk streaming policy              -> domain/chunking
Chunk lifecycle requests/events     -> domain/world_streaming
Chunk/world operation logs          -> domain/world_ops

GPU context/device/queue             -> currently engine/src/plugins/render; future RunenGPU
GPU capabilities/resources/lifetimes -> currently engine/src/plugins/render; future RunenGPU
GPU workload/hazard/submission       -> currently engine/src/plugins/render; future RunenGPU
WGPU shaders/pipelines/readback       -> currently engine/src/plugins/render; future RunenGPU
Low-level GPU surfaces/outcomes       -> currently engine/src/plugins/render; future RunenGPU

Prepared render scene/contributions  -> currently mixed in engine/src/plugins/render; future RunenRender
Render views/logical targets          -> currently mixed in engine/src/plugins/render; future RunenRender
Providers/interactions                -> future RunenRender
Materials/media/emitters              -> future RunenRender prepared rendering contracts
Visibility/transport/radiance caches  -> future RunenRender
Reconstruction/overlays/color         -> future RunenRender
Render-to-GPU lowering                -> future RunenRender using RunenGPU

Runenwerk render/plugin lifecycle     -> retained in Runenwerk
Native windows/event loop             -> retained in Runenwerk
ECS/scene/world/material/SDF adapters -> retained in Runenwerk
UI-to-render bridge                    -> retained in Runenwerk until independent reuse is proven
Shader file discovery/hot reload       -> retained in Runenwerk
Editor/debug/product render policy     -> retained in Runenwerk
Product GPU/render recovery            -> retained in Runenwerk

Current internal UI substrate          -> domain/ui/* until separately governed cutover
External reusable UI authority         -> Crystonix/runen-ui
UI state/layout/text/hit testing        -> RunenUI
Renderer-neutral UI paint output        -> RunenUI
Paint-to-overlay translation            -> Runenwerk adapter

Editor command/session contracts       -> domain/editor/editor_core
Editor runtime preview protocol        -> domain/editor/editor_preview
Editor inspector semantics             -> domain/editor/editor_inspector
Editor scene authoring contracts        -> domain/editor/editor_scene
Editor viewport semantics               -> domain/editor/editor_viewport
Editor shell projection                 -> domain/editor/editor_shell
Editor definition lifecycle             -> domain/editor/editor_definition
Editor persistence formats              -> domain/editor/editor_persistence

Runtime app lifecycle                   -> engine/src/app, engine/src/runtime
Engine plugin integration               -> engine/src/plugins
Network contracts                       -> net/engine_net
Network QUIC transport                  -> net/engine_net_quic
Simulation shared vocabulary            -> net/engine_sim
Replay/history product policy           -> net/engine_history
Runnable editor app wiring              -> apps/runenwerk_editor
Focused drawing app wiring              -> apps/runenwerk_draw
Runtime preview child app               -> apps/runenwerk_runtime_preview
External host integration               -> adapters/*
Native tablet packet normalization      -> adapters/native_tablet_input
```

## Identity placement

Identity belongs to the owner of the invariant:

```text
GpuResourceId       -> RunenGPU runtime identity
RenderProviderId    -> RunenRender semantic runtime identity
EcsEntityId         -> RunenECS/source ECS identity
AssetId             -> asset/source owner
NativeWindowId      -> Runenwerk host identity
```

Runtime IDs are not automatically stable persisted, replay, cache, or network IDs.
Adapters map identities explicitly and preserve provenance.

## Placement rules

- A concept belongs where its invariants are defined and enforced.
- Usage does not imply ownership.
- Framework repositories do not depend on Runenwerk.
- RunenGPU contains no renderer or domain meaning.
- RunenRender depends on RunenGPU and does not own WGPU directly.
- RunenSDF, RunenECS, and RunenUI remain independent of GPU/rendering by default.
- Cross-framework meaning is translated by explicit Runenwerk adapters.
- Keep integration-specific bridges in Runenwerk until independent reuse is proven.
- Do not create a universal shared-core repository or speculative package split to
  avoid explicit mappings.
- Source is deleted from Runenwerk only after the external owner, every active
  consumer, exact revision, provenance, and complete cutover are validated.
