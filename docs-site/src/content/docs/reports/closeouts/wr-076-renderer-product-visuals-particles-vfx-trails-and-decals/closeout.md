---
title: WR-076 Renderer Product Visuals Particles VFX Trails And Decals Closeout
description: Closeout evidence for particle, VFX, trail, decal, sorting, transparency, residency, temporal, and fallback renderer producer contracts.
status: completed
owner: engine
layer: engine-runtime / product-render-integration
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/renderer-product-visual-producers-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-076 Renderer Product Visuals Particles VFX Trails And Decals Closeout

## Result

`WR-076` is complete at `bounded_contract` quality. The renderer now exposes a
product-visual particle/VFX feature contract that product domains can populate
with prepared particle, VFX, trail, and decal batches without moving product
semantics into the renderer.

The implementation adds:

- `engine/src/plugins/render/features/particle_vfx/mod.rs`: typed prepared
  particle/VFX payloads, sorting/transparency intent, temporal input
  declarations, residency requests, fallback/unsupported/over-budget batch
  states, and a registered feature collector.
- `engine/src/plugins/render/features/mod.rs`: `PARTICLE_VFX_RENDER_FEATURE_ID`
  and built-in feature ordering after material handoff and before deformation.
- `engine/src/plugins/render/plugin.rs`: resource and collector registration for
  runtime preparation.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  public API notes for the new particle/VFX renderer producer contract.

## Boundary Evidence

- Product domains own emitter semantics, VFX graph meaning, trail/decal source
  truth, freshness, fallback legality, and rebuild policy.
- The renderer owns only the prepared feature descriptor, collector, fallback
  policy, inspection fields, and runtime contribution ordering.
- Sorting and transparency intent are explicit typed prepared data, not inferred
  from insertion order.
- Temporal input declarations are explicit prepared data for motion vectors,
  reactive masks, depth, exposure, and history signatures.
- Residency requests use the existing product-render residency contract.
- Missing prepared resources fail closed as typed collector diagnostics.
- Stale, fallback, unsupported, and over-budget batch states remain visible in
  registered payload inspection fields.

## Validation

Focused validation:

```text
cargo fmt
cargo test -p engine render_product_visual_particles
```

The focused test run passed three particle/VFX tests:

- ready payloads report batch, instance, residency, temporal, sorting, and
  transparency evidence;
- missing prepared resources emit typed diagnostics;
- stale fallback, over-budget, and unsupported batch states remain visible.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Vegetation, water, atmosphere, weather, and field visuals remain
  `PM-RENDER-PRODUCT-VISUALS-003` / `WR-077` scope.
- Animation/deformation and cross-family product visual evidence remain
  `PM-RENDER-PRODUCT-VISUALS-004` / `WR-078` scope.
- Runtime production examples, benchmark artifacts, hardware profiles, and
  final no-gap stack verification remain downstream product-visual and
  perfectionist-audit scope.

This closeout does not claim `runtime_proven` or `perfectionist_verified`.
