---
title: Procedural Content and Simulation Workflow Plan
description: Cross-domain SDF-first feature plan for procedural authoring, material/texturing, particles, physics, animation, simulation, runtime, and Editor integration.
status: active
owner: workspace
layer: cross-domain
canonical: true
lifecycle_exception: active_phase_evidence
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_designs:
  - ../accepted/sdf-first-field-world-platform-design.md
  - ./editor-asset-pipeline-and-content-workflow-design.md
  - ./semantic-graph-ir-and-compilation-design.md
  - ./gameplay-graph-atr-ir-and-ecs-lowering-design.md
  - ../implemented/workspace-viewport-expression-upgrade-design.md
  - ../accepted/runenwerk-editor-coordination-semantic-model.md
  - ../implemented/editor-tool-suite-registry-and-workbench-host-design.md
related_roadmaps:
  - ../../apps/runenwerk-editor/roadmap.md
  - ../../engine/plugins/render/docs/roadmap.md
related:
  - ../../domain/world-sdf/README.md
  - ../../domain/world-ops/README.md
---

# Procedural Content and Simulation Workflow Plan

## Lifecycle Note

This document remains active because it coordinates multiple unfinished
procedural, material, physics, animation, VFX, and simulation tracks across
domain, engine/runtime, Runenwerk app-integration, and Editor Tool Suite
boundaries. It is not an Editor semantic-ownership plan. Status notes for
completed slices are phase evidence only; they do not mean the full workflow
plan is implemented.

## Purpose

Define the feature-complete plan for Runenwerk's procedural authoring workflows:

- SDF modeling;
- SDF/world texturing and material channels;
- procedural texturing;
- triplanar mapping;
- PBR material products;
- Texture2D, Texture3D, and volume/field texture workflows;
- particles and VFX;
- physics and collision;
- animation and procedural motion;
- simulation/world processes such as fluids, snow, sediment, erosion, and material transport.

This plan is SDF-first and field-world-first. Mesh, GLB, and imported material workflows may exist as reference and compatibility paths, but they are not the engine's primary world-authoring substrate.

The normalized ownership rule is “the semantic owner owns the product; the Editor integrates the tool.” Domain/runtime tracks therefore establish authored truth, commands, ratification, formed products, execution, and diagnostics before Editor providers consume those contracts. Stable Tool Suite keys/provider families are the Editor extension mechanism; central product enums are not.

## Repo Truth Baseline

Implemented today:

- Standalone `dornglut/runen-sdf` owns reusable signed-field mathematics and CPU query semantics.
- `domain/world_ops` owns governed world-operation records, invalidation, dirty/build coordination, and changed-region mechanics.
- `domain/world_sdf` owns formed field/chunk/page/brick products, field-preview payload contracts, and collision/query readiness.
- `domain/graph` owns Runenwerk-authored port-graph structure only; semantic graph domains retain their own node meaning, ratification, lowering, and formed products.
- `engine/src/plugins/world` owns authoritative chunked SDF runtime integration and world build/execution plumbing.
- `engine/src/plugins/render` owns rendering execution, shader/resource realization, dynamic product targets, material feature preparation, and GPU-facing preview integration.
- `domain/material_graph` owns material graph documents, semantic node contracts, ratification, deterministic lowering, source maps, specialization/product metadata, and formed material products.
- `domain/texture` owns Texture2D/Texture3D descriptors, sampler/color-space/compression metadata, generated texture lineage, and texture ratification.
- Runenwerk Material Lab now has source-backed graph workflows, formed preview products, generated shader/render handoff, product publication, concrete rendered material preview, resource-resolution diagnostics, and scene-material integration while keeping material source truth in its owner.
- Runenwerk texture preview has concrete Texture2D/Texture3D providers and KTX2-backed dynamic preview/upload integration; unsupported resource states remain diagnostic rather than silently accepted.
- `domain/procgen` owns deterministic generator documents, seed/scope/version lineage, ratification, reservations, lowering to `world_ops`, product descriptors, bake contracts, explanation data, and changed-region semantics.
- The Procgen baseline is implemented through Phase 6D: concrete graph/preview providers, CPU field-preview formation, product publication, query-snapshot publication, viewport overlay integration, bake/rollback, persistence, and runtime-preview reload classification exist.
- P1 SDF authoring exists through scene-facing authoring adapters plus `world_ops`/`world_sdf`: command-backed operation/layer documents, SDF graph lowering, deterministic CPU field previews, commit/invalidation, and concrete field/SDF providers.
- The implemented Tool Suite Registry / Workbench Host supplies stable surface keys, provider-family narrowing, provider-owned graph routing, and host capability policy for Editor integration.
- `ui_composition` owns targets, roots, regions, mounted units, structural transactions/history, and structural persistence.

Current migration residue, not future prerequisites:

- `editor_core::DocumentKind` still enumerates many product families under #737. Existing variants are predecessor-shaped compatibility/product taxonomy, not the extension point for a new domain.
- legacy `ToolSurfaceKind` variants and persisted mappings still cover many product surfaces, while normal Tool Suite identity is stable-key-based.
- `SurfaceLocalAction` / `EditorDomainMutation` still contain some concrete Material/SDF action branches. Future domains must not deepen that generic-shell semantic enumeration merely to gain a provider.
- `M6WorkspaceProvider` remains diagnostic/fallback scaffolding; it is not capability readiness or semantic authority.

Missing today:

- no accepted/implemented `domain/particles`;
- no accepted/implemented general `domain/physics`;
- no accepted/implemented `domain/animation`;
- no accepted/implemented `domain/simulation_process`;
- no implemented gameplay event/action/state/quest + gameplay-graph product stack sufficient for runtime lowering;
- no concrete particle, physics, animation, simulation-process, or gameplay product providers backed by those missing owner contracts;
- no claim that all P3/P9 material/SDF texturing exit criteria or the full PBR capability matrix are complete;
- no post-6D Procgen breadth claim for caves, structures, scatter, worker/GPU realization, or package-level persistent caches.

## Governing Model

Procedural authoring follows this pipeline:

```text
Authored procedural document
  -> structural validation
  -> domain ratification
  -> normalized semantic IR
  -> deterministic lowering or formation plan
  -> formed product
  -> asset catalog artifact/revision
  -> runtime/preview instantiation
  -> expression products for viewport/tool surfaces
```

Runtime must not interpret editor-authored graphs every frame.

Authoring graphs are intent. Formed products are runtime-facing truth.

Editor integration starts after an owner contract exists:

```text
owner document/product
  -> owner command + ratification
  -> formed product / owner observation
  -> Runenwerk app adapter or runtime handoff
  -> Editor provider projection
  -> provider-owned interaction proposal
```

A new product suite registers stable surface keys/provider families and concrete providers. It does not gain legitimacy by adding a central document/surface/action enum variant.

## Closed Default Decisions

These defaults close the first implementation direction. Change them only by updating this design or adding an ADR.

- SDF modeling is the primary world-authoring path. Mesh, GLB, and imported material workflows are compatibility/reference paths.
- Material graph starts with SDF/field preview, PBR parameter products, procedural texture nodes, triplanar coordinates, and field material-channel output. It does not start with mesh material import.
- Procedural texture generation starts with deterministic domain formation and cached products. GPU generation is a runtime optimization behind the same formed product contract.
- Procedural generation starts from the accepted `docs-site/src/content/docs/domain/procgen/README.md` contract: graph-backed generator documents, deterministic seed/scope/version lineage, authored overlay preservation, product-job outputs, and server-validated authority before concrete provider/runtime code.
- Texture3D is a first-class volume/field texture product with dimension, color-space, sampler, compression, channel, and slice/mip inspection metadata.
- Particles start with deterministic authored emitter and simulation contracts, SDF/field spawn/collision, and editor preview. GPU compute/render integration is an engine backend and must not change authored particle documents.
- Physics starts with `world_sdf` collision readiness, rigid/kinematic/character body contracts, collider descriptors, layer/mask/material authoring, and debug surfaces. A concrete external solver adapter is engine-owned.
- Animation starts with clips, typed curves, timeline, state/blend graphs, procedural motion, events, bindings, and SDF/world-aware motion hooks. Skeletal animation is supported by contracts, but SDF/world-aware procedural motion is first-class.
- World processes start with bounded preview layers and explicit bake/commit into governed `world_ops` records.

## Required Design Follow-Ups

Before implementation starts on a feature track, create or update the owning design/domain docs below. These are not optional polish; they are the contracts that stop editor providers from inventing private behavior.

- `docs-site/src/content/docs/domain/material-graph/README.md`
  - define authored material graph documents, node catalog boundaries, ratification, PBR parameter schema, field/material outputs, source maps, and formed material products.
- `docs-site/src/content/docs/domain/texture/README.md`
  - define Texture2D, Texture3D/volume, generated texture products, sampler/color-space/compression policy, cache keys, and preview/inspection contracts.
- `docs-site/src/content/docs/domain/procgen/README.md`
  - accepted and implemented for the baseline through Phase 6D: deterministic document/seed/scope contracts, lowering, provider/runtime preview, formed CPU field products, publication/query snapshots, bake/rollback, persistence, and reload classification. Further generator families or execution/cache scaling require a new owner activation rather than reopening a generic Editor M6.2 prerequisite.
- `docs-site/src/content/docs/domain/particles/README.md`
  - define emitter documents, particle graph semantics, simulation step contracts, SDF/field coupling, formed particle products, and preview determinism.
- `docs-site/src/content/docs/domain/physics/README.md`
  - define bodies, colliders, constraints, triggers, physics materials, collision product readiness, authority rules, and solver adapter boundaries.
- `docs-site/src/content/docs/domain/animation/README.md`
  - define clips, curves, timelines, state/blend graphs, procedural motion, skeletal pose contracts, events, bindings, and source maps.
- `docs-site/src/content/docs/domain/simulation-process/README.md`
  - define material transport/world-process contracts, timescale tiers, preview layers, solver budgets, changed regions, and bake/rollback behavior.

## Feature Tracks

### Track A - SDF Modeling

Owning domains:

- standalone `dornglut/runen-sdf` for reusable field math and CPU queries;
- `domain/world_ops` for edit operation records, invalidation, and build queues;
- `domain/world_sdf` for formed chunk/page/brick payloads;
- a future Runenwerk-owned authoring package only if scene/editor SDF authoring grows beyond `domain/editor/editor_scene`; it must not duplicate RunenSDF field semantics.

Required features:

- primitive creation for sphere, box, capsule, cylinder, torus, plane, and domain-warped forms;
- boolean operations: union, subtract, intersect, smooth union, smooth subtract, smooth intersect;
- brush workflows: add, subtract, smooth, sharpen, blend, stamp, density deform;
- non-destructive layer stack with explicit write targets;
- brush falloff, radius, hardness, symmetry, repeat, mirror, and domain warp controls;
- operation history with deterministic seeds and affected bounds;
- surface projection and field picking through SDF query contracts;
- SDF graph documents for procedural field construction;
- field product previews for distance, gradient, normal, occupancy, support, and material channels.

Implementation targets:

- `domain/editor/editor_scene/src/sdf_authoring/`
  - scene-facing SDF authoring contracts and command adapters.
- `domain/world_ops/src/operations.rs::Operation`
  - extend operation vocabulary only when a new edit cannot be represented by existing CSG/material/deform operations.
- `domain/world_ops/src/build_graph.rs::BuildGraphPhase`
  - add phase distinctions only when field, material, collision, or render formation require separate scheduling.
- `apps/runenwerk_editor/src/editor_features/viewport/sdf_tools.rs`
  - SDF brush, stamp, surface-pick, and preview tools.
- `apps/runenwerk_editor/src/shell/providers/sdf_graph_canvas.rs::SdfGraphCanvasProvider`
  - graph editing over `domain/graph` plus SDF semantic descriptors.
- `apps/runenwerk_editor/src/shell/providers/field_layer_stack.rs::FieldLayerStackProvider`
  - layer ordering, write targets, operation visibility, and diagnostics.

### Track B - Materials, Texturing, and PBR

Owning domains:

- `domain/material_graph` for material graph semantics, ratification, lowering, and formed material products;
- `domain/texture` for texture asset descriptors, color spaces, dimensions, volume texture metadata, sampler policy, compression policy, and generated texture products;
- `domain/world_ops` for material field edits and material-channel invalidation;
- `domain/world_sdf` for material channel masks and formed field payload metadata;
- `engine/src/plugins/render` for render execution, shader specialization, resource binding, and GPU texture upload/runtime caches.

Required material/product families:

- SDF surface material;
- field material channel set;
- PBR material;
- procedural material graph;
- shader/material expression product;
- texture import product;
- generated texture product;
- Texture3D/volume product;
- atlas/array texture product when a material product declares layer-array or tile-packing requirements;
- triplanar material product;
- material preview expression product.

First material slice:

- authored material graph document with stable node ids and source spans;
- PBR scalar/vector parameter nodes for base color, roughness, metallic, normal strength, emissive, opacity/mask, and material id/channel bindings;
- procedural noise, fbm, ramp, remap, clamp, mix, and mask nodes;
- triplanar coordinate node for world/object/local/field-product coordinates;
- SDF/field input nodes for position, normal/gradient, distance, material channel, density, support, and wetness;
- Texture2D and Texture3D sample nodes over catalog-backed texture products;
- formed material product with parameter schema, source map, specialization fragment, diagnostics, and cache key;
- material preview on an SDF sphere, SDF box, plane, and one formed field product.

Required features:

- PBR parameter model: base color, roughness, metallic, normal, emissive, ambient occlusion, height/displacement with explicit target-capability diagnostics, opacity/mask, and material id/channel bindings;
- SDF-aware shading inputs: position, normal, gradient, distance, curvature/ambient-occlusion approximations with explicit target-capability diagnostics, material channel, density, hardness, wetness, support, and provenance/debug channels;
- triplanar mapping over world, object, local, and field-product coordinates;
- procedural texture nodes: noise, fbm, cellular, voronoi, ridged noise, gradient/ramp, remap, clamp, mix, mask, erosion/weathering masks, slope/height/curvature masks;
- Texture2D import with color-space and compression policy;
- Texture3D and volume texture assets for density, noise, masks, and material volumes;
- generated texture baking/cache products for expensive procedural graphs;
- material graph lowering to shader/material expression products;
- material graph lowering to field material channel products where the output changes world matter;
- material diagnostics for unsupported nodes, cyclic graphs, illegal writes, missing texture products, invalid PBR ranges, and unsupported runtime target.

Out of first material slice but still required:

- full texture-array and atlas packing workflows;
- material layering beyond a single explicit layer stack;
- renderer-specific shader optimization beyond stable specialization fragments;
- foreign mesh material conversion beyond reference preview.

Implementation targets:

- `domain/material_graph/src/authored.rs::MaterialGraphDocument`
  - authored semantic document using `domain/graph::GraphDefinition` for structure.
- `domain/material_graph/src/catalog.rs::MaterialNodeCatalog`
  - node descriptors for SDF, field, procedural texture, PBR, and render-expression nodes.
- `domain/material_graph/src/ratification.rs::ratify_material_graph`
  - semantic graph ratification with material issue codes.
- `domain/material_graph/src/lowering.rs::lower_material_graph`
  - current first-slice lowering to formed material descriptors; render-expression and field-material-channel lowering remain later M6.1/M6.3 work.
- `domain/material_graph/src/formed.rs::FormedMaterialProduct`
  - formed material product, parameter schema, source map, and specialization key fragment.
- `domain/texture/src/`
  - texture source descriptors, generated texture products, Texture3D/volume descriptors, sampler policy, color space, compression, and cache metadata.
- `engine/src/plugins/render/frame/contributions.rs::PreparedMaterialFeatureContribution`
  - become the render handoff for formed material products and material instance parameters.
- `engine/src/plugins/render/renderer/render_flow/provenance.rs::material_specialization_fragment_hash`
  - continue folding material feature signatures into pipeline keys.
- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs::MaterialGraphCanvasProvider`
  - material graph editing.
- `apps/runenwerk_editor/src/shell/providers/material_inspector.rs::MaterialInspectorProvider`
  - parameter editing and PBR validation.
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs::MaterialPreviewProvider`
  - SDF sphere/box/plane/field-product preview with diagnostic overlays.
- `apps/runenwerk_editor/src/shell/providers/texture_viewer.rs::TextureViewerProvider`
  - Texture2D, Texture3D slice, mip, channel, and color-space inspection.

### Track C - Procedural Generation Workflows

Owning domains:

- `domain/procgen` for procedural source descriptors, seed contracts, generator graphs, rule sets, planning lifecycle, reservations, explanation data, and formation plans;
- `domain/world_ops` for generated world edit windows and invalidation;
- `domain/world_sdf` for formed field products;
- `domain/asset` for procedural source identities and cache artifacts.

Required features:

- seed-driven deterministic generation;
- terrain/cave/structure/stamp generator documents;
- generator graphs that lower into operation plans, not live editor graph traversal;
- prototype, candidate, reservation, instance-plan, and realization lifecycle
  vocabulary for bounded procgen output;
- biome/material rule layers;
- scatter/distribution rules;
- erosion/weathering generation passes;
- preview windows with bounded spatial scope;
- candidate and reservation explanations for accepted, rejected, moved, or
  blocked planned content;
- regeneration with stable seeds and changed-region invalidation;
- bake-to-operations and bake-to-field-product workflows.

First procgen slice:

- bounded region generator document with explicit seed, version, input products,
  write targets, and first-slice planning lifecycle metadata;
- noise/height and material-rule generator nodes, with cave/stamp/scatter
  deferred;
- material rule layer that writes material-channel operations;
- reservation and conflict diagnostics for generated terrain/material claims;
- preview window with explanation data and changed-region diagnostics;
- bake-to-`world_ops::OperationRecord` and bake-to-field-product commands;
- deterministic replay test proving identical inputs form identical operation windows.

Implementation targets:

- `domain/procgen/src/document.rs::ProcgenDocument`
  - authored procedural source documents.
- `domain/procgen/src/lowering/world_ops.rs::lower_procgen_to_world_ops`
  - lowers procedural intent to deterministic `world_ops::OperationRecord` windows.
- `domain/procgen/src/ratification.rs::ratify_procgen_document`
  - rejects nondeterministic, unbounded, unsupported, conflicting, or illegal write-target generation.
- `apps/runenwerk_editor/src/shell/providers/procgen_graph_canvas.rs::ProcgenGraphCanvasProvider`
  - procedural graph editing.
- `apps/runenwerk_editor/src/shell/providers/procgen_preview.rs::ProcgenPreviewProvider`
  - bounded preview and bake controls.

### Track D - Particles and VFX

Owning domains:

- future `domain/particles` for emitter definitions, particle simulation contracts, event inputs, field-query coupling, and particle output products;
- `engine/src/plugins/render` for GPU buffer/texture execution and VFX render products;
- `domain/world_sdf` and field query contracts for collision and spawn/surface sampling.

Required features:

- backend-neutral particle simulation contract with explicit CPU preview behavior and a milestone-gated GPU backend decision;
- emitter documents and particle graph documents;
- spawn shapes: point, sphere, box, surface, SDF volume, field threshold, event burst;
- procedural emission driven by noise, curves, material channels, and world events;
- field collision and SDF surface interaction;
- attractors, forces, turbulence, curl noise, drag, gravity, and vector-field forces;
- ribbon/trail/sprite/mesh-like impostor output products;
- deterministic preview controls and scrubbing where possible;
- VFX diagnostics for particle counts, bounds, simulation cost, and missing field products.

First particle slice:

- CPU deterministic emitter preview with fixed seed, fixed timestep, and bounded count;
- spawn from point, box, sphere, SDF surface, and field threshold;
- force nodes for gravity, drag, noise/turbulence, and vector-field sampling;
- collision query against `world_sdf` readiness contracts;
- sprite output product and diagnostic bounds;
- formed particle product contract that can later target GPU compute without changing authored documents.

Implementation targets:

- future `domain/particles/src/authored/emitter.rs::ParticleEmitterDocument`
  - authored emitter and particle graph definitions.
- future `domain/particles/src/simulation/contract.rs`
  - simulation step, seed, bounds, and field-query contract.
- future `domain/particles/src/formed/product.rs`
  - formed particle simulation and render product descriptors.
- `engine/src/plugins/particles/`
  - runtime plugin for particle simulation resources and render feature publication.
- `apps/runenwerk_editor/src/shell/providers/particle_graph_canvas.rs::ParticleGraphCanvasProvider`
  - particle graph editing.
- `apps/runenwerk_editor/src/shell/providers/particle_preview.rs::ParticlePreviewProvider`
  - playback, scrubbing, and diagnostics.

### Track E - Physics and Collision

Owning domains:

- future `domain/physics` for rigid body, character, contact, constraint, trigger, and collision product contracts;
- `domain/world_sdf` for SDF collision query readiness and sweep outcomes;
- `domain/world_ops` for world mutation requests caused by physics when allowed;
- engine runtime plugins for scheduling and concrete solver integration.

Required features:

- collision product formation separate from render products;
- field-query collision against `world_sdf`;
- rigid bodies, kinematic bodies, triggers, constraints, and character movement contracts;
- physics material properties: friction, restitution, density, buoyancy, hardness, support interaction;
- shape authoring for SDF, primitive, compound, and foreign mesh/reference colliders as compatibility-only colliders when source settings explicitly request them;
- physics debug surfaces for contacts, sweeps, constraints, sleeping, activation, and missing payload readiness;
- authoring for physics layers, masks, joints, anchors, and gameplay-locked chunks;
- explicit authority and write-target rules for simulation-produced world changes.

First physics slice:

- collision product descriptor over `world_sdf` readiness plus primitive/SDF collider descriptors;
- kinematic character sweep against formed SDF world products;
- rigid body, kinematic body, trigger, layer, mask, and physics material documents without solver-specific internals;
- debug visualization for sweeps, contacts, missing collision payloads, and sleeping/activation state;
- explicit command path for any physics-produced world mutation.

Implementation targets:

- future `domain/physics/src/collision/product.rs`
  - collision product descriptors and readiness.
- future `domain/physics/src/body.rs`
  - rigid/kinematic/character body contracts.
- future `domain/physics/src/material.rs`
  - physics material properties linked to field material channels.
- future `domain/physics/src/constraints/`
  - joint, anchor, and contact constraints.
- `engine/src/plugins/physics/`
  - concrete runtime/solver integration and fixed-step scheduling.
- `apps/runenwerk_editor/src/shell/providers/physics_authoring.rs::PhysicsAuthoringProvider`
  - body/collider/material editing.
- `apps/runenwerk_editor/src/shell/providers/physics_debug.rs::PhysicsDebugProvider`
  - contacts, sweeps, readiness, and activation diagnostics.

### Track F - Animation and Procedural Motion

Owning domains:

- future `domain/animation` for clip, timeline, curve, state machine, blend tree, procedural motion, root motion, and animation-event contracts;
- future animation/runtime adapters for concrete playback and sampling;
- future `domain/physics` for secondary motion handoff when animation products declare physics coupling.

Required features:

- timeline and curve editing;
- animation clips and authored curves;
- state machines and blend trees;
- procedural motion nodes;
- root motion;
- animation events;
- skeletal pose contracts when skeleton assets exist;
- SDF/world-aware motion hooks: surface grounding, slope response, field-driven offsets, IK target against SDF surface, and procedural secondary motion;
- preview playback, scrubbing, and onion-skin/ghost views when the animation product declares ghostable samples;
- diagnostics for missing bindings, bad curves, invalid state transitions, unsupported runtime target, and desynced source maps.

First animation slice:

- clip document with typed keyframes, interpolation policy, and source map;
- typed curve document and curve editor provider;
- timeline document for clip ranges, events, and preview playback;
- state machine with explicit transition conditions and diagnostics;
- procedural motion node for SDF surface grounding and slope response;
- animation formed product that omits editor canvas/timeline layout state.

Implementation targets:

- future `domain/animation/src/clip.rs::AnimationClip`
  - clip and keyframe contracts.
- future `domain/animation/src/curve.rs`
  - typed curve data and interpolation policy.
- future `domain/animation/src/state_machine.rs`
  - state, transition, and condition contracts.
- future `domain/animation/src/procedural.rs`
  - procedural motion graph contracts.
- `apps/runenwerk_editor/src/shell/providers/timeline.rs::TimelineProvider`
  - timeline and clip editing.
- `apps/runenwerk_editor/src/shell/providers/curve_editor.rs::CurveEditorProvider`
  - curve authoring.
- `apps/runenwerk_editor/src/shell/providers/animation_graph_canvas.rs::AnimationGraphCanvasProvider`
  - state machine, blend tree, and procedural motion graph editing.

### Track G - Simulation and World Processes

Owning domains:

- future `domain/simulation_process` for shared water, snow, sediment, erosion, weathering, material transport, timescale, preview-layer, and bake contracts;
- `domain/world_ops` for governed mutation records;
- `domain/world_sdf` for field products and collision/query payloads.

Required features:

- material transport contracts for water, snow, sediment, deposition, compaction, melt/freeze, and erosion;
- local interactive simulation and background world-process simulation;
- explicit timescale and activation tiers;
- local preview layers separate from ratified authored layers;
- bake/commit workflows from simulation preview into governed world operations;
- diagnostics for changed regions, product freshness, solver budget, and unsupported scope.

First simulation-process slice:

- material transport preview document with bounded region scope;
- explicit timescale class and solver budget;
- sediment, snow, wetness, and erosion material-channel operation outputs;
- preview layer distinct from authored layers and last valid formed products;
- bake/commit into governed `world_ops` records with changed-region diagnostics;
- rollback path that preserves authored layers and prior valid products.

Implementation targets:

- future `domain/simulation_process/src/material_transport.rs`
  - material exchange operation contracts.
- future `domain/simulation_process/src/timescale.rs`
  - rate class and activation policy contracts.
- `apps/runenwerk_editor/src/shell/providers/simulation_preview.rs::SimulationPreviewProvider`
  - preview, bake, rollback, and diagnostics.

## Milestone-To-Track Matrix

This matrix makes feature coverage explicit. A milestone cannot close unless its required tracks have tests, diagnostics, source lineage, and failed-product preservation for the listed scope.

| Milestone | SDF modeling | Materials/textures | Procgen | Particles | Physics | Animation | World processes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P0 | field products + owner adapters | asset/product kinds + tool-suite surfaces | owner contract + product kinds | owner contract + product kinds | owner contract + product kinds | owner contract + product kinds | owner contract + product kinds |
| P1 | primitives, brushes, layers, SDF graph, field previews | material-channel overlays | not required | not required | collision-readiness display only | not required | not required |
| P2 | field inputs for material graph | material graph, PBR, triplanar, Texture2D, Texture3D, generated cache | not required | not required | not required | not required | not required |
| P3 | SDF/field material previews | render handoff, previews, procedural texture cache | not required | not required | not required | not required | not required |
| P4 | generated SDF/world operations | material rule layers | generator documents, bounded preview, bake | not required | not required | not required | not required |
| P5 | SDF spawn/collision queries | particle material/debug channels | declared event inputs | emitter docs, graph, preview, formed products | field collision readiness | not required | not required |
| P6 | collision products over SDF world | physics material channel links | not required | collision coupling only | bodies, colliders, constraints, solver boundary, debug | not required | mutation authority |
| P7 | SDF motion hooks | not required | not required | event coupling only | secondary-motion handoff only | clips, curves, timeline, state/blend, procedural motion | not required |
| P8 | field mutation and preview layers | material-channel transport | process-generated operations | world event coupling | authority/write targets | event coupling | transport, erosion, snow, water, sediment, bake/rollback |
| P9 | integrated production loop | integrated production loop | integrated production loop | integrated production loop | integrated production loop | integrated production loop | integrated production loop |

## Milestone Sequence

### P0 - Structural Prerequisites

Close editor document tabs, provider routing, scoped modes, asset catalog, field-product formation, and viewport expression products.

Exit criteria:

- product domains expose owner-defined authored/product identities and contracts before Editor integration;
- asset/product catalogs can represent the implemented product families without making Editor taxonomy their semantic owner;
- new Editor tool surfaces register through stable Tool Suite keys/provider families and do not require new central product enum variants;
- viewport/tool surfaces consume typed expression/products, not renderer-private textures or live editor graph state;
- `python3 tools/docs/validate_docs.py` passes.

Current status: the shared Tool Suite/provider/workbench substrate, persisted layout support, asset/product taxonomy, product publication/query-snapshot paths, and fail-closed provider routing exist. Existing `DocumentKind` and `ToolSurfaceKind` product variants are retained migration/compatibility evidence under ADR 0025, not P0 requirements for future domains.

### P1 - SDF Modeling Core

Deliver SDF primitives, brush editing, operation layers, SDF graph authoring, and formed field previews.

Status after the 2026-05-09 P1 closeout: complete for the CPU/editor-surface boundary. Authored operation layers and source-backed SDF graph documents lower through the same normalized operation-window path; all P1 boolean intents lower to governed `world_ops` records; commits append to an app-held operation log, mark dirty chunks, and form deterministic CPU field-preview products for scalar distance, vector gradient, occupancy, and material channels. Renderer/GPU overlays remain deferred to P3.

Exit criteria:

- users can create, edit, reorder, disable, preview, and commit SDF operations;
- SDF operation changes invalidate affected field products through `domain/world_ops`;
- preview surfaces show distance, normal/gradient, occupancy, and material-channel overlays.

### P2 - Procedural Material and Texture Foundation

Deliver material graph and texture domain contracts before broad feature breadth.

Exit criteria:

- `domain/material_graph` and `domain/texture` exist with ratifiers and formed product descriptors;
- PBR parameters, triplanar mapping, procedural nodes, Texture2D, Texture3D, and generated texture cache metadata are representable;
- first-slice material graphs can be authored, ratified, lowered, previewed, and rejected with source-mapped diagnostics;
- material graph lowering produces source maps and diagnostics.

Status: the original descriptor/domain foundation is complete and current implementation has advanced beyond it. Source-backed Material Lab workflows, deterministic material lowering, generated shader artifacts, formed preview products, rendered material-preview targets, product publication, resource binding diagnostics, and KTX2-backed Texture2D/Texture3D preview/upload paths exist. This closes the old “descriptor-only/no GPU preview” description, but does not by itself claim the complete P3 field-material/triplanar/PBR capability matrix.

### P3 - SDF/Field Texturing and PBR Preview

Connect material products to SDF/field previews and render feature contributions.

Exit criteria:

- material previews can render SDF primitives and field products;
- triplanar mapping works against SDF/world coordinates;
- procedural textures can be generated or cached;
- PBR parameter changes hot reload safely into preview when the preview capability matrix permits it.

Status: rendered Material Lab preview and texture GPU-preview/resource handoff now exist, so P3 is no longer an “add any rendered preview” milestone. Remaining P3 closure must be judged against the field/SDF material handoff, procedural texture/cache behavior, triplanar/world-coordinate behavior, and explicit PBR capability matrix rather than reimplementing the existing preview spine.

### P4 - Procedural World Generation

Deliver seed-driven generator documents and bounded preview/bake workflows.

Exit criteria:

- a generator graph can form deterministic world operation windows;
- bounded preview and bake-to-world workflows exist;
- invalidation and rebuild diagnostics show changed regions and products.

Status: the baseline first slice is complete through Procgen Phase 6D, including deterministic graph documents, bounded CPU preview, lowering, changed-region/explanation data, publication, bake/rollback, persistence, and reload classification. Additional terrain/cave/structure/scatter breadth or GPU/worker realization is separate follow-up product work.

### P5 - Particles and VFX

Deliver particle emitter documents, particle graph authoring, SDF/field collision, and preview.

Exit criteria:

- emitters can spawn from SDF surfaces and field volumes;
- particle products can preview in editor surfaces;
- authored particle documents are unchanged by CPU/GPU backend choice;
- runtime diagnostics expose counts, bounds, and missing field-product readiness.

### P6 - Physics and Collision Authoring

Deliver physics domain contracts, collision product formation, rigid/character authoring, and debug surfaces.

Exit criteria:

- physics authoring works for bodies, colliders, materials, layers, masks, constraints, and triggers;
- `world_sdf` collision readiness is visible in the editor;
- solver-specific runtime state does not leak into `domain/physics` authored documents or formed products;
- simulation preview does not mutate authored documents without explicit commands.

### P7 - Animation and Procedural Motion

Deliver animation clips, curves, timeline, state/blend graphs, and SDF/world-aware procedural motion hooks.

Exit criteria:

- clips, curves, state machines, and blend trees can be authored and previewed;
- procedural motion can query SDF/world products through declared contracts;
- editor timeline, canvas, and panel state are omitted from formed animation products;
- animation events and root motion have source lineage and diagnostics.

### P8 - World Processes and Material Transport

Deliver simulation workflows for erosion, snow, water, sediment, accumulation, and material exchange.

Exit criteria:

- simulation preview layers are distinct from ratified authored layers;
- bake/commit creates governed world operations;
- rollback preserves authored layers and prior valid products;
- diagnostics expose changed regions, timescale class, solver budget, and product freshness.

### P9 - Integrated Procedural Production Workflow

Deliver a complete production loop across SDF modeling, procedural material/texturing, particles, physics, animation, world processes, runtime preview, and publishing.

Exit criteria:

- users can author a procedural scene from SDF geometry, procedural PBR materials, generated textures, particles, physics, animation, and world processes;
- every formed product has source lineage, diagnostics, cache/rebuild behavior, and hot reload boundaries;
- failed generation/import/lowering preserves the last valid product;
- full milestone validation gates pass.

## Editor Workspace Additions

Required workspace profiles:

- `SDF Modeling`;
- `Materials`;
- `Textures`;
- `Procedural Generation`;
- `Particles`;
- `Physics`;
- `Animation`;
- `Simulation`;
- `Debug`.

Required panels/tool surfaces:

- SDF Graph Canvas;
- SDF Brush Browser;
- Field Layer Stack;
- Field Product Viewer;
- Material Graph Canvas;
- Material Inspector;
- Material Preview;
- Texture Viewer;
- Texture3D/Volume Viewer;
- Procgen Graph Canvas;
- Procgen Preview;
- Particle Graph Canvas;
- Particle Preview;
- Physics Authoring;
- Physics Debug;
- Timeline;
- Curve Editor;
- Animation Graph Canvas;
- Simulation Preview;
- Simulation Diagnostics.

All of these must extend the shared provider and tool-surface framework. None should create a private side-channel interaction model.

## Procedural Workflow Doctrine

Procedural workflows are first-class authoring, not hidden runtime tricks.

Rules:

- procedural sources are authored assets with typed ids and revisions;
- seeds are explicit;
- random generation must be deterministic for fixed seed, inputs, and version;
- expensive procedural outputs are formed products with cache keys;
- generated products keep source lineage;
- preview layers are not silently promoted to authored truth;
- bake/commit workflows produce commands or world operations;
- runtime execution consumes formed products, not editor graph state;
- diagnostics must explain every rejected graph, failed generation, stale product, and unsupported target.

## Remaining Decision Gates

These are real decisions that must close at the named milestone. They are intentionally not open-ended roadmap items.

- P2 baseline decisions are closed by current implementation: Material Lab lowers to formed material/shader artifacts and the Editor uses KTX2-backed dynamic Texture2D/Texture3D preview/upload integration. Broader format/compression support remains texture/render product evolution, not an open prerequisite for the existing baseline.
- P3: close the explicit PBR preview capability matrix for height/displacement, ambient occlusion, opacity/mask, and normal handling so unsupported outputs fail with explicit diagnostics.
- P4 baseline is closed through Procgen Phase 6D. Choose any additional procedural generation families (terrain breadth, cave, structure, scatter, or others) only in a separately activated Procgen/world product slice.
- P5: choose whether the first runtime particle backend after CPU preview is GPU compute, indirect draw expansion, or CPU simulation with GPU upload. Authored particle documents must remain backend-neutral.
- P6: choose the first concrete physics runtime adapter or an in-house minimal solver. `domain/physics` remains solver-neutral either way.
- P7: choose the first skeletal pose/skeleton asset contract if skeletal assets become part of the first shipping workflow.
- P8: choose the first high-fidelity world-process family for production hardening: snow/sediment, water, or erosion.
- P9: choose final release-quality validation fixtures that cover a complete SDF-first procedural scene, including material, texture, procgen, particles, physics, animation, simulation, runtime preview, and failed-product recovery.

## Negative Doctrine

Do not:

- make material graphs mesh-first;
- make the viewport depend directly on material graphs, particle graphs, physics internals, or animation graphs;
- execute editor-authored semantic graphs directly as runtime authority;
- let generated textures or field products replace source assets without catalog ratification;
- treat Texture3D as just an image import without volume/sampler/color-space metadata;
- let particles, physics, animation, or world processes mutate authored world state through side channels;
- hide procedural seeds, versions, or source maps;
- make one universal procedural graph for materials, SDF, particles, animation, and physics.

## Validation Strategy

Each track needs tests for:

- structural graph validation;
- semantic ratification;
- deterministic lowering;
- source lineage preservation;
- cache key stability;
- failed product preservation;
- hot reload safety;
- viewport expression product display;
- command/ratification boundaries;
- domain ownership boundaries.

Representative test names:

```text
material_graph_rejects_unknown_node_kind
triplanar_material_lowering_preserves_source_node_lineage
texture3d_descriptor_requires_volume_metadata
sdf_brush_commit_emits_world_operation_with_bounds
procgen_same_seed_forms_same_operation_window
particle_emitter_rejects_missing_field_product
physics_preview_cannot_mutate_authored_world_without_command
animation_curve_lowering_omits_editor_canvas_state
simulation_bake_preserves_preview_to_operation_lineage
```
