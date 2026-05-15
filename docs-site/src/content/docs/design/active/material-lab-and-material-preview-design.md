---
title: Material Lab And Material Preview Design
description: Active design for roadmap-visible material graph authoring, ratification, preview products, diagnostics, and render handoff.
status: active
owner: domain/material_graph
layer: domain / app-runtime / engine-render
canonical: true
last_reviewed: 2026-05-15
related_designs:
  - ./editor-rendered-world-and-multi-entity-viewport-design.md
  - ./render-product-surface-foundation-bundle-design.md
related_roadmaps:
  - ../../apps/runenwerk-editor/roadmap.md
---

# Material Lab And Material Preview Design

## Status

Active design, roadmap-first. Implementation starts after rendered-world V1 and the field visualizer stabilize the viewport product contract.

## Decision

Material Lab uses source-backed material graph documents as truth. Canvas state is a projection, not the material source of truth.

Owning code paths:

- `domain/material_graph/src`
- `apps/runenwerk_editor/src/shell/providers/material_graph_canvas.rs`
- `apps/runenwerk_editor/src/shell/providers/material_inspector.rs`
- `apps/runenwerk_editor/src/shell/providers/material_preview.rs`
- `apps/runenwerk_editor/src/asset_pipeline`
- `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs`

## V1 Contract

V1 requires:

- `MaterialGraphDocument` source identity;
- ratification diagnostics before preview/render handoff;
- lowering into a renderer-consumable material product;
- material preview as a viewport/product target;
- failed preview preserving the prior valid artifact;
- source lineage in diagnostics and product descriptors.

## Non Goals

- No canvas-only material truth.
- No renderer-specific material graph ownership in domain code.
- No prefab material binding until prefab V2 has source/catalog identity.

## Roadmap Visibility

Material Lab must appear explicitly in the editor roadmap and workspace roadmap so it is not mistaken for a missing design. UI Designer remains the already-promoted self-authoring path; Material Lab is a separate material authoring track.

## Tests

Required coverage:

- source-backed material graph round trip;
- ratification blocks invalid graph products;
- failed preview keeps previous valid product;
- material preview target uses viewport product selection;
- provider surfaces fail closed until product handoff is available.
