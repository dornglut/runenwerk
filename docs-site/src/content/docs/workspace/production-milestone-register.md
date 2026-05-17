---
title: Production Milestone Register
description: Generated production milestone register with gates, WR links, and acceptance criteria.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-17
related:
  - ./production-track-planning-model.md
  - ./production-track-index.md
  - ./production-tracks.yaml
  - ./roadmap-items.yaml
---

# Production Milestone Register

This register is generated from [production-tracks.yaml](./production-tracks.yaml).
Do not edit it directly.

| Track | Milestone | Kind | State | Dependencies | WR links | Design gates | Evidence gates | Acceptance criteria |
|---|---|---|---|---|---|---|---|---|
| PT-SDF-OW | PM-SDF-OW-001 Production product spine | implementation | completed | N/A | WR-019, WR-026, WR-021 | design:docs-site/src/content/docs/design/accepted/sdf-first-production-capability-map.md requires accepted<br>design:docs-site/src/content/docs/design/active/field-visualizer-product-workflow-design.md requires active<br>design:docs-site/src/content/docs/design/active/editor-asset-pipeline-and-content-workflow-design.md requires active<br>design:docs-site/src/content/docs/design/active/material-lab-and-material-preview-design.md requires active | docs-site/src/content/docs/reports/closeouts/wr-019-field-visualizer-product-workflow/closeout.md requires completed<br>docs-site/src/content/docs/reports/closeouts/wr-026-source-backed-asset-editor-adapters/closeout.md requires completed<br>docs-site/src/content/docs/reports/closeouts/wr-021-material-lab-and-material-preview-products/closeout.md requires completed | Product debug, asset catalog adapters, material preview products, and renderer handoff are validated through their WR closeouts.<br>No product workflow adds a parallel viewer, asset truth store, or renderer-owned semantic source. |
| PT-SDF-OW | PM-SDF-OW-002 Open world substrate design | design | active | PM-SDF-OW-001 | WR-001, WR-014, WR-015 | design:docs-site/src/content/docs/design/accepted/sdf-first-field-world-platform-design.md requires accepted<br>design:docs-site/src/content/docs/design/deferred/sdf-world-production-slice-design.md requires accepted | N/A | Terrain, material, collision/query, render, diagnostics, and streaming ownership are accepted before execution.<br>The design preserves future caves, interiors, multiplayer authority, and world-process simulation. |
| PT-SDF-OW | PM-SDF-OW-003 Playable SDF character | design | designing | PM-SDF-OW-002 | WR-014, WR-015, WR-022 | design:docs-site/src/content/docs/design/active/sdf-procedural-animation-and-animated-models-design.md requires accepted<br>design:docs-site/src/content/docs/design/deferred/sdf-physics-collision-system-design.md requires accepted | N/A | The player can walk, idle, and run using SDF-aware motion and strict query products.<br>Footstep, trample, wetness, and diagnostics hooks are explicit products or events, not renderer side effects. |
| PT-SDF-OW | PM-SDF-OW-004 Atmosphere and material response | design | designing | PM-SDF-OW-002 | WR-014, WR-015 | design:docs-site/src/content/docs/design/deferred/day-night-atmosphere-system-design.md requires accepted | N/A | Time, celestial, atmosphere, material response, and diagnostics products are accepted.<br>Future gameplay schedules and multiplayer authority remain possible. |
| PT-SDF-OW | PM-SDF-OW-005 Vegetation field interaction | design | designing | PM-SDF-OW-003, PM-SDF-OW-004 | WR-014, WR-015 | design:docs-site/src/content/docs/design/deferred/field-vegetation-system-design.md requires accepted | N/A | Grass responds to player motion and wind through products with visible diagnostics.<br>Near, mid, and far LOD behavior remains stable and inspectable. |
| PT-SDF-OW | PM-SDF-OW-006 SDF prefab production set | design | designing | PM-SDF-OW-002 | WR-022 | design:docs-site/src/content/docs/design/active/sdf-prefab-composition-system-design.md requires accepted | N/A | Trees, rocks, and ruins have source identity, placement rules, render products, collision/query products, and diagnostics.<br>Procedural placement and authored placement can share the same product contract. |
| PT-SDF-OW | PM-SDF-OW-007 Water and wetness fields | design | designing | PM-SDF-OW-004, PM-SDF-OW-006 | WR-014, WR-015 | design:docs-site/src/content/docs/design/deferred/water-wetness-field-system-design.md requires accepted | N/A | Rivers, lakes, wet shorelines, simple query products, and diagnostics are accepted.<br>Future fluids and erosion remain compatible with the field-product model. |
| PT-SDF-OW | PM-SDF-OW-008 Enemy and influence AI proof | design | designing | PM-SDF-OW-003, PM-SDF-OW-006 | WR-011, WR-014, WR-015, WR-022 | design:docs-site/src/content/docs/design/deferred/field-influence-ai-system-design.md requires accepted<br>design:docs-site/src/content/docs/design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md requires accepted | N/A | Enemies emit and consume explicit influence/perception products.<br>Basic enemy behavior does not create hidden renderer or gameplay graph authority. |
| PT-SDF-OW | PM-SDF-OW-009 Production hardening and playable evidence | hardening | designing | PM-SDF-OW-005, PM-SDF-OW-007, PM-SDF-OW-008 | WR-018, WR-019, WR-021, WR-022 | design:docs-site/src/content/docs/design/accepted/field-product-contracts-diagnostics-and-residency-design.md requires accepted<br>design:docs-site/src/content/docs/design/accepted/execution-fabric-and-product-jobs-design.md requires accepted | N/A | A player can move through an SDF field, observe day/night, affect vegetation, approach prefabs, see water/wetness, encounter enemies, and inspect diagnostics.<br>LOD, freshness, fallback, residency, and product-lineage diagnostics are stable enough for continued production. |
