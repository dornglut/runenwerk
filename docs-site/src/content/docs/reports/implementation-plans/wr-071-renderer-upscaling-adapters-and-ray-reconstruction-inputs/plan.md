---
title: WR-071 Renderer Upscaling Adapters And Ray Reconstruction Inputs Implementation Contract
description: Design-first contract for optional upscaling adapter hooks, ray reconstruction inputs, capability diagnostics, and native fallback.
status: active
owner: engine
layer: engine-runtime / renderer temporal
canonical: false
last_reviewed: 2026-05-23
related_designs:
  - ../../../design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-071 Renderer Upscaling Adapters And Ray Reconstruction Inputs Implementation Contract

## Goal

Prepare the implementation slice for `PM-RENDER-TEMPORAL-003` and `WR-071`.
This row must add renderer-owned inspection contracts for optional
FSR-style upscaling adapter hooks and ray reconstruction inputs without making
any vendor SDK, hardware ray-query path, or adapter result required for
correct rendering.

This is a design-first and promotion-readiness contract. Product code changes
are authorized only after `WR-071` is applied to the active roadmap, promoted by
the roadmap workflow, selected by the stack coordinator, and rechecked by
`task production:plan`.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-temporal-reconstruction-and-dynamic-resolution-design.md`:
  accepted doctrine for portable temporal reconstruction, dynamic resolution,
  optional upscaling adapters, ray reconstruction inputs, capability
  diagnostics, and native fallback.
- `docs-site/src/content/docs/reports/closeouts/wr-070-renderer-temporal-inputs-history-and-dynamic-resolution/closeout.md`:
  completed temporal input, history, jitter, and dynamic-resolution evidence
  that WR-071 extends.
- `docs-site/src/content/docs/reports/closeouts/pm-render-temporal-001-temporal-reconstruction-doctrine/closeout.md`:
  completed temporal doctrine acceptance evidence.
- `engine/src/plugins/render`: owning renderer platform boundary for temporal
  diagnostics, adapter capability evidence, ray reconstruction input evidence,
  and fallback decisions.

## Readiness

`task production:plan -- --milestone PM-RENDER-TEMPORAL-003 --roadmap WR-071`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-071-renderer-upscaling-adapters-and-ray-reconstruction-inputs/plan.md`.

This contract clears the design-first gap by making the accepted doctrine,
completed WR-070 dependency, write scopes, validation, stop conditions, and
closeout shape explicit before implementation. It also records the architecture
governance decision that no ADR is required for renderer-owned capability and
diagnostic DTOs, while an ADR is required if implementation introduces a
durable cross-domain adapter ABI, new backend dependency direction, or vendor
adapter as baseline behavior.

After applying the intake proposal, WR-071 may be promoted only when:

- `WR-070` remains completed with valid closeout evidence;
- `PM-RENDER-TEMPORAL-003` is active and still selected by the stack
  coordinator;
- `task production:plan -- --milestone PM-RENDER-TEMPORAL-003 --roadmap WR-071`
  reports a promotable or promotion-contract action rather than `design_first`;
- roadmap, production, docs, and planning validators pass.

The promotion preflight now reports:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- roadmap blocker: `B2`;
- roadmap dependency: `WR-070:completed`;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`;
- suggested command:

```text
task roadmap:promote -- --id WR-071 --state current_candidate --evidence "Accepted renderer temporal reconstruction doctrine, completed PM-RENDER-TEMPORAL-001 doctrine closeout, completed WR-070 temporal inputs/history/dynamic-resolution closeout, and active WR-071 upscaling adapter/ray reconstruction input contract."
```

WR-071 may be promoted only when this readiness evidence remains true and the
stack coordinator still selects `PM-RENDER-TEMPORAL-003`.

## Governance Decisions

- DDD bounded context owner: `engine/src/plugins/render`.
- Renderer-owned vocabulary: adapter kind, adapter capability state, adapter
  invocation eligibility, unsupported reason, ray reconstruction input
  availability, ray reconstruction history compatibility, native fallback, and
  temporal upscaling diagnostics.
- Source-owner vocabulary: camera projection, scene/product generation, SDF
  query policy, hardware ray-query resource authority, material reactivity
  semantics, exposure meaning, and product freshness.
- Translation boundary: WR-071 may consume prepared temporal evidence from
  WR-070 and prepared raymarch/ray-query input descriptors. It must report
  whether those inputs are sufficient for an optional adapter path. It must not
  become the source of scene, SDF, ray-query, material, exposure, product, or
  camera truth.
- Clean Architecture direction: the engine renderer may depend on established
  engine, domain, and foundation contracts. It must not introduce app/editor
  concepts, tool-only state, or a mandatory vendor SDK dependency into the
  portable renderer path.
- ADR requirement: no ADR is required for typed renderer inspection DTOs and
  fail-closed diagnostics. Stop for ADR if implementation defines a durable
  cross-domain adapter ABI, adds a mandatory external upscaler dependency,
  changes ray-query ownership, or makes a vendor adapter required for baseline
  rendering.
- Team Topologies ownership: complicated-subsystem renderer platform consuming
  stream-aligned product, SDF, ray-query, material, exposure, and camera
  producer evidence.

## Implementation Scope After Promotion

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-upscaling-adapters-and-ray-reco
docs-site/src/content/docs/reports/implementation-plans/wr-071-renderer-upscaling-adapters-and-ray-reconstruction-inputs/plan.md
docs-site/src/content/docs/reports/closeouts/wr-071-renderer-upscaling-adapters-and-ray-reconstruction-inputs/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/inspect/temporal.rs`: keep WR-070 input, history,
  jitter, and dynamic-resolution DTOs as the portable temporal baseline.
- `engine/src/plugins/render/inspect/temporal_upscaling.rs`: add optional
  adapter capability, invocation eligibility, ray reconstruction input, and
  fail-closed diagnostic DTOs. Use this adjacent module instead of growing
  `temporal.rs` into a mixed responsibility file.
- `engine/src/plugins/render/inspect/mod.rs`: export the WR-071 inspection API.
- `engine/tests/render_temporal_upscaling.rs`: guard adapter unsupported,
  missing ray input, history mismatch, and visible native fallback behavior.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  document the public inspection surface and fallback semantics.

## Non-Goals

- Do not implement FSR, DLSS, XeSS, frame generation, or vendor SDK calls.
- Do not make hardware ray-query support a baseline requirement.
- Do not allocate or execute adapter GPU passes unless the existing renderer
  runtime already exposes the required portable path in the selected WR scope.
- Do not move SDF, ray-query, camera, exposure, material, product, or fallback
  authority into renderer inspection code.
- Do not claim `runtime_proven`; production evidence remains WR-072.
- Do not add app/editor UI, asset pipeline changes, or broad runtime rewrites.

## Required Implementation Shape

WR-071 must provide typed renderer evidence for:

1. Adapter kind, support state, required capability flags, and unsupported
   reason.
2. Adapter invocation eligibility derived from WR-070 temporal evidence,
   dynamic internal/output resolution, history validity, and prepared input
   availability.
3. Ray reconstruction input availability for depth, motion vectors,
   disocclusion/reactive masks, raymarch depth or distance evidence,
   ray-query hit or distance evidence, exposure, and history compatibility.
4. Native fallback state that remains visible when an adapter is unsupported,
   missing inputs, or rejected by history/capability mismatch.
5. Typed diagnostics for missing ray inputs, unsupported adapter capability,
   stale history signature, invalid dynamic resolution, and forbidden
   adapter-required rendering.

The implementation should favor small renderer-owned DTOs and a focused
inspection function, likely `inspect_render_temporal_upscaling(...)`, over
runtime-global mutable state. It should reuse WR-070 temporal evidence types
instead of duplicating input/history validation.

## Validation

Focused validation expected for implementation:

```text
cargo fmt
cargo test -p engine render_temporal
cargo test -p engine render_temporal_upscaling
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

If implementation touches render-flow runtime execution, dynamic target
selection, or backend capability reporting, add the smallest focused existing
test for that changed subsystem before closeout.

## Stop Conditions

Stop before product code if:

- `WR-071` is not applied, promoted, and selected by the coordinator;
- `task production:plan -- --milestone PM-RENDER-TEMPORAL-003 --roadmap WR-071`
  still reports `design_first`, a promotion blocker, or a metadata blocker;
- any implementation requires a vendor SDK to prove the portable path;
- adapter support would become required for correctness;
- ray-query, SDF, camera, exposure, material, product, or fallback authority
  would move into renderer inspection code;
- required validation cannot run or fails;
- closeout evidence cannot honestly state the remaining WR-072 production
  evidence gap.

## Closeout Requirements

The closeout must include:

- exact changed modules and functions;
- architecture evidence showing renderer-owned diagnostics and preserved
  producer ownership;
- validation commands and results;
- public API/doc updates;
- roadmap and production metadata updates;
- completion quality `bounded_contract`;
- known gaps that leave examples, benchmark/report artifacts, hardware
  profiles, and runtime production evidence to WR-072.

WR-071 completion cannot claim `runtime_proven` or
`perfectionist_verified`.
