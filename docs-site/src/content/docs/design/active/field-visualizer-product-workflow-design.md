---
title: Field Visualizer Product Workflow Design
description: Active design for inspecting scalar, vector, atlas, volume, brickmap, and history field products through viewport products.
status: active
owner: apps/runenwerk_editor
layer: app-runtime / editor-ui
canonical: true
last_reviewed: 2026-05-15
related_designs:
  - ./editor-rendered-world-and-multi-entity-viewport-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ../accepted/sdf-first-field-world-platform-design.md
related_roadmaps:
  - ../../apps/runenwerk-editor/roadmap.md
---

# Field Visualizer Product Workflow Design

## Status

Active design. Implementation is staged after the rendered-world V1 scene packet because field visualization must reuse viewport product routing instead of creating another preview path.

## Decision

The field visualizer is a viewport product workflow, not a separate viewer.

Owning code paths:

- `apps/runenwerk_editor/src/runtime/viewport/product_registry.rs`
- `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs`
- `apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs`
- `apps/runenwerk_editor/src/shell/providers/field_product_viewer.rs`
- `domain/editor/editor_shell/src/composition/build_viewport_panel.rs::viewport_options_popup`

## Product Kinds

The visualizer must route through existing product descriptors for:

- scalar fields;
- vector fields;
- atlas layers;
- volume slices;
- brickmap debug views;
- temporal/history color products.

Each product exposes availability, producer health, dimensions, format, freshness, and source reality through `editor_viewport::ExpressionProductDescriptor`.

## Controls

V1 controls belong in viewport/session state:

- product kind;
- channel or component selection;
- slice index for volume products;
- color ramp;
- debug mode;
- unavailable-product diagnostics.

Controls must not duplicate product target identity. They select or parameterize the product that the viewport already presents.

## Non Goals

- No parallel texture viewer path for fields.
- No new renderer-owned field source.
- No product publication outside the product-job and query-snapshot boundaries.

## Tests

Required coverage:

- product selection maps to the correct `ViewportSurfacePresentationSlot`;
- unavailable products stay visible with diagnostics;
- visualizer session state persists per tool surface;
- field products use the same dynamic target registry as scene color, picking ids, and overlay.
