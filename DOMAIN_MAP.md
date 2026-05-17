# Domain Map

## Purpose

This document answers: where does this concept belong? The canonical docs tree is `docs-site/src/content/docs`; this root file is the quick placement map.

## Governance Docs

```text
Architecture doctrine              -> docs-site/src/content/docs/guidelines/runenwerk-architecture.md
Workspace boundaries               -> docs-site/src/content/docs/guidelines/architecture.md
SDF-first field-product decision   -> docs-site/src/content/docs/adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
SDF-first platform architecture    -> docs-site/src/content/docs/design/accepted/sdf-first-field-world-platform-design.md
Field product target contracts     -> docs-site/src/content/docs/design/accepted/field-product-contracts-diagnostics-and-residency-design.md
Animated SDF procedural animation  -> docs-site/src/content/docs/design/active/sdf-procedural-animation-and-animated-models-design.md
Animated SDF lowering invariant    -> docs-site/src/content/docs/adr/accepted/0011-animated-sdf-authoring-graphs-lower-before-runtime.md
Animated SDF broader proposal      -> docs-site/src/content/docs/adr/proposed/animated-sdf-lowering-and-purpose-specific-products.md
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
Portable external resource references -> foundation/resource_ref
Domain ratification rules         -> owning domain ratifier
ECS world/query/system runtime      -> domain/ecs
Schedule planning/execution         -> domain/scheduler
Scene transform contracts           -> domain/scene
Geometry primitives/queries         -> domain/geometry
Asset identity/catalog/import contracts -> domain/asset
Shared field-product contracts          -> domain/product
SDF fields/queries                  -> domain/sdf
Animated SDF semantic model/lowering -> active design until accepted owner crates exist
Graph substrate                     -> domain/graph
Texture product descriptors         -> domain/texture
Material graph semantics            -> domain/material_graph
Procgen planning lifecycle, generator documents, reservations, lowering contract -> domain/procgen
Generic field-product target design -> docs-site/src/content/docs/design/accepted/field-product-contracts-diagnostics-and-residency-design.md
Drawing documents/strokes/brushes/paper/composition/tile lineage -> domain/drawing
Spatial coordinates/indexing        -> domain/spatial, domain/spatial_index
Chunk streaming policy              -> domain/chunking
Chunk/world op logs and deltas       -> domain/world_ops
SDF world payload/collision data and current SDF field-product specialization -> domain/world_sdf
UI substrate primitives             -> domain/ui/*
UI surface semantics                -> domain/ui/ui_surface
Backend-neutral graph editor contracts -> domain/ui/ui_graph_editor
UI definition/formation framework   -> domain/ui/ui_definition
Editor command/session contracts    -> domain/editor/editor_core
Editor runtime preview protocol     -> domain/editor/editor_preview
Editor inspector semantics          -> domain/editor/editor_inspector
Editor scene authoring contracts    -> domain/editor/editor_scene
Editor viewport semantics           -> domain/editor/editor_viewport
Editor workspace/shell projection   -> domain/editor/editor_shell
Editor definition lifecycle        -> domain/editor/editor_definition
Editor persistence formats          -> domain/editor/editor_persistence
Runtime app lifecycle               -> engine/src/app, engine/src/runtime
Engine plugin integration           -> engine/src/plugins
Render graph/runtime execution      -> engine/src/plugins/render
Network contracts                   -> net/engine_net
Network QUIC transport              -> net/engine_net_quic
Simulation shared vocabulary        -> net/engine_sim
Replay/history substrate            -> net/engine_history
Runnable editor app wiring          -> apps/runenwerk_editor
Focused drawing app wiring          -> apps/runenwerk_draw
Runtime preview child app           -> apps/runenwerk_runtime_preview
External host integration           -> adapters/*
Native tablet packet normalization  -> adapters/native_tablet_input
```

## Rule

A concept belongs where its invariants are defined and enforced. Usage does not imply ownership.
