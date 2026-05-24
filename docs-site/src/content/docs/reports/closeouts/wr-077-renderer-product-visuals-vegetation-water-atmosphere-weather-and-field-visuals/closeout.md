---
title: WR-077 Renderer Product Visuals Vegetation Water Atmosphere Weather And Field Visuals Closeout
description: Closeout evidence for world visual producer renderer contracts, bounded working sets, residency, temporal inputs, and diagnostics.
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

# WR-077 Renderer Product Visuals Vegetation Water Atmosphere Weather And Field Visuals Closeout

## Result

`WR-077` is complete at `bounded_contract` quality. The renderer now exposes a
world visual producer feature contract that product domains can populate with
prepared vegetation, grass, water, wetness, atmosphere, weather, and field
summary batches without moving product semantics into the renderer.

The implementation adds:

- `engine/src/plugins/render/features/world/visuals/mod.rs`: typed prepared
  world visual payloads, bounded working-set counts, scale bands, temporal
  input declarations, residency requests, fallback/unsupported/over-budget
  batch states, and a registered feature collector.
- `engine/src/plugins/render/features/world/mod.rs`: world visual module
  boundary and public exports inside the existing world render subsystem.
- `engine/src/plugins/render/features/mod.rs`: `WORLD_VISUAL_RENDER_FEATURE_ID`,
  `WORLD_VISUAL_RENDER_FEATURE_LABEL`, and built-in feature ordering after
  particle/VFX and material handoff and before deformation.
- `engine/src/plugins/render/plugin.rs`: resource and collector registration
  for runtime preparation.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  public API notes for the world visual renderer producer contract.

## Boundary Evidence

- Product domains own vegetation simulation, water and wetness truth,
  atmosphere/weather/day-night meaning, field product lineage, freshness,
  fallback legality, authority, and rebuild policy.
- The renderer owns only the prepared feature descriptor, collector, fallback
  policy, bounded working-set inspection, temporal input declarations,
  residency request consumption, and runtime contribution ordering.
- Scale bands and working-set counts are explicit typed prepared data, not
  inferred from labels or draw counts.
- Residency requests use the existing product-render residency contract.
- Temporal input declarations are explicit prepared data for motion vectors,
  depth, exposure, reactive masks, weather masks, and history signatures.
- Missing prepared resources fail closed as typed collector diagnostics.
- Stale, fallback, unsupported, and over-budget world visual states remain
  visible in registered payload inspection fields.
- No ADR was required because the slice stays inside accepted renderer
  execution, field product, SDF residency, scale, and temporal contracts and
  does not move product truth, fallback authority, or a durable cross-domain
  world visual ABI into renderer code.

## Validation

Focused validation:

```text
cargo fmt --check
cargo test -p engine render_product_visual_world
cargo test -p engine render_sdf_residency
cargo test -p engine render_temporal_inputs
```

The focused world visual test run passed three tests:

- ready payloads report scale band, bounded working-set, residency, temporal,
  and visual-kind evidence;
- missing prepared resources emit typed diagnostics;
- stale fallback, over-budget, and unsupported batch states remain visible.

The supporting SDF residency and temporal input test filters also passed.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Animation/deformation and cross-family product visual evidence remain
  `PM-RENDER-PRODUCT-VISUALS-004` / `WR-078` scope.
- Runtime production examples, benchmark artifacts, hardware profiles, and
  final no-gap stack verification remain downstream product-visual and
  perfectionist-audit scope.
- This closeout does not claim product-domain simulation truth, water/weather
  authoring completeness, backend-specific visual quality, `runtime_proven`, or
  `perfectionist_verified`.
