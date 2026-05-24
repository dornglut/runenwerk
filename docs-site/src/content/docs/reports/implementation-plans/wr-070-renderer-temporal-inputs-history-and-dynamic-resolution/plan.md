---
title: WR-070 Renderer Temporal Inputs History And Dynamic Resolution Implementation Contract
description: Design-first contract for renderer temporal inputs, history validity, jitter, diagnostics, and dynamic internal resolution.
status: active
owner: engine
layer: engine-runtime / renderer temporal
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-070 Renderer Temporal Inputs History And Dynamic Resolution Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-TEMPORAL-002` and `WR-070`.
This row must introduce renderer-owned temporal input inspection, history
validity, jitter evidence, and dynamic internal resolution diagnostics without
moving camera, product, scene, exposure, material, SDF, ray-query, or fallback
authority into the renderer.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-070` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md`:
  accepted doctrine for portable temporal reconstruction, dynamic resolution,
  history validity, input availability, optional adapters, and fallback
  diagnostics.
- `docs-site/src/content/docs/reports/closeouts/pm-render-temporal-001-temporal-reconstruction-doctrine/closeout.md`:
  completed temporal doctrine acceptance evidence.
- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  scale doctrine that separates internal renderer working sets, dynamic
  targets, budgets, and product truth.
- `docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md`:
  completed working-set and residency budget evidence that WR-070 may consume
  for dynamic target and history-resource diagnostics.
- `engine/src/plugins/render`: owning renderer platform boundary for prepared
  temporal inputs, inspection DTOs, history signatures, and diagnostics.

## Readiness

Initial `task production:plan -- --milestone PM-RENDER-TEMPORAL-002 --roadmap WR-070`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/plan.md`.

This contract clears the design-first gap by making the accepted doctrine,
completed PM-001 closeout, completed WR-061 scale residency evidence, write
scopes, validation, stop conditions, and closeout shape explicit before
implementation.

After applying the intake proposal and repairing the exact promotion metadata
blocker, `task production:plan -- --milestone PM-RENDER-TEMPORAL-002 --roadmap
WR-070` reports:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependency: `WR-061:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-070 --state current_candidate --evidence "Accepted renderer temporal reconstruction doctrine, completed PM-RENDER-TEMPORAL-001 doctrine closeout, completed WR-061 scale residency closeout, and active WR-070 temporal inputs/history/dynamic-resolution contract."
```

WR-070 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-TEMPORAL-002`.

## Governance Decisions

- DDD bounded context owner: `engine/src/plugins/render`.
- Renderer-owned vocabulary: temporal input availability, jitter phase, history
  signature, history invalidation, reconstruction mode, internal resolution,
  output resolution, native fallback, and temporal diagnostics.
- Source-owner vocabulary: camera projection, scene/product generation,
  exposure/luminance meaning, material reactivity semantics, SDF/ray-query
  query policy, product freshness, fallback legality, and authority class.
- Translation boundary: WR-070 may consume prepared producer inputs and expose
  renderer-derived execution evidence. It must not assign durable camera,
  product, material, exposure, SDF, or ray-query truth.
- ADR requirement: no ADR is required for adding renderer-owned inspection and
  history diagnostics. Stop for ADR if implementation persists a new
  cross-domain temporal ABI, moves camera/product/exposure/fallback authority
  into renderer code, or makes vendor upscalers baseline behavior.
- Team Topologies ownership: complicated-subsystem renderer platform consuming
  stream-aligned camera, scene, product, material, SDF, ray-query, and exposure
  producers.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-temporal-inputs-history-and-dyn
docs-site/src/content/docs/reports/implementation-plans/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/plan.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/inspect`: temporal input, history validity,
  dynamic-resolution, fallback, and diagnostic DTOs.
- `engine/src/plugins/render/renderer` and render-flow/preflight boundaries:
  prepared temporal input checks, history signature construction, and dynamic
  resolution invariants.
- `engine/tests`: fail-closed tests for missing inputs, history mismatch,
  dynamic resolution drift, and fallback diagnostics.
- `docs-site/src/content/docs/engine/reference/plugins/render`: public API and
  usage documentation for temporal inspection and dynamic-resolution behavior.

## Non-Goals

- Do not implement FSR, DLSS, XeSS, frame generation, or vendor-specific adapter
  calls in WR-070. Adapter hooks and ray reconstruction inputs remain WR-071.
- Do not create a durable cross-domain temporal ABI.
- Do not make renderer state the source of camera, product, exposure, material,
  SDF, ray-query, freshness, fallback, or authority truth.
- Do not claim `runtime_proven`; production evidence remains WR-072.
- Do not add broad app/editor UI or product workflows.

## Required Implementation Shape

WR-070 must provide typed renderer evidence for:

1. Internal render resolution and output resolution as separate values.
2. Jitter sequence identity, jitter phase, and per-frame jitter offset.
3. Motion-vector, depth, exposure/luminance, and reactive-mask style input
   availability.
4. History resource identity, history signature, history age, and invalidation
   reason.
5. Reconstruction mode and native fallback state.
6. Dynamic-resolution scale limits and diagnostics.
7. Typed errors/warnings for missing inputs, invalid history, resolution
   mismatch, producer-generation mismatch, and unsupported temporal mode.

The implementation should favor small renderer-owned DTOs and focused
inspection functions over runtime-global mutable state. It should reuse existing
render-flow, prepared-frame, dynamic-target, timing, and inspection patterns
where they already solve the boundary.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_temporal
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow_fragments
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If the implementation changes an existing render-flow or prepared-frame
contract, add the smallest relevant existing test filter named by the changed
module before closeout.

## Stop Conditions

Stop before product code if:

- `WR-070` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-TEMPORAL-002 --roadmap WR-070`
  still reports `design_first`, a promotion blocker, or a metadata blocker;
- any implementation requires moving source truth or fallback authority into
  renderer code;
- a vendor adapter is needed to make the portable path work;
- the required validation set cannot run or fails;
- closeout evidence cannot honestly state the remaining quality gaps.

## Closeout Requirements

The closeout must include:

- exact changed modules and functions;
- architecture evidence showing renderer-owned diagnostics only;
- validation commands and results;
- public API/doc updates;
- roadmap and production metadata updates;
- completion quality `bounded_contract`;
- known gaps that leave adapter hooks/ray inputs to WR-071 and runtime
  production evidence to WR-072.

WR-070 completion cannot claim `runtime_proven` or `perfectionist_verified`.
