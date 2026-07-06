# Domain Map

## Purpose

This root document answers: where does this concept belong?

The canonical docs tree is `docs-site/src/content/docs`. This file is only the quick placement map for contributors and agents.

## Governance docs

```text
Human entrypoint                  -> README.md
AI agent entrypoint               -> AGENTS.md
Programming principles            -> docs-site/src/content/docs/guidelines/programming-principles.md
Architecture boundaries           -> docs-site/src/content/docs/guidelines/architecture.md
Architecture doctrine             -> docs-site/src/content/docs/guidelines/runenwerk-architecture.md
UI framework architecture spine   -> docs-site/src/content/docs/architecture/ui-framework-architecture.md
Dependency direction              -> DEPENDENCY_RULES.md
Terminology                        -> GLOSSARY.md
Testing doctrine                   -> TESTING.md
Crate inventory                    -> CRATES.md
Workspace workflow                 -> docs-site/src/content/docs/workspace/start-here.md
AI agent boundaries                -> docs-site/src/content/docs/workspace/ai-agent-boundaries.md
```

## Concept map

```text
Typed ID primitives                -> foundation/id
Typed ID macro support             -> foundation/id_macros
Diagnostics vocabulary             -> foundation/diagnostics
Ratification vocabulary            -> foundation/ratification
Command descriptor/proposal vocabulary -> foundation/commands
Schema vocabulary                  -> foundation/schema
Portable resource references       -> foundation/resource_ref
Domain ratification rules          -> owning domain ratifier
ECS world/query/system runtime     -> domain/ecs
Schedule planning/execution        -> domain/scheduler
Scene transform contracts          -> domain/scene
Geometry primitives/queries        -> domain/geometry
Asset identity/import contracts    -> domain/asset
Shared product contracts           -> domain/product
SDF fields/queries                 -> domain/sdf
Graph substrate                    -> domain/graph
Texture product descriptors        -> domain/texture
Material graph semantics           -> domain/material_graph
Procgen planning/lowering          -> domain/procgen
Drawing documents/strokes/tiles    -> domain/drawing
Spatial coordinates/indexing       -> domain/spatial, domain/spatial_index
Chunk streaming policy             -> domain/chunking
Chunk lifecycle requests/events    -> domain/world_streaming
Chunk/world operation logs         -> domain/world_ops
SDF world payload/collision data   -> domain/world_sdf
UI substrate primitives            -> domain/ui/*
UI surface semantics               -> domain/ui/ui_surface
UI story proof orchestration       -> domain/ui/ui_story
Graph editor contracts             -> domain/ui/ui_graph_editor
UI definition framework            -> domain/ui/ui_definition
UI app integration proof bridge    -> domain/ui/ui_app_integration
SDF UI target/projection evidence  -> docs-site/src/content/docs/architecture/ui-framework-architecture.md plus game-runtime/SDF target docs
Editor command/session contracts   -> domain/editor/editor_core
Editor runtime preview protocol    -> domain/editor/editor_preview
Editor inspector semantics         -> domain/editor/editor_inspector
Editor scene authoring contracts   -> domain/editor/editor_scene
Editor viewport semantics          -> domain/editor/editor_viewport
Editor workspace/shell projection  -> domain/editor/editor_shell
Editor definition lifecycle        -> domain/editor/editor_definition
Editor persistence formats         -> domain/editor/editor_persistence
Runtime app lifecycle              -> engine/src/app, engine/src/runtime
Engine plugin integration          -> engine/src/plugins
Render graph/runtime execution     -> engine/src/plugins/render
Network contracts                  -> net/engine_net
Network QUIC transport             -> net/engine_net_quic
Simulation shared vocabulary       -> net/engine_sim
Replay/history substrate           -> net/engine_history
Runnable editor app wiring         -> apps/runenwerk_editor
Focused drawing app wiring         -> apps/runenwerk_draw
Runtime preview child app          -> apps/runenwerk_runtime_preview
External host integration          -> adapters/*
Native tablet packet normalization -> adapters/native_tablet_input
AI integrations                    -> apps, tools, or adapters
```

## Rule

A concept belongs where its invariants are defined and enforced. Usage does not imply ownership.
