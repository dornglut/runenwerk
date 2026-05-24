---
title: WR-077 Renderer Product Visuals Vegetation Water Atmosphere Weather And Field Visuals Implementation Contract
description: Design-first contract for product-owned vegetation, water, atmosphere, weather, and field visual producer integration.
status: active
owner: engine
layer: engine-runtime / product-render-integration
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/renderer-product-visual-producers-platform-design.md
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-077 Renderer Product Visuals Vegetation Water Atmosphere Weather And Field Visuals Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-PRODUCT-VISUALS-003` and
`WR-077`.

This row must let product-owned vegetation, grass, water, wetness, atmosphere,
weather, and field visual producers submit prepared renderer contributions,
product-surface outputs, and residency requests through shared renderer
contracts. The renderer may own execution ordering, bounded working sets,
derived residency, fallback diagnostics, timing, and inspection DTOs. Product
domains keep semantic truth, source products, simulation meaning, freshness,
authority, fallback legality, rebuild policy, and user-facing meaning.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-077` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-product-visual-producers-platform-design.md`:
  accepted doctrine for product-owned visual producers and renderer execution
  APIs.
- `docs-site/src/content/docs/design/accepted/field-product-contracts-diagnostics-and-residency-design.md`:
  accepted field product source-truth, freshness, diagnostics, residency,
  authority, and query-policy boundary.
- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  accepted finite working-set, residency budget, scale band, degraded mode, and
  inspection vocabulary.
- `docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md`:
  accepted renderer ownership of derived SDF and field-product GPU residency
  while product domains keep world truth.
- `docs-site/src/content/docs/design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md`:
  accepted temporal input, history, reactive-mask, depth, exposure, and
  diagnostic vocabulary for visual products that feed temporal reconstruction.
- `docs-site/src/content/docs/reports/closeouts/pm-render-product-visuals-001-product-visual-producer-doctrine/closeout.md`:
  completed product visual producer doctrine evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`:
  completed bounded scale, visibility, LOD, compaction, and indirect-submission
  inspection evidence for `PM-RENDER-SCALE-003`.
- `docs-site/src/content/docs/reports/closeouts/wr-064-sparse-sdf-brick-page-and-clipmap-residency/closeout.md`:
  completed sparse SDF residency, invalidation, and diagnostics evidence for
  `PM-RENDER-SDF-002`.
- `docs-site/src/content/docs/reports/closeouts/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/closeout.md`:
  completed temporal input, history, jitter, and dynamic-resolution inspection
  evidence for `PM-RENDER-TEMPORAL-001`.
- `engine/src/plugins/render/features/mod.rs`,
  `engine/src/plugins/render/features/world/mod.rs`,
  `engine/src/plugins/render/runtime/frame_prepare.rs`, and
  `engine/src/plugins/render/inspect/prepared_frame.rs`: existing renderer
  feature, world visual, frame preparation, and prepared-frame inspection
  boundaries.

## Readiness

`task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-003 --roadmap
WR-077` reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- roadmap dependencies: none;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-077-renderer-product-visuals-vegetation-water-atmosphere-weather-and-field-visuals/plan.md`.

This contract clears the design-first gap by naming the accepted doctrine,
required completed evidence, owner boundary, implementation modules,
validation, stop conditions, and closeout quality before implementation.

After applying the intake proposal, WR-077 may be promoted only when:

- `PM-RENDER-PRODUCT-VISUALS-001` remains completed with valid closeout
  evidence;
- `WR-062`, `WR-064`, and `WR-070` remain completed with valid closeout
  evidence;
- `PM-RENDER-PRODUCT-VISUALS-003` is active and still selected by the stack
  coordinator;
- `task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-003 --roadmap
  WR-077` reports a promotable or promotion-contract action rather than
  `design_first`;
- roadmap, production, docs, and planning validators pass.

WR-077 implementation may start only after the row is `current_candidate`, this
contract remains active, and the stack coordinator advances from gate repair to
implementation.

The promotion preflight now reports:

- production milestone state: `designing`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-062:completed`, `WR-064:completed`, and
  `WR-070:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-077 --state current_candidate --evidence "Accepted product visual producer doctrine, active WR-077 world visual producer implementation contract, completed PM-RENDER-PRODUCT-VISUALS-001 closeout, completed WR-062 scale visibility closeout, completed WR-064 SDF residency closeout, and completed WR-070 temporal inputs closeout."
```

WR-077 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-PRODUCT-VISUALS-003`.

After promotion, `task production:plan -- --milestone
PM-RENDER-PRODUCT-VISUALS-003 --roadmap WR-077` reports:

- production milestone state: `designing`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-062:completed`, `WR-064:completed`, and
  `WR-070:completed`;
- next action: `write_implementation_contract`.

WR-077 implementation may start only after this current-candidate evidence
remains true and the stack coordinator selects implementation for
`PM-RENDER-PRODUCT-VISUALS-003`.

## Architecture Governance

- Recommendation: use this contract as the design gate and defer product code
  until WR-077 is applied, promoted, selected, and rechecked by
  `task production:plan`.
- DDD bounded context owner: `engine/src/plugins/render` owns renderer feature
  execution contracts, derived residency and visibility records, fallback
  diagnostics, temporal input declarations, product-surface integration, and
  prepared-frame inspection.
- Product-owner vocabulary: vegetation/grass simulation state, water/wetness
  flow truth, shoreline semantics, atmosphere/weather/day-night meaning, field
  product lineage, freshness, authority, query policy, fallback legality, and
  rebuild policy.
- Renderer-owned vocabulary: render feature descriptor, contribution collector,
  prepared world visual payload, scale band, residency request, bounded working
  set, product-surface target, temporal input declaration, diagnostic kind,
  timing label, and inspection DTO.
- Translation boundary: product domains emit prepared renderer data through
  resources, product selections, or registered collectors. The renderer must not
  infer simulation authority, water truth, weather meaning, foliage lifecycle,
  or fallback legality from draw counts, field names, shader labels, or cached
  GPU records.
- Clean Architecture check: dependency direction stays legal when renderer
  runtime consumes domain/foundation product contracts and product-prepared
  DTOs. No domain crate may depend on `engine`, renderer internals, backend
  handles, or concrete GPU resources for this slice.
- ADR requirement: no new ADR is required while WR-077 consumes accepted
  product visual, field product, SDF residency, scale, and temporal designs.
  Stop for ADR if implementation changes product truth ownership, creates a
  durable cross-domain world-visual ABI, or makes renderer fallback policy
  authoritative for product domains.
- ATAM-lite: the main tradeoff is a discoverable typed world-visual surface
  versus one generic field payload. The decision is to use explicit
  renderer-owned typed payloads for world visual families while preserving the
  registered feature contribution path, so diagnostics remain inspectable
  without moving product semantics into renderer code.
- Strangler Fig: not applicable unless implementation replaces an existing
  world/SDF renderer path. If replacement is required, route old and new paths
  side by side behind feature descriptors and stop for a migration plan.
- Fitness functions: focused renderer tests must prove source-truth separation,
  missing/stale/fallback diagnostics, scale/residency budget inspection,
  temporal input declaration, bounded far-field summaries, and prepared-frame
  inspection. Docs, roadmap, production, and planning validators must pass
  before closeout.
- Team Topologies ownership: complicated-subsystem renderer platform work that
  enables stream-aligned world and product visual producers.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
engine/examples
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-product-visuals-vegetation-wate
docs-site/src/content/docs/reports/implementation-plans/wr-077-renderer-product-visuals-vegetation-water-atmosphere-weather-and-field-visuals/plan.md
docs-site/src/content/docs/reports/closeouts/wr-077-renderer-product-visuals-vegetation-water-atmosphere-weather-and-field-visuals/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/features/mod.rs` module: add renderer feature IDs,
  labels, descriptors, fallback policies, and built-in registration for the
  vegetation/water/atmosphere/weather/field visual family.
- `engine/src/plugins/render/features/world/mod.rs` module: expose cohesive
  world visual submodules without creating catch-all helper files.
- `engine/src/plugins/render/features/world/visuals/mod.rs` module: add typed
  prepared payload structs, collector descriptors, and resource-to-contribution
  translation for vegetation, grass, water, wetness, atmosphere, weather, and
  field visual summaries if implementation needs a new subsystem.
- `engine/src/plugins/render/features/world/sdf_residency.rs` module: consume
  existing SDF residency summaries and diagnostics only as renderer-owned
  derived cache evidence; do not make SDF residency product truth.
- `engine/src/plugins/render/runtime/frame_prepare.rs` module: keep world visual
  collection inside the existing registered contribution and prepared-frame
  path; do not add live ECS extraction during submission.
- `engine/src/plugins/render/inspect/prepared_frame.rs` module: expose world
  visual contribution status, scale band, residency state, temporal input,
  fallback, unsupported, and over-budget evidence in inspection DTOs.
- `engine/tests/render_product_visual_world.rs` module: add focused tests for
  ready, missing, stale, fallback, unsupported, over-budget, bounded far-field,
  residency, scale-band, and temporal-input diagnostics.
- `engine/examples/render_product_visual_world.rs` module: add a finite
  non-windowed evidence example only if tests alone cannot prove the intended
  renderer consumption chain.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the public world visual producer contract after implementation.

## Non-Goals

- Do not implement vegetation simulation, water simulation, weather systems,
  atmosphere truth, day/night authority, erosion/sand/heat/humidity simulation,
  field product generation, gameplay effects, or editor authoring workflows.
- Do not move product truth, source assets, freshness, fallback authority,
  rebuild policy, or user-facing meaning into renderer code.
- Do not add backend-specific water, atmosphere, or weather GPU algorithms as
  the baseline contract.
- Do not widen foundation or domain crates to expose renderer-only handles.
- Do not make SDF residency, temporal history, or scale diagnostics
  authoritative product state.
- Do not claim `runtime_proven` or `perfectionist_verified`; PM-004 owns final
  cross-family product visual production evidence.

## Required Implementation Shape

WR-077 must provide a bounded renderer integration that proves:

1. Vegetation, water, atmosphere, weather, and field visual producers submit
   prepared contributions without renderer-owned semantic truth.
2. Scale band, residency request, product generation, fallback state, and
   unsupported capability state are explicit prepared renderer data.
3. Far-field and summary visual products use bounded working sets and expose
   addressable, resident, visible, and submitted counts separately.
4. SDF residency and field-product diagnostics remain derived renderer cache
   evidence and do not become authoritative field truth.
5. Temporal input declarations are visible when world visual products require
   motion vectors, depth, exposure, reactive masks, weather masks, or history
   signatures.
6. Missing, stale, fallback, unsupported, over-budget, and invalid-residency
   states produce diagnostics instead of silent success.
7. Public docs teach product producers to prepare world visual contributions
   through renderer contracts without depending on renderer internals.

## Acceptance Criteria

- `PM-RENDER-PRODUCT-VISUALS-003` can close at `bounded_contract` quality with
  visible known gaps for animation/deformation and final product visual
  production evidence.
- WR-077 closeout links focused tests and any example output that prove prepared
  world visual contributions, residency requests, scale-band summaries,
  temporal declarations, and fallback diagnostics.
- Roadmap archive evidence names the accepted design, this active contract,
  completed prerequisite closeouts, validation commands, and remaining PM-004
  scope.
- Product semantic truth remains outside renderer modules.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_product_visual_world
cargo test -p engine render_sdf_residency
cargo test -p engine render_temporal_inputs
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If implementation touches shared frame preparation, prepared-frame inspection,
render-flow validation, or SDF residency code, add the smallest focused
existing test filter for that shared subsystem before closeout.

## Stop Conditions

Stop before product code if:

- the stack coordinator no longer selects `PM-RENDER-PRODUCT-VISUALS-003`;
- WR-077 is still `blocked_deferred` or lacks valid promotion evidence;
- `task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-003 --roadmap
  WR-077` still reports `design_first`, a promotion blocker, or a metadata
  blocker;
- prerequisite closeout evidence for PM-001, WR-062, WR-064, or WR-070 is
  missing or invalid;
- implementation would require renderer code to own vegetation, water,
  atmosphere, weather, field-product, or simulation semantic truth;
- a product domain needs a new durable cross-domain ABI not covered by accepted
  designs;
- fallback legality would need to move from product domains into renderer
  policy;
- far-field or summary products cannot stay bounded by explicit residency,
  visibility, and submitted-work counts;
- focused tests cannot prove missing, stale, fallback, residency, scale, and
  temporal diagnostic behavior.

## Closeout Requirements

Closeout must update:

- `docs-site/src/content/docs/reports/closeouts/wr-077-renderer-product-visuals-vegetation-water-atmosphere-weather-and-field-visuals/closeout.md`;
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` and
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` as needed;
- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- renderer public docs under
  `docs-site/src/content/docs/engine/reference/plugins/render`.

Required validation before closeout:

```text
cargo fmt
cargo test -p engine render_product_visual_world
cargo test -p engine render_sdf_residency
cargo test -p engine render_temporal_inputs
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Perfectionist Closeout Audit

WR-077 is expected to close as `bounded_contract`.

Known quality gaps that must remain visible:

- Animation/deformation and cross-family product visual evidence remain PM-004 /
  WR-078 scope.
- Runtime production examples, benchmark artifacts, hardware profiles, and
  final no-gap stack verification remain downstream product-visual and
  perfectionist-audit scope unless explicitly completed by a later legal
  production milestone.
- PM-003 must not claim product-domain simulation truth, weather/water/field
  authoring completeness, or backend-specific GPU visual quality.

The closeout must not claim `runtime_proven` or `perfectionist_verified` unless
the final product visual evidence milestone and perfectionist audit have both
completed with no known quality gaps.
