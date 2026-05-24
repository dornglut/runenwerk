---
title: WR-076 Renderer Product Visuals Particles VFX Trails And Decals Implementation Contract
description: Design-first contract for product-owned particle, VFX, trail, decal, sorting, and transparency render producer integration.
status: active
owner: engine
layer: engine-runtime / product-render-integration
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/accepted/renderer-product-visual-producers-platform-design.md
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-076 Renderer Product Visuals Particles VFX Trails And Decals Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-PRODUCT-VISUALS-002` and
`WR-076`.

This row must let product-owned particle, VFX, trail, decal, sorting, and
transparency producers submit prepared renderer contributions and residency
requests through shared renderer contracts. The renderer may own execution
ordering, fallback diagnostics, scale visibility, temporal input consumption,
and inspection DTOs. Product domains keep semantic truth, emitter lifetime,
authoring state, freshness, authority, fallback legality, and user-facing
meaning.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-076` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-product-visual-producers-platform-design.md`:
  accepted doctrine for product-owned visual producers and renderer execution
  APIs.
- `docs-site/src/content/docs/design/accepted/field-product-contracts-diagnostics-and-residency-design.md`:
  accepted field product source-truth, diagnostics, and residency boundary.
- `docs-site/src/content/docs/reports/closeouts/pm-render-product-visuals-001-product-visual-producer-doctrine/closeout.md`:
  completed product visual producer doctrine evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md`:
  completed procedural instance API evidence for bounded local visuals.
- `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`:
  completed visibility, LOD, compaction, and indirect-submission inspection
  evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/closeout.md`:
  completed temporal input, history, and dynamic-resolution inspection
  evidence.
- `engine/src/plugins/render/features/mod.rs`, `engine/src/plugins/render/frame/contribution_registry.rs`,
  `engine/src/plugins/render/runtime/frame_prepare.rs`, and
  `engine/src/plugins/render/inspect/prepared_frame.rs`: existing renderer
  feature, registered payload, frame preparation, and prepared-frame inspection
  boundaries.

## Readiness

`task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-002 --roadmap
WR-076` reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- roadmap dependencies: none;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-076-renderer-product-visuals-particles-vfx-trails-and-decals/plan.md`.

This contract clears the design-first gap by naming the accepted doctrine,
required completed evidence, owner boundary, implementation modules,
validation, stop conditions, and closeout quality before implementation.

After applying the intake proposal, WR-076 may be promoted only when:

- `PM-RENDER-PRODUCT-VISUALS-001` remains completed with valid closeout
  evidence;
- `WR-058`, `WR-062`, and `WR-070` remain completed with valid closeout
  evidence;
- `PM-RENDER-PRODUCT-VISUALS-002` is active and still selected by the stack
  coordinator;
- `task production:plan -- --milestone PM-RENDER-PRODUCT-VISUALS-002 --roadmap
  WR-076` reports a promotable or promotion-contract action rather than
  `design_first`;
- roadmap, production, docs, and planning validators pass.

WR-076 implementation may start only after the row is `current_candidate`, this
contract remains active, and the stack coordinator advances from gate repair to
implementation.

The promotion preflight now reports:

- production milestone state: `designing`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-058:completed`, `WR-062:completed`, and
  `WR-070:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-076 --state current_candidate --evidence "Accepted product visual producer doctrine, completed PM-RENDER-PRODUCT-VISUALS-001 closeout, completed WR-058 procedural instance API closeout, completed WR-062 scale visibility closeout, completed WR-070 temporal inputs closeout, and active WR-076 particle/VFX implementation contract."
```

WR-076 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-PRODUCT-VISUALS-002`.

After promotion, `task production:plan -- --milestone
PM-RENDER-PRODUCT-VISUALS-002 --roadmap WR-076` reports:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- roadmap blocker: `B2`;
- roadmap dependencies: `WR-058:completed`, `WR-062:completed`, and
  `WR-070:completed`;
- next action: `write_implementation_contract`;
- implementation contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-076-renderer-product-visuals-particles-vfx-trails-and-decals/plan.md`.

WR-076 implementation may start only after this current-candidate evidence
remains true and the stack coordinator selects
`execute_next_wr_implementation_contract`.

## Governance Decisions

- DDD bounded context owner: `engine/src/plugins/render` owns renderer feature
  execution contracts, registered prepared payloads, fallback diagnostics,
  residency request consumption, and prepared-frame inspection.
- Product-owner vocabulary: particle emitters, VFX graph semantics, trail
  authoring, decal source placement, transparency intent, sorting intent,
  freshness, authority, fallback legality, and rebuild policy.
- Renderer-owned vocabulary: render feature descriptor, contribution collector,
  registered payload kind, prepared contribution status, fallback policy,
  scale band, residency request, temporal input declaration, timing label, and
  inspection DTO.
- Translation boundary: product domains emit prepared renderer data through
  resources or registered collectors. The renderer must not infer emitter
  lifetime, gameplay meaning, simulation truth, authoring state, or fallback
  authority from draw counts or payload strings.
- Clean Architecture check: dependency direction stays legal because renderer
  runtime consumes domain/foundation product contracts and product-prepared
  DTOs. No domain crate may depend on `engine`, renderer internals, backend
  handles, or concrete GPU resources for this slice.
- ADR requirement: no new ADR is required while WR-076 consumes the accepted
  product visual producer doctrine and stays inside renderer execution
  contracts. Stop for ADR if the implementation changes product truth
  ownership, adds a durable cross-domain particle/VFX ABI, or makes renderer
  fallback policy authoritative for product domains.
- ATAM-lite: the main tradeoff is a discoverable typed particle/VFX surface
  versus a generic registered payload path. The decision is to use typed
  renderer-owned payload structs for the particle/VFX family while preserving
  `PreparedRegisteredFeaturePayload` for extension, so common diagnostics stay
  inspectable without turning renderer into the product semantic owner.
- Strangler Fig: not applicable; this is greenfield product visual integration
  on existing feature contribution and prepared-frame boundaries.
- Fitness functions: focused renderer tests must prove collector registration,
  missing/stale/fallback diagnostics, scale/residency request inspection,
  temporal input declaration, and prepared-frame inspection. Docs validation
  and roadmap/production checks must pass before closeout.
- Team Topologies ownership: complicated-subsystem renderer platform work that
  enables stream-aligned product visual producers.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
engine/examples
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-product-visuals-particles-vfx-t
docs-site/src/content/docs/reports/implementation-plans/wr-076-renderer-product-visuals-particles-vfx-trails-and-decals/plan.md
docs-site/src/content/docs/reports/closeouts/wr-076-renderer-product-visuals-particles-vfx-trails-and-decals/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/features/mod.rs` module: add renderer feature IDs,
  labels, descriptors, fallback policy, and resource registration for the
  particle/VFX/trail/decal family.
- `engine/src/plugins/render/features/particle_vfx/mod.rs` module: add typed
  prepared payload structs, collector descriptors, and resource-to-contribution
  translation for particles, VFX, trails, decals, sorting, and transparency.
- `engine/src/plugins/render/frame/contributions.rs` module: add only the
  narrow contribution helpers needed to insert or inspect the typed particle
  visual payload; prefer `PreparedRegisteredFeaturePayload` if enum growth would
  make product families less discoverable.
- `engine/src/plugins/render/runtime/frame_prepare.rs` module: keep product
  visual collection inside the existing `collect_registered_feature_contributions`
  path unless a typed helper is required for diagnostics.
- `engine/src/plugins/render/inspect/prepared_frame.rs` module: expose
  particle/VFX contribution status, payload kind, residency request count,
  temporal input declarations, and fallback policy in inspection DTOs.
- `engine/tests/render_product_visual_particles.rs` module: add focused tests
  for ready, missing, stale, disabled, fallback, over-budget, sorting,
  transparency, residency, and temporal-input diagnostics.
- `engine/examples/render_product_visual_particles.rs` module: add a finite
  non-windowed evidence example if tests alone cannot prove the user-visible
  renderer chain.

## Non-Goals

- Do not implement product-domain particle simulation, VFX graph evaluation,
  trail authoring, decal placement semantics, gameplay effects, or editor tools.
- Do not move product truth, source assets, fallback authority, freshness,
  rebuild policy, or user-facing meaning into renderer code.
- Do not add backend-specific GPU particle simulation, shader hot paths,
  vendor-specific transparency features, or hardware claims in this slice.
- Do not widen foundation or domain crates to expose renderer-only handles.
- Do not claim `runtime_proven` or `perfectionist_verified`; PM-004 owns final
  product visual production evidence.

## Required Implementation Shape

WR-076 must provide a bounded renderer integration that proves:

1. Particle/VFX/trail/decal product visuals submit prepared contributions
   without renderer-owned semantic truth.
2. Sorting and transparency intent are explicit prepared renderer data, not
   inferred from insertion order or string labels.
3. Residency requests are typed and inspectable.
4. Temporal input declarations are visible when particle-style visuals require
   motion vectors, reactive masks, depth, exposure, or history-sensitive
   behavior.
5. Missing, stale, disabled, fallback, unsupported, and over-budget states
   produce diagnostics instead of silent success.
6. Scale visibility and submission counts remain bounded and inspectable.
7. The public docs teach product producers to prepare contributions through the
   renderer contract without becoming renderer internals.

## Acceptance Criteria

- `PM-RENDER-PRODUCT-VISUALS-002` can close at `bounded_contract` quality with
  visible known gaps for world visual producers and final product visual
  production evidence.
- WR-076 closeout links focused tests and any example output that prove
  prepared contributions, residency requests, temporal declarations, and
  fallback diagnostics.
- Roadmap archive evidence names the accepted design, this active contract,
  completed prerequisite closeouts, validation commands, and remaining PM-003 /
  PM-004 scope.
- Product semantic truth remains outside renderer modules.

## Stop Conditions

Stop before implementation if:

- the stack coordinator no longer selects `PM-RENDER-PRODUCT-VISUALS-002`;
- WR-076 is still `blocked_deferred` or lacks valid promotion evidence;
- the implementation would require renderer code to own particle, VFX, trail,
  or decal semantic truth;
- a product domain needs a new durable cross-domain ABI not covered by the
  accepted design;
- fallback legality would need to move from product domains into renderer
  policy;
- write-scope conflicts or validation failures appear;
- focused tests cannot prove missing, stale, fallback, residency, and temporal
  diagnostic behavior.

## Closeout Requirements

Closeout must update:

- `docs-site/src/content/docs/reports/closeouts/wr-076-renderer-product-visuals-particles-vfx-trails-and-decals/closeout.md`;
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` and
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` as needed;
- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- renderer public docs under
  `docs-site/src/content/docs/engine/reference/plugins/render`.

Required validation before closeout:

```text
cargo fmt
cargo test -p engine render_product_visual_particles
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

WR-076 is expected to close as `bounded_contract`.

Known quality gaps that must remain visible:

- Vegetation, water, atmosphere, weather, and field visuals remain PM-003 /
  WR-077 scope.
- Animation/deformation and cross-family product visual evidence remain PM-004 /
  WR-078 scope.
- Hardware pixel evidence, benchmark artifacts, and no-gap stack verification
  remain PT-RENDER-PERFECTION scope unless explicitly completed by later
  product visual evidence.

The closeout must not claim `runtime_proven` or `perfectionist_verified` unless
the final product visual evidence milestone and perfectionist audit have both
completed with no known quality gaps.
