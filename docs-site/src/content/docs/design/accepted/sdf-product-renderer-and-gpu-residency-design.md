---
title: SDF Product Renderer and GPU Residency Design
description: Accepted renderer architecture for consuming SDF-first field products without owning world truth.
status: accepted
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-12
related_adrs:
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ./sdf-first-field-world-platform-design.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./render-fragment-data-driven-maturity-design.md
supersedes:
  - ../superseded/sdf-product-renderer-architecture.md
---

# SDF Product Renderer and GPU Residency Design

## Status

Accepted renderer architecture.

## Purpose

The renderer is an execution and presentation layer over prepared products. It
must render SDF-first worlds, product surfaces, diagnostics, and future product
families without owning world, simulation, editor, or gameplay truth.

## Renderer Position

Renderer input flow:

```text
domain products and runtime product selection
  -> prepared render product selection
  -> GPU residency requests
  -> derived renderer caches
  -> render flow execution
  -> product surfaces, main surfaces, viewports, diagnostics
```

The renderer consumes products. It does not own authored world state, product
authority, prefab truth, animation truth, simulation state, gameplay authority,
or editor workflow policy.

## GPU Representation

Accepted first long-term GPU model:

- sparse sampled SDF bricks, page tables, and clipmap windows for terrain,
  caves, and large field products;
- analytic or graph SDF evaluation for prefabs, characters, and compact field
  compositions where it is better than uploaded samples;
- material and substance channels stored alongside or referenced by SDF field
  products;
- product generation metadata attached to uploaded resources;
- derived mesh products allowed only as preview, debug, fallback, export, import
  bridge, or explicitly accepted specialized products.

The model is hybrid by design. The invariant is not one GPU storage format; the
invariant is product lineage and derived-cache status.

## Product Selection

`RenderProductSelection` is the target prepared input shape. It should include:

- view or camera identity;
- selected product identities;
- selected scale bands;
- product generations;
- fallback, ghost, or stale markers;
- required target descriptors;
- GPU residency requests;
- diagnostics toggles and overlay selections.

The renderer may reject, fall back, or display diagnostics according to the
product query policy. It must never silently promote missing, stale, ghost, or
visual-only data into strict authority.

## Product Surfaces

Render product surfaces remain the stable engine capability for scene color,
picking ids, overlays, previews, field slices, diagnostic views, history, and
offscreen products.

The accepted model builds on existing render product surface architecture:

- one compiled render flow can run for many prepared render views or product
  invocations;
- target identity is explicit through flow-owned targets, dynamic targets, or
  surface targets;
- target aliases resolve per prepared invocation;
- UI samples backend-neutral binding sources, not backend handles;
- render submission consumes prepared data and performs no live ECS extraction.

## GPU Residency

GPU residency is derived cache management.

Renderer-owned resources may include:

- SDF brick atlases;
- SDF page tables;
- material and substance atlases;
- SDF instance buffers;
- animated pose buffers;
- vegetation, water, atmosphere, VFX, and diagnostic buffers;
- history targets and temporal caches.

Residency rules:

- every uploaded resource is tied to product generation or source generation;
- descriptor changes invalidate or reallocate only affected resources;
- stale, fallback, missing, and ghost states remain visible to diagnostics;
- prior valid resources may be preserved only when product policy allows
  failed-preserved fallback;
- backend handles do not cross into domain, UI, or app product descriptions.

## Product Producers

Renderer-facing producers prepare products; they do not own product truth.

Accepted producer families:

- SDF world and cave producer;
- SDF prefab producer;
- SDF character producer;
- vegetation producer;
- water and wetness producer;
- atmosphere/day-night producer;
- VFX producer;
- diagnostics producer.

Each producer consumes product selections and emits prepared render
contributions, GPU residency requests, and diagnostics. Product family truth
stays with the owning domain.

## Day/Night And Atmosphere

Time and celestial descriptions belong in domain product contracts. Engine time
and runtime own ticking and execution. Renderer consumes prepared atmosphere
products such as sun, moon, sky, fog, exposure, ambient response, water tint,
and vegetation glow parameters.

Day/night must not be a renderer-only color lerp. It is product state consumed
by rendering, materials, vegetation, water, AI, and future radiance products.

## Diagnostics

Renderer diagnostics must expose:

- selected render products and generations;
- GPU residency and upload state;
- target alias resolution;
- dynamic product target state;
- stale, fallback, ghost, and missing products;
- non-sampleable or invalid target bindings;
- history target validity;
- backend allocation failures.

Diagnostics expose read-only DTOs. They must not expose mutable `wgpu` handles
or backend-owned resources to domain/app/UI layers.

## Validation Expectations

Future renderer work should prove:

- prepared render products can drive SDF terrain without mesh assumptions;
- product surfaces render through prepared views and target aliases;
- GPU resources track product generation;
- stale/fallback/ghost states are inspectable;
- strict product policy cannot be bypassed by renderer fallbacks;
- UI sampling remains backend-neutral.
