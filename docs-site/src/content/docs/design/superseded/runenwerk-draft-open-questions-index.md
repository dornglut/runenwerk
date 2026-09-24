---
title: Draft Design Open Questions Index
description: Superseded consolidated open questions for the SDF-first draft design set.
status: superseded
owner: workspace
layer: cross-domain
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/sdf-first-field-world-platform-design.md
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../accepted/execution-fabric-and-product-jobs-design.md
  - ../accepted/sdf-first-production-capability-map.md
---

# Draft Design Open Questions Index

## Status

Superseded index.

The accepted SDF-first design set resolves or explicitly defers the questions
tracked here.

This document consolidates open questions from the current SDF-first field-product design draft set.

It is intended to help review, prioritize, and resolve architectural decisions before promoting the draft docs to active design docs.

## Source Documents

The current draft set is assumed to include:

1. `adaptive-field-product-system-design.md`
2. `sdf-product-renderer-architecture.md`
3. `sdf-world-production-slice-design.md`
4. `open-world-product-streaming-system-design.md`
5. `field-product-diagnostics-system-design.md`
6. `sdf-prefab-composition-system-design.md`
7. `sdf-character-animation-system-design.md`
8. `field-vegetation-system-design.md`
9. `day-night-atmosphere-system-design.md`
10. `water-wetness-field-system-design.md`
11. `sdf-physics-collision-system-design.md`
12. `field-influence-ai-system-design.md`
13. `procgen-field-product-system-design.md`
14. `field-vfx-particles-system-design.md`
15. `fluid-snow-erosion-world-processes-system-design.md`

---

# Highest-Priority Cross-Cutting Questions

These questions should be resolved before implementation starts, because multiple docs depend on them.

## Product ownership

1. Where does the canonical field-product descriptor vocabulary live?
2. Which product families belong in existing crates and which require future domain crates?
3. Which product states are universal versus family-specific?
4. Which products are authoritative, derived, visual-only, or strict?

## Renderer and GPU representation

1. What is the first accepted SDF GPU representation?
2. How are sampled SDF fields, analytic SDFs, SDF prefab graphs, and animated SDF characters represented for rendering?
3. How strict is the renderer allowed to be when products are stale, fallback, or missing?
4. Which diagnostics overlays are required for the first production slice?

## Scale, scope, and streaming

1. What are the canonical first scale bands?
2. What are the first product scopes: chunk, region, view, basin, sector, clipmap?
3. Which consumers may use ghost summaries?
4. What is the first memory/upload/rebuild budget policy?

## SDF-first production slice

1. What is the minimum complete SDF character rig?
2. What is the first prefab composition model?
3. What is the first vegetation representation?
4. What is the first water representation?
5. What are the required day/night hooks for render, material, vegetation, water, and enemies?

## Multiplayer and authority

1. Which product families become multiplayer-authoritative later?
2. Which states are replicated as operations/generations versus derived locally?
3. How are authority mismatches diagnosed?

---

# Open Questions by Document

## adaptive-field-product-system-design.md
1. Where should the canonical field-product descriptor vocabulary live: existing `domain/world_sdf`, a broader field-product domain, or several owning domain crates?
2. Which product families are part of the initial accepted taxonomy, and which remain future extension points?
3. Which scale bands are canonical engine-wide terms versus product-family-specific terms?
4. Which scope types are required at the parent level: chunk, region, sector, basin, view, clipmap window, and non-spatial scope?
5. Which product states are mandatory for all products: freshness, residency, retention, rebuild policy, lineage, diagnostics, or only a subset?
6. How strict should the parent architecture be about stale/fallback product use, and which decisions are delegated to product consumers?
7. Should product dependency tracking be implemented as a general graph product, or remain owned by product families and world operation systems?
8. What is the minimum accepted product query contract shape before renderer/physics/AI consumers are added?
9. Which diagnostics are parent-level required diagnostics versus family-specific diagnostics?
10. What is the promotion path from `design/draft/` to `design/active/` for the full doc set?

## sdf-product-renderer-architecture.md
1. What is the canonical GPU representation for sampled SDF chunks: dense bricks, sparse bricks, clipmap pages, or a hybrid?
2. How much SDF evaluation should be shader-side analytic versus uploaded sampled fields?
3. How are animated SDF character graphs encoded for GPU evaluation?
4. How many product generations can remain resident before eviction?
5. Should atmosphere be part of render runtime only, or have a domain-level time/day-night contract?
6. What is the first supported material-field channel set for the production slice?
7. How should renderer inspection expose product lineage without pulling domain internals into renderer code?
8. What fallback is allowed for missing SDF terrain in the main game surface?
9. Do field-product diagnostics render as overlays, panels, or both?
10. How strict should the renderer be when asked to render stale products?

## sdf-world-production-slice-design.md
1. What is the minimum accepted SDF character rig model?
2. How many SDF material channels are required for the first field?
3. What scale bands are mandatory for first production content?
4. What product diagnostics must appear in the first editor overlay?
5. How much water interaction is required before fluid simulation exists?
6. Which field products are authoritative in multiplayer later?
7. How is dark fantasy/fairytale art direction represented as data?
8. What is the minimal accepted combat/enemy interaction?
9. How much generated terrain variation is enough to prove the system?
10. What is the first accepted fallback for missing terrain products?

## open-world-product-streaming-system-design.md
1. What are the canonical chunk and region sizes for the first production slice?
2. How many scale bands are required before implementation begins?
3. What products are allowed to use ghost summaries?
4. Which products may be stale but still render?
5. What is the first memory budget target?
6. How should editor selection override streaming policy?
7. What is the first multiplayer-relevant product set?
8. How much prefetch is required for fast traversal?
9. How should caves map onto existing chunk/region scopes?
10. Should water basins be independent streaming scopes from regions?

## field-product-diagnostics-system-design.md
1. What diagnostic code namespace should be used for field products?
2. Which product diagnostics belong in `world_sdf` versus `world_ops` versus engine render?
3. How should editor panels subscribe to product diagnostics?
4. Should diagnostics be stored as products themselves?
5. Which diagnostics are persisted across sessions?
6. How should diagnostics integrate with existing ratification reports?
7. What is the minimum overlay set for the first production slice?
8. Should product lineage be visualized as a graph, table, or both?
9. How do diagnostics behave in multiplayer clients versus server authority?
10. How much backend renderer state may be exposed in inspection DTOs?

## sdf-prefab-composition-system-design.md
1. What is the minimum SDF composition graph needed for first production prefabs?
2. Should prefab composition reuse `domain/graph` directly or define a prefab-specific graph over it?
3. How are child prefabs referenced without creating cyclic dependencies?
4. What is the first accepted material channel set?
5. How are animated prefab bounds updated?
6. What prefab outputs are mandatory versus optional?
7. How are authored and procedural placements unified?
8. What is the first prefab ratification report shape?
9. How much prefab state is multiplayer-authoritative?
10. How do prefab product generations map to asset revisions?

## sdf-character-animation-system-design.md
1. What is the minimum SDF rig representation for first production characters?
2. Should rig controls live in a new animation domain or reuse scene transforms initially?
3. What animation graph substrate should be used?
4. How should SDF body graphs encode deformation masks?
5. What collision shape is required for first player movement?
6. How much root motion is supported initially?
7. How are footstep events generated from SDF pose data?
8. How should enemy animation update rates scale with distance?
9. How are character material masks represented?
10. What parts of animation state are multiplayer-authoritative?

## field-vegetation-system-design.md
1. What is the first vegetation rule descriptor shape?
2. How much SDF blade detail is required in near field?
3. Should grass be raymarched, instanced procedural SDF, or hybrid?
4. What is the first trample field storage format?
5. How long should trample recovery last?
6. How does water/wetness modify grass rules?
7. What is the first wind source before a full wind field exists?
8. Which vegetation products are gameplay-relevant?
9. What diagnostic overlays are mandatory?
10. How are glowing night plants represented?

## day-night-atmosphere-system-design.md
1. Does time-of-day start in a domain crate or engine runtime?
2. What is the first time scale?
3. Should moon phase be included immediately?
4. How are phase profiles authored?
5. Which material response channels are required first?
6. Which enemy schedule hooks are required first?
7. Does atmosphere produce a formal field product or render contribution only at first?
8. How are radiance products invalidated later?
9. How should editor time scrubbing affect runtime simulation?
10. What is authoritative in multiplayer?

## water-wetness-field-system-design.md
1. What is the first water authoring model: path, mask, basin, or generated depression?
2. Should river/lake scopes be part of `world_sdf`, a water domain, or product descriptors only?
3. What is the first water surface representation?
4. How is wetness stored: scalar field, material channel, or separate product?
5. What is required for first buoyancy/query behavior?
6. How do water products cross chunk boundaries?
7. What water states are multiplayer-authoritative?
8. How does water affect grass density and trample fields?
9. What is the first foam/mist representation?
10. How does full fluid simulation replace or enrich the first water products?

## sdf-physics-collision-system-design.md
1. What is the first strict collision product format: sampled SDF, analytic SDF, or hybrid?
2. What is the minimum character controller query set?
3. Should collision product descriptors live in `world_sdf` or a future physics domain?
4. How are collision products ratified?
5. What is the first broadphase structure?
6. How are active body regions pinned for streaming?
7. What collision fallback is allowed for generated terrain?
8. How are SDF prefab collision products authored?
9. What collision state is authoritative in multiplayer?
10. How do collision products interact with future fluid products?

## field-influence-ai-system-design.md
1. What is the first enemy perception field: threat, scent, sound, visibility, or a minimal combination?
2. Should influence field descriptors live in a new gameplay/AI domain?
3. What query contract should enemy AI use first?
4. How is field authority represented for multiplayer?
5. How do influence fields interact with navigation?
6. What is the first value representation: scalar grid, sparse field, graph, or hybrid?
7. How does day/night modify threat/perception?
8. How do vegetation and fog affect visibility?
9. What diagnostic overlays are mandatory?
10. How does influence product invalidation integrate with entity movement?

## procgen-field-product-system-design.md
1. Should procgen be a new domain crate or implemented first inside existing world/product domains?
2. What is the first generator descriptor format?
3. How are generator versions tracked?
4. What is the first deterministic terrain algorithm?
5. How are authored edits layered over generated terrain?
6. How are river/lake basins generated?
7. How are prefab placements ratified?
8. What procgen products are authoritative in multiplayer?
9. How much generation can happen at runtime versus offline/cache?
10. How should generated products be debugged in editor?

## field-vfx-particles-system-design.md
1. What is the first runtime particle representation?
2. Do VFX descriptors belong in asset, material, or a future VFX domain?
3. What VFX are mandatory for the first production slice?
4. What effects are gameplay-relevant versus visual-only?
5. How are particle buffers represented as products?
6. How does transparency sorting work with SDF rendering?
7. What is the first advection input before full wind exists?
8. How are VFX authored for prefabs?
9. What VFX state is multiplayer-authoritative?
10. What diagnostics overlays are mandatory?

## fluid-snow-erosion-world-processes-system-design.md
1. When should a separate fluid domain crate be introduced?
2. What process products belong in `world_sdf` versus future simulation domains?
3. What is the first solver-state descriptor format?
4. Which world processes can mutate terrain?
5. What mutation candidate format should processes use?
6. How are background/offline process jobs scheduled?
7. How much simulation state is persisted?
8. What is authoritative in multiplayer?
9. How are fluid/snow/erosion diagnostics visualized?
10. How do process products affect material_graph products?

---

# Resolution Order

Recommended order for resolving questions.

## Stage 1: Parent contracts

Resolve:

1. Field-product descriptor ownership.
2. Product family taxonomy.
3. Scope model.
4. Scale-band model.
5. Freshness/residency/retention/rebuild policy states.
6. Parent diagnostic requirements.

Relevant docs:

- `adaptive-field-product-system-design.md`
- `field-product-diagnostics-system-design.md`
- `open-world-product-streaming-system-design.md`

## Stage 2: Renderer representation

Resolve:

1. First SDF GPU representation.
2. Render product resolver strictness.
3. Product surface and target alias behavior.
4. SDF world, prefab, character, vegetation, water, atmosphere producer boundaries.

Relevant docs:

- `sdf-product-renderer-architecture.md`
- `sdf-world-production-slice-design.md`

## Stage 3: First production slice systems

Resolve:

1. SDF prefab graph substrate.
2. SDF character rig and body representation.
3. Vegetation representation.
4. Day/night ownership and product shape.
5. Water/wetness first representation.

Relevant docs:

- `sdf-prefab-composition-system-design.md`
- `sdf-character-animation-system-design.md`
- `field-vegetation-system-design.md`
- `day-night-atmosphere-system-design.md`
- `water-wetness-field-system-design.md`

## Stage 4: Gameplay/simulation authority

Resolve:

1. Strict collision product ownership.
2. Influence/AI field ownership.
3. Procgen domain ownership.
4. VFX visual-only versus gameplay-relevant split.
5. Fluid/snow/erosion mutation authority.
6. Multiplayer authoritative product classes.

Relevant docs:

- `sdf-physics-collision-system-design.md`
- `field-influence-ai-system-design.md`
- `procgen-field-product-system-design.md`
- `field-vfx-particles-system-design.md`
- `fluid-snow-erosion-world-processes-system-design.md`

---

# Recommended Tracking Shape

When a question is resolved, record:

| Field | Meaning |
|---|---|
| Question | The original open question. |
| Decision | The accepted answer. |
| Owner | Owning crate/domain/design doc. |
| Status | Open, proposed, accepted, superseded. |
| Follow-up docs | Docs requiring updates. |
| Follow-up implementation | Crates/modules affected. |
| Validation | Tests/docs validation needed. |

Example:

```text
Question:
  What is the first accepted SDF GPU representation?

Decision:
  Use sparse fixed-size SDF bricks with a GPU page table for first production terrain,
  while preserving analytic SDF evaluation for prefabs and characters.

Owner:
  sdf-product-renderer-architecture.md

Follow-up docs:
  adaptive-field-product-system-design.md
  open-world-product-streaming-system-design.md
  sdf-prefab-composition-system-design.md
```

---

# Promotion Gate

Before promoting the draft doc set to `design/active/`, resolve or explicitly defer:

1. Parent product ownership.
2. Initial scope and scale-band vocabulary.
3. Renderer SDF GPU representation.
4. Diagnostics code ownership.
5. First production slice acceptance criteria.
6. Strict collision versus visual product policy.
7. Day/night ownership.
8. Multiplayer authority placeholders.
9. Procgen ownership placeholder.
10. Any broken cross-links or frontmatter validation issues.
