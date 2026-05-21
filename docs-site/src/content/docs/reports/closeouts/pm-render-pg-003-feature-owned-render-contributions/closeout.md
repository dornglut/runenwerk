---
title: PM-RENDER-PG-003 Feature-Owned Render Contributions Closeout
description: Closeout evidence for the bounded feature-owned render contribution collector implementation slice.
status: completed
owner: engine
layer: engine-runtime / render prepared-frame contracts
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/feature-owned-render-contributions-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-contract-ergonomics-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# PM-RENDER-PG-003 Feature-Owned Render Contributions Closeout

## Result

`PM-RENDER-PG-003` completed as a bounded feature-owned render contribution
collector slice.

The implementation adds a typed contribution collector registry and a
registered payload compatibility bridge so new render features can contribute
prepared-frame payloads without adding new feature-specific central enum
variants. Collectors run during `RenderPrepare`, declare the prepared resources
they read, report typed diagnostics, and feed submit through the existing
validated prepared-frame boundary.

The slice migrates one low-risk existing path: the scene-route contribution now
flows through the registered collector path while legacy central contribution
variants remain available for later migrations. The implementation does not add
render fragments, hot reload, render execution graph compiler maturity,
material lowering, all-feature migration, live submit extraction, or
renderer-owned product policy.

## Implementation Evidence

- `engine/src/plugins/render/frame/contribution_registry.rs` adds
  `RenderFeatureContributionCollectorRegistryResource`,
  `RenderFeatureContributionCollector`, typed collector descriptors, declared
  resource requirements, registered payload kinds, registered payload
  inspection, and the built-in scene-route collector.
- `engine/src/plugins/render/frame/contribution_diagnostics.rs` adds typed
  `PreparedFeatureContributionDiagnostic` values and severity classification.
- `engine/src/plugins/render/frame/contributions.rs` adds contribution
  diagnostics and `PreparedFeaturePayload::Registered(...)` for the registered
  payload bridge.
- `engine/src/plugins/render/runtime/frame_prepare.rs` collects registered
  feature contributions during frame preparation, validates declared resource
  access, rejects feature/payload conflicts with typed diagnostics, and keeps
  submit free of live ECS extraction.
- `engine/src/plugins/render/renderer/prepare.rs` hashes registered payload
  kind and runtime signature so renderer caching remains deterministic.
- `engine/src/plugins/render/inspect/prepared_frame.rs` exposes prepared-frame
  feature contribution inspection, including registered payload summaries and
  fields.
- Render public API docs and the render flow usage guide document the collector
  registry contract, submit boundary, and compatibility adapter.

## Validation

Focused validation passed:

```text
cargo test -p engine render_feature_contributions
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
```

Observed focused-test coverage:

- `cargo test -p engine render_feature_contributions`: 5 passed.
- `cargo test -p engine render_runtime_inspect`: 10 passed.
- `cargo test -p engine render_flow`: 17 passed.

Workflow validation passed before closeout:

```text
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

## Completion Quality

Completion quality is `bounded_contract`.

This is not `runtime_proven`: the slice proves the prepared-frame collector
contract, typed diagnostics, compatibility bridge, and inspection surface, but
it does not claim new product-chain GPU behavior or pixel-proof render output.

This is not `perfectionist_verified`: later production milestones still own
render graph compiler maturity, broad product-surface hardening, multi-surface
presentation, render fragments and hot reload, and final production readiness
inspection.

## Known Gaps

- `PM-RENDER-PG-004` still owns render execution graph compiler maturity.
- `PM-RENDER-PG-005` still owns broad product-surface platform hardening.
- `PM-RENDER-PG-006` still owns multi-surface presentation.
- `PM-RENDER-PG-007` still owns render fragments and hot reload.
- `PM-RENDER-PG-008` still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
- Only the scene-route contribution path migrated in this bounded slice;
  remaining feature contribution migrations require future legal WR slices.
- PM-003 did not claim `runtime_proven` or `perfectionist_verified` evidence.
