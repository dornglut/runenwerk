---
title: WR-064 Sparse SDF Brick Page And Clipmap Residency Implementation Contract
description: Design-first contract for renderer-owned sparse SDF brick, page, and clipmap residency evidence.
status: active
owner: engine
layer: engine-runtime / renderer sdf residency
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-064 Sparse SDF Brick Page And Clipmap Residency Implementation Contract

## Goal

Prepare the bounded implementation slice for `PM-RENDER-SDF-002` and
`WR-064`. This row adds renderer-owned derived residency records for sparse SDF
chunk products: brick atlas ranges, page-table entries, clipmap windows,
generation keys, cache pressure, invalidation state, and inspection DTOs.

This is a design-first contract. It clears the deferred intake questions and
prepares WR-064 for roadmap application and promotion. It does not authorize
product code changes until the stack coordinator selects WR-064 for
implementation after roadmap gates are satisfied.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md`:
  renderer SDF execution owns derived GPU residency, raymarch execution,
  acceleration data, timing, and diagnostics, while SDF products own source
  truth, payload lineage, freshness, query policy, and strict consumer
  semantics.
- `docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md`:
  renderer GPU residency is a derived cache over prepared SDF products, not
  product truth or fallback authority.
- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  residency evidence must distinguish addressable, selected, resident,
  visible, submitted, and measured renderer work with explicit budget pressure.
- `docs-site/src/content/docs/reports/closeouts/pm-render-sdf-001-sparse-sdf-rendering-doctrine/closeout.md`:
  the sparse SDF rendering doctrine is accepted and gates this residency slice.
- `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`:
  WR-061 completed finite renderer working-set and residency-budget evidence.
- `domain/world_sdf/src/storage.rs`:
  `SdfChunkPayload`, `SdfPageCoord3`, `SdfPageRecord`, and `SdfBrickRecord`
  define the domain-owned sparse SDF payload shape consumed by renderer-derived
  residency.
- `domain/world_sdf/src/product.rs`:
  `FieldProductDescriptor::product_core` translates world SDF products into
  generic product contracts with lineage, scale band, freshness, residency, and
  query policy.
- `engine/src/plugins/render/residency/resource.rs`:
  `RenderGpuResidencyResource::derive_from_selections` is the existing
  renderer-owned finite residency and budget evidence boundary.
- `engine/src/plugins/render/features/world/runtime_cache.rs`:
  `WorldRuntimeCacheResource` is the current world-render cache and invalidation
  owner inside the renderer.

## Readiness

`task production:plan -- --milestone PM-RENDER-SDF-002 --roadmap WR-064`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-064-sparse-sdf-brick-page-and-clipmap-residency/plan.md`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "WR-064 Sparse SDF Brick Page And Clipmap Residency" --scope "PM-RENDER-SDF-002 engine/src/plugins/render sparse SDF derived GPU residency brick atlas page table clipmap inspection"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` for derived renderer
  cache state, GPU residency evidence, render inspection DTOs, and diagnostics.
- Source truth owner: `domain/world_sdf` and upstream product publication for
  chunk payloads, page tables, brick payloads, chunk revisions, payload checksums,
  freshness, query policy, fallback legality, and strict consumer semantics.
- Translation boundary: generic product selections and residency requests enter
  the renderer through `domain/product` contracts; SDF payload data may be read
  to derive renderer cache descriptors but must not become renderer-owned
  product truth.
- ADR requirement: no ADR is required if WR-064 only adds renderer-owned
  derived residency DTOs, cache records, inspections, and tests. Stop for ADR
  or accepted design update before implementation if the slice introduces a
  persisted cross-domain ABI, changes dependency direction, moves SDF fallback
  or query authority into renderer code, or rewrites domain SDF payload
  semantics.
- Team Topologies ownership: complicated-subsystem renderer platform work with
  stream-aligned SDF/domain product producers.

## Promotion Readiness

After the design-first contract and intake proposal were applied,
`task production:plan -- --milestone PM-RENDER-SDF-002 --roadmap WR-064`
reported:

- production milestone state: `designing`;
- roadmap state: `ready_next`;
- dependency evidence: `WR-061` is completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`.

Accepted promotion evidence:

- Completed `PM-RENDER-SDF-001` sparse SDF rendering doctrine closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-sdf-001-sparse-sdf-rendering-doctrine/closeout.md`.
- Accepted SDF world rendering and raymarch acceleration doctrine:
  `docs-site/src/content/docs/design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md`.
- Accepted SDF product renderer and GPU residency doctrine:
  `docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md`.
- Accepted renderer scale residency and GPU-driven visibility doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`.
- Completed WR-061 renderer scale working-set registry and residency budget
  closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`.
- This WR-064 design-first sparse SDF residency implementation contract.

Promotion command:

```text
task roadmap:promote -- --id WR-064 --state current_candidate --evidence "Completed PM-RENDER-SDF-001 sparse SDF doctrine closeout, accepted SDF product renderer and scale residency doctrines, completed WR-061 renderer residency-budget closeout, and WR-064 design-first SDF brick/page/clipmap residency implementation contract."
```

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-sparse-sdf-brick-page-and-clipmap-reside
docs-site/src/content/docs/reports/implementation-plans/wr-064-sparse-sdf-brick-page-and-clipmap-residency/plan.md
docs-site/src/content/docs/reports/closeouts/wr-064-sparse-sdf-brick-page-and-clipmap-residency/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/features/world/runtime_cache.rs`:
  `WorldRuntimeCacheResource` currently owns world-render cache entries,
  stale-chunk invalidation, and SDF-related budget byte fields. Keep this as the
  renderer-side invalidation/cache root unless the implementation creates a
  narrower `features/world/sdf_residency.rs` module exported by
  `features/world/mod.rs`.
- `engine/src/plugins/render/features/world/sdf_residency.rs`:
  preferred new module for `RenderSdfResidencyResource`,
  `RenderSdfChunkResidencyEntry`, brick atlas records, page-table residency,
  clipmap window records, generation keys, pressure classification, and
  invalidation summaries if the cache logic grows beyond the existing
  `runtime_cache.rs` responsibilities.
- `engine/src/plugins/render/inspect/sdf_residency.rs`:
  preferred public inspection DTO owner for SDF residency reports. DTOs must
  expose page, brick, clipmap, generation, memory, and fallback/unsupported
  diagnostics without leaking backend handles.
- `engine/src/plugins/render/inspect/gpu_residency.rs` and
  `engine/src/plugins/render/residency/resource.rs`:
  reuse WR-061 budget and finite working-set vocabulary. Extend these only for
  aggregate SDF residency integration, not for SDF product truth.
- `engine/src/plugins/render/inspect/mod.rs` and
  `engine/src/plugins/render/features/world/mod.rs`: module-boundary exports
  only.
- `engine/tests/render_sdf_residency.rs`: focused integration tests for valid
  SDF residency derivation, stale-generation invalidation, missing product
  lineage, page/brick count invariants, clipmap budget pressure, and public
  inspection DTO shape.

## Required Contracts

WR-064 must introduce explicit renderer evidence for:

- addressable SDF product count, selected SDF chunk products, resident SDF
  chunk products, resident page count, resident brick count, resident byte
  estimate, upload byte estimate, and rejected/missing product count;
- brick atlas ranges keyed by product identity, chunk id, chunk revision,
  payload checksum, and renderer cache generation;
- page-table entries keyed by `SdfPageCoord3`, page generation, brick count, and
  resident byte estimate;
- clipmap windows with scale band, center or chunk coverage, source generation,
  resident page/brick coverage, memory pressure, and fallback/unsupported
  diagnostics;
- invalidation evidence for stale chunk revisions, changed product lineage,
  retired products, missing payloads, and cache-generation churn;
- inspection DTOs that report pressure and degraded states without choosing
  product fallback, rebuild policy, or query policy.

The API must fail closed:

- missing lineage, missing payload reference, stale product freshness,
  query-policy mismatch, absent residency request, negative/overflowed counts,
  or resident brick/page counts larger than source payload counts are blocking
  diagnostics;
- clipmap budget pressure and brick-pool pressure must be explicit states;
- unsupported or unavailable GPU residency features must be typed diagnostics,
  not silent success;
- renderer cache records must not infer product freshness, fallback legality,
  collision conservatism, or gameplay interaction semantics.

## Non-Goals

WR-064 does not implement:

- raymarch acceleration, distance mips, empty-space skipping, candidate lists,
  temporal raymarch caches, or step-count diagnostics; those belong to WR-065.
- Runtime examples, visual proof, benchmarks, hardware profiles, or
  production-readiness evidence for the complete SDF renderer path; those
  belong to WR-066.
- SDF authoring, stamp evaluation, collision truth, physics conservatism,
  gameplay field semantics, product fallback policy, query authority, product
  rebuild policy, or product publication ownership.
- Mesh/material lighting, hardware ray queries, temporal upscaling, product
  visual producers, or final renderer perfectionist audit work.
- Persisted SDF residency ABI or cross-domain serialization changes unless an
  ADR and accepted design update approve that scope first.

## Implementation Steps

1. Inspect `domain/world_sdf` payload/product contracts, product selection
   contracts, WR-061 residency DTOs, and existing world-render cache state
   before adding renderer types.
2. Add a narrowly scoped SDF residency module under
   `engine/src/plugins/render/features/world` if the existing
   `runtime_cache.rs` cannot host the brick/page/clipmap records cleanly.
3. Add renderer-owned SDF residency descriptors for chunk products, page-table
   entries, brick atlas records, clipmap windows, budget pressure, and
   invalidation summaries. Use typed identities and generation keys; do not use
   ad hoc strings for product identity or source freshness.
4. Add a derivation function that consumes prepared SDF product selections and
   SDF payload references, computes renderer cache/residency records, and emits
   blocking diagnostics for missing or stale product evidence.
5. Add `engine/src/plugins/render/inspect/sdf_residency.rs` with public DTOs and
   an `inspect_render_sdf_residency(...)` entry point.
6. Integrate SDF residency summary counts with existing renderer residency or
   readiness-budget evidence only where that keeps count vocabulary consistent.
7. Add focused tests in `engine/tests/render_sdf_residency.rs` and update
   renderer reference docs with the normal SDF residency workflow and the
   deferred WR-065/WR-066 boundary.
8. Close WR-064 only after focused tests, docs validation, roadmap validation,
   production validation, and planning validation pass.

## Required Validation

Implementation validation must include:

```text
cargo fmt
cargo test -p engine render_sdf
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If no `render_sdf` filter exists yet, create focused tests in
`engine/tests/render_sdf_residency.rs` and make the filter meaningful before
claiming completion.

## Acceptance Criteria

- Renderer SDF residency records are derived from accepted product selections
  and SDF payload evidence; they do not own product truth.
- Brick atlas, page-table, and clipmap-window inspection DTOs expose source
  identity, generation, resident counts, byte estimates, pressure, invalidation,
  and degraded states.
- Missing, stale, unsupported, over-budget, and inconsistent SDF residency
  inputs fail closed with typed diagnostics.
- Runtime scale evidence vocabulary remains consistent with WR-061 and does not
  collapse SDF addressable, selected, resident, visible, submitted, or measured
  work into one status.

## Stop Conditions

Stop before implementation if:

- WR-064 has not been applied to active roadmap state and promoted through the
  required roadmap workflow;
- implementation would move SDF product truth, payload freshness, query policy,
  fallback legality, collision conservatism, gameplay semantics, or product
  rebuild policy into renderer code;
- implementation requires a persisted cross-domain ABI or dependency-direction
  change without accepted ADR/design evidence;
- page, brick, clipmap, or cache-generation identity cannot be represented with
  typed contracts;
- tests cannot prove that missing/stale/unsupported SDF residency inputs fail
  closed instead of producing success-shaped DTOs.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-064-sparse-sdf-brick-page-and-clipmap-residency/closeout.md
```

The closeout must record:

- exact changed modules, functions, DTOs, tests, and docs;
- architecture governance decision and ADR decision;
- SDF product-to-renderer translation boundary evidence;
- page, brick, clipmap, invalidation, budget-pressure, and diagnostic evidence;
- focused validation output;
- roadmap, production, docs, and planning validation output;
- completion quality and known quality gaps.

Expected completion quality is `bounded_contract`. `runtime_proven` remains
blocked until WR-066 proves the SDF renderer path through examples, benchmarks,
visual evidence, and production readiness. `perfectionist_verified` remains
blocked until `PT-RENDER-PERFECTION` completes the final no-gap renderer audit.

## Perfectionist Closeout Audit

WR-064 must preserve visible gaps for later tracks instead of claiming runtime
perfection:

- WR-065 owns raymarch acceleration, candidate lists, distance mips, and
  step-count diagnostics.
- WR-066 owns runtime SDF examples, visual proof, benchmarks, and
  runtime-proven evidence.
- PT-RENDER-PERFECTION owns final no-gap verification.

Anti-drift guards must prevent:

- descriptor-only completion where SDF residency DTOs are never derived from
  product/payload evidence;
- prepared-data-only completion where cache records never expose inspection
  diagnostics;
- fallback-only completion that silently treats missing SDF payloads as valid
  resident pages;
- renderer-owned product truth or query policy;
- untyped string identities for page, brick, clipmap, product, or generation
  evidence.
