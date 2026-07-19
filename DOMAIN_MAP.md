# Domain Map

## Purpose

This root document answers: where does this concept belong now, and which owner
will retain it after the repository-family cutovers?

Canonical long-form authority lives under `docs-site/src/content/docs`.

## Governance docs

```text
Human entrypoint                  -> README.md
AI agent entrypoint               -> AGENTS.md
Programming principles            -> docs-site/src/content/docs/guidelines/programming-principles.md
Repository-family architecture    -> docs-site/src/content/docs/architecture/repository-family-architecture.md
Repository extraction ADR         -> docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
Dependency direction              -> DEPENDENCY_RULES.md
Terminology                       -> GLOSSARY.md
Testing doctrine                  -> TESTING.md
Crate inventory                   -> CRATES.md
Workspace workflow                -> docs-site/src/content/docs/workspace/start-here.md
Current work                      -> docs-site/src/content/docs/workspace/planning/active-work.md
```

## Target repository ownership

```text
Reusable SDF mathematics/queries  -> future Crystonix/RunenSDF
Reusable ECS and macros           -> future Crystonix/RunenECS
Context-generic schedule planning -> future RunenECS/runen_schedule package
Neutral render graph/planning     -> future Crystonix/RunenRender/runenrender_core
WGPU backend execution/surfaces   -> future Crystonix/RunenRender/runenrender_wgpu
Reusable UI framework             -> Crystonix/RunenUI, separate workstream
Cross-domain integration/products -> Crystonix/Runenwerk
```

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
ECS derive macros                   -> domain/ecs_macros, future RunenECS
Generic schedule DAG/planning       -> domain/scheduler, future RunenECS/runen_schedule
Entity-to-spatial adaptation        -> Runenwerk adapter, not ECS core
Spatial coordinates/indexing        -> domain/spatial, domain/spatial_index

Geometry primitives/queries         -> domain/geometry
SDF fields/queries                  -> domain/sdf, future RunenSDF
SDF world payload/collision policy  -> domain/world_sdf, retained by Runenwerk
SDF render preparation              -> Runenwerk render adapter

Scene transform contracts           -> domain/scene
Asset identity/import contracts     -> domain/asset
Shared product contracts            -> domain/product
Graph substrate                     -> domain/graph
Texture product descriptors         -> domain/texture
Material graph semantics            -> domain/material_graph
Procgen planning/lowering           -> domain/procgen
Drawing documents/strokes/tiles     -> domain/drawing
Chunk streaming policy              -> domain/chunking
Chunk lifecycle requests/events     -> domain/world_streaming
Chunk/world operation logs          -> domain/world_ops

Neutral render graph/planning       -> currently engine/src/plugins/render, future RunenRender core
WGPU resources/execution/surfaces   -> currently engine/src/plugins/render/backend, future RunenRender WGPU
Runenwerk render plugin/lifecycle   -> retained in Runenwerk
ECS/scene/material/SDF render maps  -> retained Runenwerk adapters
Editor/debug render policy          -> retained Runenwerk adapters/apps

Current internal UI substrate       -> domain/ui/* until separately governed cutover
External reusable UI authority      -> Crystonix/RunenUI, separate workstream
Future UI-to-render translation     -> Runenwerk adapter using generic render contracts

Editor command/session contracts    -> domain/editor/editor_core
Editor runtime preview protocol     -> domain/editor/editor_preview
Editor inspector semantics          -> domain/editor/editor_inspector
Editor scene authoring contracts    -> domain/editor/editor_scene
Editor viewport semantics           -> domain/editor/editor_viewport
Editor shell projection             -> domain/editor/editor_shell
Editor definition lifecycle         -> domain/editor/editor_definition
Editor persistence formats          -> domain/editor/editor_persistence

Runtime app lifecycle               -> engine/src/app, engine/src/runtime
Engine plugin integration           -> engine/src/plugins
Network contracts                   -> net/engine_net
Network QUIC transport              -> net/engine_net_quic
Simulation shared vocabulary        -> net/engine_sim
Replay/history product policy       -> net/engine_history
Runnable editor app wiring          -> apps/runenwerk_editor
Focused drawing app wiring          -> apps/runenwerk_draw
Runtime preview child app           -> apps/runenwerk_runtime_preview
External host integration           -> adapters/*
Native tablet packet normalization  -> adapters/native_tablet_input
AI integrations                     -> apps, tools, or adapters
```

## Placement rules

- A concept belongs where its invariants are defined and enforced.
- Usage does not imply ownership.
- Framework cores do not depend on Runenwerk.
- Cross-domain meaning is translated by explicit Runenwerk adapters.
- Do not create a universal shared-core repository to avoid explicit mappings.
- Source is deleted from Runenwerk only after the external owner and cutover are
  validated.
