# Domain Map

## Purpose

This document answers: where does this concept belong? The canonical docs tree is `docs-site/src/content/docs`; this root file is the quick placement map.

## Governance Docs

```text
Architecture doctrine              -> docs-site/src/content/docs/guidelines/runenwerk-architecture.md
Workspace boundaries               -> docs-site/src/content/docs/guidelines/architecture.md
Dependency direction               -> DEPENDENCY_RULES.md
AI-assisted contribution rules     -> AI_GUIDE.md
Terminology                        -> GLOSSARY.md
Testing doctrine                   -> TESTING.md
Crate inventory                    -> CRATES.md
```

## Concept Map

```text
Typed ID primitives                -> foundation/id
Typed ID macro support             -> foundation/id_macros
Diagnostics vocabulary             -> foundation/diagnostics
Ratification vocabulary           -> foundation/ratification
Command descriptor/proposal vocabulary -> foundation/commands
Schema vocabulary                  -> foundation/schema
Domain ratification rules         -> owning domain ratifier
ECS world/query/system runtime      -> domain/ecs
Schedule planning/execution         -> domain/scheduler
Scene transform contracts           -> domain/scene
Geometry primitives/queries         -> domain/geometry
SDF fields/queries                  -> domain/sdf
Spatial coordinates/indexing        -> domain/spatial, domain/spatial_index
Chunk streaming policy              -> domain/chunking
Chunk/world op logs and deltas       -> domain/world_ops
SDF world payload/collision data     -> domain/world_sdf
UI substrate primitives             -> domain/ui/*
UI surface semantics                -> domain/ui/ui_surface
Editor command/session contracts    -> domain/editor/editor_core
Editor inspector semantics          -> domain/editor/editor_inspector
Editor scene authoring contracts    -> domain/editor/editor_scene
Editor viewport semantics           -> domain/editor/editor_viewport
Editor workspace/shell projection   -> domain/editor/editor_shell
Editor persistence formats          -> domain/editor/editor_persistence
Runtime app lifecycle               -> engine/src/app, engine/src/runtime
Engine plugin integration           -> engine/src/plugins
Render graph/runtime execution      -> engine/src/plugins/render
Network contracts                   -> net/engine_net
Network QUIC transport              -> net/engine_net_quic
Simulation shared vocabulary        -> net/engine_sim
Replay/history substrate            -> net/engine_history
Runnable editor app wiring          -> apps/runenwerk_editor
External host integration           -> adapters/*
```

## Rule

A concept belongs where its invariants are defined and enforced. Usage does not imply ownership.
