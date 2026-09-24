---
title: Material Graph Domain
description: Accepted ownership boundary for authored material graphs, ratification, lowering, and formed material products.
status: accepted
owner: material-graph
layer: domain
canonical: true
last_reviewed: 2026-05-09
related_docs:
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../graph/README.md
  - ../texture/README.md
  - ../world-ops/README.md
  - ../world-sdf/README.md
---

# Material Graph Domain

`domain/material_graph` owns authored material graph semantics. The editor may present and route graph surfaces, but graph canvas state is never the material source of truth.

## Pipeline

Every material graph slice follows this invariant:

```text
authored material graph document
  -> graph structural validation
  -> material ratification
  -> normalized material IR
  -> formed material product
  -> asset artifact/revision
  -> runtime/preview product
  -> editor diagnostics/surfaces
```

Runtime and preview systems consume formed material products. They must not consume editor session state, canvas node positions, draft UI panels, or unratified graph documents.

## Ownership

This domain owns:

- `MaterialGraphDocument` authored document contracts using `domain/graph::GraphDefinition` for neutral graph structure;
- material node catalog boundaries for PBR, SDF/field inputs, procedural math/noise, texture sampling, triplanar coordinates, and material-channel outputs;
- material issue codes, ratifiers, semantic diagnostics, and source maps;
- normalized material IR used by deterministic lowering;
- formed material products, parameter schema, specialization fragments, source lineage, cache keys, and failed-product preservation semantics;
- lowering contracts for render/material expression products and field material-channel products.

This domain does not own:

- editor graph canvas layout, selection, pan, zoom, or interaction state;
- renderer shader compilation, GPU resource binding, or backend optimization;
- Texture2D, Texture3D, sampler, color-space, compression, or upload policy, which belong to `domain/texture` and engine/app adapters;
- world mutation execution, which must go through `domain/world_ops`;
- runtime hot-reload policy before a formed product and adapter contract exists.

## Accepted First Slice

The first implementation slice may add:

- PBR scalar/vector parameters for base color, roughness, metallic, normal strength, emissive, opacity/mask, and material id/channel bindings;
- SDF/field input nodes for position, normal/gradient, distance, material channel, density, support, and wetness;
- deterministic procedural nodes for noise, fbm, ramp, remap, clamp, mix, and mask;
- triplanar coordinate nodes for world, object, local, and field-product coordinate spaces;
- Texture2D and Texture3D sample nodes that reference catalog-backed texture products;
- source-mapped diagnostics for missing inputs, unsupported nodes, cycles, illegal writes, invalid PBR ranges, missing texture products, and unsupported runtime targets.

The accepted initial lowering target is a domain-owned formed material descriptor with parameter schema, source map, cache key, and specialization fragment. Engine/render adapters may translate that descriptor into `PreparedMaterialFeatureContribution`; `domain/material_graph` must not depend on engine render crates.

The editor owns descriptor-first provider surfaces for material graph canvas, material inspector, and material preview. These surfaces may display material graph catalog entries, source diagnostics, formed product descriptors, cache keys, and reload status, but they do not make graph canvas state authoritative and do not execute renderer-private shader behavior.

## Gates

Do not implement runtime material execution until:

- `domain/material_graph` exists and ratifies authored material graph candidates;
- `domain/texture` exists for texture product descriptors used by texture sample nodes;
- material graph lowering produces deterministic formed products with source lineage and cache keys;
- asset artifact/revision and reload classification can preserve the prior valid material product on failure.

Do not add gameplay scripting, arbitrary graph execution, or renderer-private shader authoring in this domain.
