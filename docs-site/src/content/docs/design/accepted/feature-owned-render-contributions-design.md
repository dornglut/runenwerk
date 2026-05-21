---
title: Feature-Owned Render Contributions Design
description: Accepted design for PM-RENDER-PG-003 feature-owned prepared render contribution collectors without renderer-owned product truth.
status: accepted
owner: engine
layer: engine-runtime / render prepared-frame contracts
canonical: true
last_reviewed: 2026-05-21
related_designs:
  - ./render-product-graph-platform-design.md
  - ./render-contract-ergonomics-design.md
  - ./render-execution-graph-compiler-maturity-design.md
  - ./product-surface-platform-hardening-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./render-fragment-data-driven-maturity-design.md
related_roadmaps:
  - ../../engine/roadmaps/fully-featured-renderer-roadmap.md
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# Feature-Owned Render Contributions Design

## Status

This is the accepted design contract for `PM-RENDER-PG-003`.

It accepts a bounded design for replacing central render feature contribution
growth with typed, inspectable, feature-owned collectors. It does not authorize
product code changes by itself. Implementation still requires the normal
production planning gate, legal WR state, focused validation, closeout evidence,
and a rerun of `task ai:goal -- --track PT-RENDER-PG`.

## Goal

Make render feature contributions extensible without turning the renderer into
the owner of product truth:

```text
owning domain or app prepares product/render-facing state
  -> feature-owned contribution collector declares required prepared resources
  -> RenderPrepare runs collectors and validates typed contribution packets
  -> compatibility adapter fills PreparedFrameContributions
  -> RenderSubmit consumes PreparedRenderFrame only
```

The renderer owns the prepared-frame contribution contract, validation,
inspection, ordering, fallback gate, and backend-neutral execution handoff. It
must not compute product truth, product freshness, authority, fallback legality,
rebuild policy, product dependencies, or residency policy.

## Current Friction

The current contribution path has correct submit-time ownership but too much
central growth:

- `engine/src/plugins/render/frame/contributions.rs::PreparedFeaturePayload`
  grows a central enum variant for every feature payload.
- `engine/src/plugins/render/frame/contributions.rs::PreparedFrameContributions`
  owns feature-specific insertion helpers.
- `engine/src/plugins/render/runtime/frame_prepare.rs::build_frame_feature_contributions`
  centrally extracts feature resources from the ECS world.
- `engine/src/plugins/render/renderer/prepare.rs::hash_prepared_feature_contribution`
  centrally matches every payload shape for runtime signatures.
- Backend preparation and render-flow execution consume prepared contribution
  gates correctly, but new features still need central payload and collector
  edits before they can participate.

The defect is extensibility and inspection ownership, not permission for live
submit extraction. `RenderSubmit` must continue to consume `PreparedRenderFrame`
only.

## Locked Decisions

`PM-RENDER-PG-003` has these accepted decisions:

- Add a feature contribution collector registry beside the current central path.
- New render features must be able to register typed contribution collectors
  without adding new feature-specific variants to `PreparedFeaturePayload`.
- A single compatibility payload or packet type may bridge registered payloads
  into the existing `PreparedFrameContributions` structure during migration.
- Type erasure is allowed only behind typed registration metadata, runtime
  signature, validation, and inspection contracts. Opaque `Box<dyn Any>` maps,
  stringly payload bags, unvalidated plugin packets, and submit-time downcasts
  are forbidden.
- Collectors must declare the prepared resources they read. The collector
  context should expose only declared resources or fail validation before
  collection.
- Collectors produce backend-neutral prepared contribution packets, status,
  fallback policy, diagnostics, and inspection DTOs.
- Existing central enum variants remain during migration. They are not removed
  in the first PM-003 implementation slice.
- `RenderSubmit` and backend execution do not perform live ECS extraction.
- Product truth, freshness, authority, fallback legality, rebuild policy,
  product dependency truth, and residency policy remain with Product Jobs,
  product domains, and owning app/domain producers.

## Ownership

DDD bounded context owner:

- `engine/src/plugins/render` owns render feature contribution registration,
  collector execution during render prepare, prepared-frame contribution
  validation, diagnostics, inspection, fallback gates, and backend-neutral
  contribution packets.

Translation boundaries:

- Product domains and apps publish prepared render-facing resources and
  diagnostics. They own semantic truth and mutation policy.
- Render collectors translate those prepared resources into
  `PreparedFeatureContribution` data for one `RenderFeatureId`.
- Render backend code translates validated prepared contribution packets into
  derived GPU execution state.

Team Topologies ownership:

- Complicated-subsystem render owner enabling stream-aligned feature and product
  teams.

No new `foundation` or `domain` crate is required for PM-003. Create a
cross-domain contract crate only if a later accepted design needs render
contribution descriptions outside the engine render plugin.

## Public Contract Shape

The implementation should add explicit render-owned contracts with names close
to the existing vocabulary:

```text
engine/src/plugins/render/frame/contribution_registry.rs
engine/src/plugins/render/frame/contribution_diagnostics.rs
engine/src/plugins/render/frame/contributions.rs
engine/src/plugins/render/runtime/frame_prepare.rs
engine/src/plugins/render/inspect/prepared_frame.rs
```

Expected concepts:

- `RenderFeatureContributionCollectorId`;
- `RenderFeatureContributionPayloadKind`;
- `RenderFeatureContributionCollectorDescriptor`;
- `RenderFeatureContributionResourceRequirement`;
- `RenderFeatureContributionCollectorRegistryResource`;
- `PreparedRegisteredFeaturePayload` or equivalent compatibility packet;
- `PreparedFeatureContributionDiagnostic`;
- prepared-frame inspection fields for collector id, payload kind, status,
  fallback policy, declared resources, runtime signature, and diagnostics.

Collector descriptors must include:

- feature id;
- collector id;
- payload kind;
- ordered resource requirements;
- fallback policy default;
- capability declarations needed by render-flow validation;
- validation function or typed validator;
- inspection function;
- runtime signature function.

The registry may integrate with `RenderFeatureRegistryResource`, but it must not
make the feature descriptor registry a product-truth registry. `RenderFeatureId`
ordering, fallback policy, and render execution capability are render concerns;
source product meaning is not.

## Collector Execution Contract

Render prepare should collect contributions in a deterministic order:

1. Synchronize `RenderFeatureRegistryResource`.
2. Resolve the execution feature ids from compiled render flows.
3. Resolve registered contribution collectors for those feature ids.
4. Validate collector metadata and declared resource availability.
5. Run collectors against declared prepared resources only.
6. Record typed diagnostics for missing, stale, disabled, invalid, and
   conflicting contributions.
7. Fill the existing `PreparedFrameContributions` structure through a
   compatibility adapter.
8. Insert missing contributions for unresolved execution features according to
   the feature fallback policy.

The collector context should not expose backend handles, command encoders,
render pipelines, bind groups, or app/editor private workflow objects.

Collectors may report that a contribution is `Ready`, `Stale`, `Disabled`, or
`Missing`, but they must not decide product freshness or fallback legality from
raw product graphs. Those values must already be represented in prepared
resource state or domain diagnostics.

## Typed Payload And Inspection Rules

Registered payloads must be typed enough to be validated and inspected:

- each payload has a stable payload kind;
- each payload has a deterministic runtime signature;
- each payload exposes an inspection DTO without backend handles;
- each payload validates portable limits and required aliases before submit;
- each payload can be rejected with structured diagnostics;
- each payload remains associated with exactly one `RenderFeatureId` and
  collector id in a prepared frame.

A generic registered payload wrapper is acceptable as the strangler bridge, but
the wrapper must preserve the typed payload kind and inspection hooks. A new
feature must not require adding a feature-specific enum variant to the central
payload enum.

## Diagnostics Contract

PM-003 diagnostics must be structured and producer/feature-oriented enough for
tools:

- feature id;
- collector id;
- payload kind when known;
- required resource type name when resource resolution fails;
- severity;
- status;
- message;
- optional source label or owning prepared-resource label.

Diagnostics must cover:

- duplicate collector registration for one feature/payload kind;
- collector references unknown feature id;
- missing declared prepared resource;
- collector emits a feature id different from its descriptor;
- collector emits an unregistered payload kind;
- collector validation rejects its payload;
- collector conflict when two collectors try to publish the same feature
  without an explicit merge policy.

Diagnostics are prepared-frame diagnostics. They do not replace product-domain
diagnostics for stale products, failed rebuilds, illegal fallbacks, or authority
violations.

## Strangler Migration Sequence

Implementation must be migration-safe:

1. Add collector registry, typed registered payload packet, diagnostics, and
   inspection support beside the current central path.
2. Keep all existing `PreparedFeaturePayload` feature-specific variants and
   insertion helpers.
3. Add a compatibility adapter that can insert registered payload packets into
   `PreparedFrameContributions`.
4. Prove a test-only registered feature can contribute without adding a new
   feature-specific central payload variant.
5. Migrate one low-risk existing contribution path to the collector registry.
   Prefer scene route or another small prepared contribution that has no product
   truth, no material lowering, and no backend allocation policy.
6. Preserve equivalent prepared-frame inspection and fallback behavior for the
   migrated path.
7. Add a guard that rejects new feature-specific central payload variants for
   newly registered features once the registered path exists.
8. Leave material, draw, world, deformation, wind, cave, detail, and procedural
   world migration for later bounded slices unless the implementation contract
   explicitly narrows one of them.

This is a Strangler Fig migration. The old and new paths must coexist until
inspection, diagnostics, and renderer behavior prove equivalence.

## Relationship To Render Fragments

PM-003 is not the render fragment implementation.

Render fragments describe reusable render-flow pieces, merge provenance, asset
or package metadata, validation, hot reload, and last-good fragment promotion.
PM-003 only creates the prepared contribution collector contract that fragments
and feature implementations can later target.

`PM-RENDER-PG-007` still owns render fragment assets, merge, hot reload, and
last-good fragment behavior.

## Relationship To Compiler Maturity

PM-003 does not mature the whole-frame render execution graph compiler.

The collector registry may declare render capabilities required by a feature
contribution, but `PM-RENDER-PG-004` still owns compiler validation for render
resources, pass order, target aliases, history scopes, resource lifetimes, and
backend capability mismatches.

## Architecture Governance Result

Architecture governance review for this design resolves as:

- DDD owner: `engine/src/plugins/render`.
- Dependency direction: unchanged. The implementation remains in engine runtime
  and may consume foundation/domain prepared contracts through existing engine
  dependencies.
- ADR need: no new ADR while the contract remains engine-local and preserves
  the accepted Render Product Graph Platform boundary. Add an ADR only if
  contribution payloads become a cross-domain ABI, persisted plugin contract,
  or external mod interface.
- ATAM tradeoff: typed registry plus inspection has more boilerplate than a raw
  trait-object map, but it preserves diagnosability, validation, and migration
  safety. That tradeoff is accepted.
- Required fitness functions: registry validation tests, duplicate/conflict
  diagnostic tests, registered-payload inspection tests, compatibility adapter
  tests, migrated-feature equivalence tests, and submit-boundary tests proving
  no live ECS extraction in backend submit.

## Implementation Gates

Before code changes:

1. Rerun `task ai:goal -- --track PT-RENDER-PG`.
2. Create or select a bounded PM-003 implementation WR row through the roadmap
   workflow. Do not repurpose `WR-010`; `WR-010` remains the render-fragment
   and hot-reload row for `PM-RENDER-PG-007`.
3. Run `task production:plan -- --milestone PM-RENDER-PG-003 --roadmap <WR-ID>`
   only when `task ai:goal` selects an active or ready-next implementation
   action for that bounded PM-003 row.
4. Promote or switch WR state only through the roadmap workflow.
5. Implement one bounded collector-registry slice, then validate and close it
   out before starting PM-004.

## Validation Required For Implementation

The implementation contract must include focused tests for:

- collector descriptor registration and ordering;
- duplicate collector and unknown feature diagnostics;
- missing declared resource diagnostics;
- registered payload validation and runtime signatures;
- registered payload prepared-frame inspection;
- compatibility adapter output in `PreparedFrameContributions`;
- one migrated low-risk feature path with equivalent inspection;
- render submit consuming only `PreparedRenderFrame`.

Expected command families:

```text
cargo test -p engine render_feature_contributions
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Non-Goals

PM-003 must not:

- implement render fragments, hot reload, fragment assets, fragment merge, or
  last-good fragment promotion;
- mature the render execution graph compiler beyond collector capability
  declarations;
- migrate every existing render feature contribution path in one slice;
- remove existing central enum variants before compatibility is proven;
- move product truth, product freshness, fallback legality, authority, rebuild
  policy, product dependencies, or residency policy into render collectors;
- add renderer-private handles to app or domain producers;
- perform backend allocation or command encoding in contribution collectors;
- add native multi-window, product-surface hardening beyond contribution
  collection, material graph lowering, asset cooking, plugin ABI, or production
  readiness work.

## Acceptance Bar

This design is accepted when:

- feature-owned collectors have a typed registry, diagnostics, validation, and
  inspection contract;
- the migration keeps old and new contribution paths side by side;
- new features can register a collector without adding feature-specific central
  payload variants;
- the submit boundary remains prepared-frame-only;
- product truth and product policy remain outside renderer helpers and
  collectors;
- PM-004 through PM-008 remain explicitly out of scope.
