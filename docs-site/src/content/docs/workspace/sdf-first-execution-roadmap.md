---
title: SDF-First Open-World Substrate Roadmap
description: Canonical cross-track sequencing roadmap for the editable, streamable, deterministic SDF-first open-world substrate.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
related_adrs:
  - ../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ../design/accepted/sdf-first-field-world-platform-design.md
  - ../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../design/accepted/sdf-first-production-capability-map.md
related_roadmaps:
  - ./roadmap-index.md
  - ./repo-execution-priority-checklist.md
  - ../apps/runenwerk-editor/roadmap.md
  - ../net/ecs-runtime-prioritized-roadmap.md
  - ../engine/plugins/render/docs/roadmap.md
---

# SDF-First Open-World Substrate Roadmap

## Purpose

This is the canonical cross-track sequencing roadmap for turning the accepted
SDF-first field-product architecture into an editable, streamable,
deterministic open-world substrate.

The target result is not finished open-world gameplay in one jump. The target
is the substrate that makes open-world gameplay possible without private
execution paths or renderer-owned world truth.

The immediate priority is now M6.2 procgen through the first accepted procgen
product track. That work must build on the completed execution substrate so
procgen, gameplay graph, particles, physics, animation, world processes,
renderer residency, and strict runtime consumers use formed products without
inventing private execution paths.

Owning domain and app roadmaps still own detailed implementation steps. This
page owns cross-track ordering when those roadmaps overlap.

## End State

The long-term open-world substrate is:

```text
authored edits
  + deterministic procgen
  + SDF field products
  + query/render/physics consumers
  -> editable, streamable, diagnosable open world
```

Required end-state properties:

- authored edits remain governed operations or authored layers;
- deterministic procgen produces scoped products, not hidden runtime truth;
- formed SDF and field products carry identity, lineage, freshness, authority,
  residency, query policy, and diagnostics;
- product jobs publish only through deterministic barriers;
- strict consumers reject stale, fallback, ghost, missing, visual-only, and
  diagnostic-only products;
- render selections and GPU residency are derived from product truth;
- renderer, UI, and debug surfaces remain product consumers, not world truth.

## Current Focus

The current work is Phase 6 of the execution substrate: the first procgen
product track. Phases 1 through 5 are complete: serial product publication
outcomes, deterministic publication barriers, runtime query snapshots, strict
runtime consumption decisions, app-owned editor proof surfaces, render product
selection producers, derived renderer GPU residency, and the accepted procgen
domain readiness contract now exist.

Current gaps:

- `domain/procgen` code, ratification, lowering, product-job integration,
  preview, bake, and editor providers remain unimplemented until Phase 6.

## Phased Roadmap

Use this order for current implementation planning. Each phase must close its
acceptance gate before product-domain implementation advances.

### Phase 0 - Batch 1 Contract Alignment

Status: complete.

Acceptance gate:

- shared product vocabulary exists in `domain/product`;
- current SDF, material, texture, asset/import, editor, scheduler, ECS, engine,
  and renderer-prep surfaces align with the vocabulary;
- serial behavior remains equivalent to the pre-batch runtime;
- full gate validation passes.

Out of scope:

- product-job dispatch;
- runtime query snapshot production;
- GPU residency;
- procgen implementation.

### Phase 1 - Serial Product Jobs And Publication Barriers

Status: complete as of 2026-05-12.

Intent: route current product formation through job descriptors and publish
formed outputs only at deterministic barriers while preserving serial runtime
behavior.

Closeout evidence:

- [reports/closeouts/sdf-first-execution-phase-1/closeout.md](../reports/closeouts/sdf-first-execution-phase-1/closeout.md)

Acceptance gate:

- product jobs have outcome/publication metadata with diagnostics;
- the runtime stages product outcomes and publishes them only at
  `ProductPublication` barriers;
- publication order is deterministic and inspectable;
- failed-preserved products require diagnostics and respect failure policy;
- current editor field-product jobs still run serially and remain app-owned
  until promoted deliberately.

Out of scope:

- worker pools or parallel execution;
- global product registry authority;
- procgen products;
- GPU upload or residency.

### Phase 2 - Query Snapshots And Strict Consumer Policy

Status: complete as of 2026-05-13.

Intent: make deferred/runtime queries explicit products with generation,
freshness, invalidation, consumer class, and diagnostics.

Closeout evidence:

- [reports/closeouts/sdf-first-execution-phase-2/closeout.md](../reports/closeouts/sdf-first-execution-phase-2/closeout.md)

Acceptance gate:

- runtime query snapshots carry source generation, response generation, scope,
  freshness, consumer class, invalidation policy, and diagnostics;
- strict consumers reject stale, fallback, ghost, missing, visual-only, and
  diagnostic-only products outside ratifiers, not only in unit tests;
- query-snapshot invalidation is deterministic and visible to diagnostics;
- editor and renderer consumers can inspect why a snapshot was accepted,
  rejected, or preserved.

Out of scope:

- broad AI, physics, or procgen behavior using snapshots;
- parallel query execution;
- fallback promotion into authoritative truth.

### Phase 3 - Render Product Selection Producers

Status: complete as of 2026-05-13.

Intent: populate backend-neutral `RenderProductSelection` from formed products,
generations, views, and diagnostics.

Closeout evidence:

- [reports/closeouts/sdf-first-execution-phase-3/closeout.md](../reports/closeouts/sdf-first-execution-phase-3/closeout.md)

Acceptance gate:

- render prepare produces selections from product descriptors and generations;
- selected products, stale/fallback/ghost markers, required targets, residency
  requests, and diagnostics are inspectable;
- render submit consumes prepared selections and does not perform live ECS
  extraction to discover product truth;
- renderer fallback behavior cannot bypass strict product policy.

Out of scope:

- SDF terrain renderer rebuild;
- GPU SDF residency;
- material or texture upload;
- renderer-owned product authority.

### Phase 4 - Derived GPU Residency

Status: complete as of 2026-05-13.

Intent: add renderer-owned GPU cache and residency management derived from
product selections.

Closeout evidence:

- [reports/closeouts/sdf-first-execution-phase-4/closeout.md](../reports/closeouts/sdf-first-execution-phase-4/closeout.md)

Acceptance gate:

- GPU resources are tied to product identity and generation;
- residency requests can allocate, preserve, evict, or invalidate derived caches
  with diagnostics;
- stale, fallback, ghost, missing, and failed-preserved states remain visible;
- backend handles stay inside renderer/backend code and never enter domain,
  editor, UI, or product descriptors.

Out of scope:

- renderer-owned world truth;
- procgen algorithms;
- full product-family render feature coverage beyond proving the residency
  contract.

### Phase 5 - Procgen Readiness Gate

Status: complete as of 2026-05-13.

Intent: accept the procgen ownership and product-domain contract before any
procgen code starts.

Closeout evidence:

- [reports/closeouts/sdf-first-execution-phase-5/closeout.md](../reports/closeouts/sdf-first-execution-phase-5/closeout.md)

Acceptance gate:

- `docs-site/src/content/docs/domain/procgen/README.md` is accepted;
- generator descriptor format, seed/scope/version policy, cache lineage,
  authored edit layering, runtime/offline generation policy, and multiplayer
  authority policy are decided;
- procgen products are defined as product-job outputs with publication,
  query-snapshot, strict-consumer, render-selection, and diagnostics paths;
- procgen editor surfaces remain design/docs-only until the contract is
  accepted.

Out of scope:

- `domain/procgen` code;
- generator algorithms;
- procgen preview or bake execution.

### Phase 6 - First Procgen Product Track

Intent: implement the first visible open-world producer on top of the completed
execution substrate. This is a procgen-owned product track, not a generic
content-platform launch.

Acceptance gate:

- `domain/procgen` owns deterministic generator documents, seed/scope/version
  contracts, planning lifecycle metadata, ratification, and lowering;
- first-slice prototypes, candidates, reservations, instance plans,
  realizations, and explanations are scoped to bounded region terrain/material
  generation;
- bounded generator jobs form products through product jobs and publication
  barriers;
- same seed, scope, version, inputs, and upstream generations produce identical
  operation windows and diagnostics;
- bake-to-`world_ops` and bake-to-field-product flows preserve authored edits
  and produce changed-region diagnostics.

Out of scope:

- full biome editor;
- full caves;
- gameplay spawn systems;
- procedural quests;
- particles, physics, animation, and world-process domains.

## Procgen Code Gate

M6.2 procgen code may start only from the accepted
`docs-site/src/content/docs/domain/procgen/README.md` contract now that Phases
1 through 5 are complete.

Procgen code must not bypass product jobs, query snapshots, publication
barriers, strict consumer policy, render product selection, or derived GPU
residency.

Gameplay graph, particles, physics, animation, and world-process domains follow
only after their owning contracts can consume the same substrate.

## Roadmap Ownership

- `workspace/sdf-first-execution-roadmap.md` owns cross-track order.
- `apps/runenwerk-editor/roadmap.md` owns editor milestone detail.
- `domain/scheduler/*` owns scheduler contract detail.
- `domain/ecs/*` owns ECS state, query, command, and runtime-bridge detail.
- `net/ecs-runtime-prioritized-roadmap.md` remains a net/runtime convergence
  tracker and feeds this roadmap where ECS runtime work is relevant.
- `engine/plugins/render/docs/roadmap.md` owns render implementation detail, but
  SDF renderer and GPU residency work must follow accepted product selection
  and residency contracts.

## Validation Expectations

Roadmap updates should verify:

- docs validation passes;
- no priority list says M6.2 procgen code starts outside the accepted procgen
  contract or completed execution substrate;
- completed contracts are marked as implemented and unfinished gates remain
  explicit;
- renderer, UI, and debug products remain derived state;
- strict consumers cannot be satisfied by visual-only product paths;
- `domain/product` is not described as a global product registry or authority
  owner.

## Finished Baselines

- editor M4 asset and field-product foundations exist;
- M5 runtime preview and reload boundaries exist for the current product
  families;
- M6.0 shared workspace substrate, M6.1 material/texture contracts, and P1 SDF
  modeling core exist;
- accepted SDF-first field-product, execution-fabric, and renderer-residency
  designs define the target architecture;
- Batch 1 contract alignment is complete: `domain/product`, product-core
  adapters, serial scheduler waves/barriers, ECS explicit deferred barriers,
  and backend-neutral render product-selection metadata exist;
- Phase 1 serial product jobs and publication barriers are complete:
  publication outcomes are ratified in `domain/product`, every serial scheduler
  wave emits `ApplyDeferredCommands` then `ProductPublication`, ECS exposes
  product-agnostic barrier hooks, engine runtime owns publication staging, and
  editor field-product artifacts publish through app-owned barrier handling;
- Phase 2 query snapshots and strict consumer policy are complete:
  `domain/product` owns strict consumption decisions and query-snapshot
  publication reports, every serial scheduler wave emits
  `ApplyDeferredCommands`, `ProductPublication`, then
  `QuerySnapshotPublication`, ECS exposes product-agnostic source generation
  helpers, engine runtime owns query snapshot staging/publication/invalidation,
  renderer inspection reads snapshot decisions without backend handles, and the
  editor viewport publishes observation snapshots through app-owned barrier
  handling;
- Phase 3 render product selection producers are complete: `domain/product`
  owns typed render-selection state and ratification, engine render preparation
  stores producer-scoped `RenderProductSelection` contributions, prepared-frame
  inspection exposes selected products, target descriptors, residency requests,
  and diagnostics without backend handles, and the editor viewport produces
  selections from accepted query snapshots during `RenderPrepare`;
- Phase 4 derived GPU residency is complete: `engine/plugins/render` owns
  logical renderer GPU cache handles, derives residency from prepared render
  product selections, allocates, preserves, invalidates, evicts, rejects, and
  journals cache state deterministically, exposes read-only residency
  inspection without backend handles, and the editor viewport records concise
  residency summaries while world render-cache invalidation removes stale typed
  cache entries;
- Phase 5 procgen readiness is complete: the accepted
  `domain/procgen/README.md` contract defines future procgen ownership,
  graph-backed generator documents, seed/scope/version policy, cache lineage,
  authored overlay preservation, runtime/offline policy, server-validated
  multiplayer authority, and product output paths through the completed
  execution substrate;
- `world_sdf`, material, texture, asset/import, and editor product surfaces
  align with product-core metadata while preserving their owning-domain APIs;
- render prepare carries backend-neutral `RenderProductSelection` metadata as
  prepared frame state without GPU handles or renderer-owned world truth.
