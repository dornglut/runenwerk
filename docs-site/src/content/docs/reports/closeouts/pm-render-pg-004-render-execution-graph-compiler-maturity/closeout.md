---
title: PM-RENDER-PG-004 Render Execution Graph Compiler Maturity Closeout
description: Closeout evidence for the bounded render execution graph compiler maturity implementation slice.
status: completed
owner: engine
layer: engine-runtime / render graph compiler
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/feature-owned-render-contributions-design.md
  - ../../../design/accepted/render-contract-ergonomics-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# PM-RENDER-PG-004 Render Execution Graph Compiler Maturity Closeout

## Result

`PM-RENDER-PG-004` completed as a bounded render execution graph compiler
maturity slice.

The implementation adds typed static compiler diagnostics, backend-neutral
capability validation, compiled resource lifetime windows, and prepared-frame
execution preflight before backend command encoding. The renderer still consumes
prepared render selections and compiled render flows only; it does not extract
live ECS product truth during submit and does not own product selection,
freshness, authority, fallback legality, rebuild policy, dependency truth, or
residency policy.

The slice does not implement render fragments, hot reload, broad product-surface
hardening, native multi-window presentation, material lowering, production
readiness budgets, capture/replay policy, or renderer-owned product policy.

## Implementation Evidence

- `engine/src/plugins/render/graph/diagnostics.rs` adds typed
  `RenderExecutionGraphDiagnostic` reports, diagnostic severity/kind values, and
  compiler/preflight error wrappers.
- `engine/src/plugins/render/graph/capabilities.rs` adds
  `RenderBackendCapabilityProfile` and backend-neutral compiled-flow capability
  inspection without exposing WGPU handles.
- `engine/src/plugins/render/graph/resource_lifetimes.rs` derives
  `CompiledResourceLifetimeWindow` values from compiled pass/resource usage.
- `engine/src/plugins/render/graph/prepared_validation.rs` validates prepared
  frames against compiled flows before backend encoding, including flow presence,
  prepared views, duplicate invocations, target aliases, dynamic target
  descriptors, history signatures, feature gates, compute dispatches, and
  uniform payloads.
- `engine/src/plugins/render/graph/planning.rs` exposes
  `compile_flow_plan_checked(...)` while preserving the existing
  `compile_flow_plan(...)` compatibility path.
- `engine/src/plugins/render/renderer/render_flow/execute.rs` runs compiler
  preflight before dynamic target realization and backend command encoding.
- `engine/src/plugins/render/renderer/setup.rs` and
  `engine/src/plugins/render/renderer/mod.rs` expose
  `Renderer::last_preflight_report()` for inspection without backend handle
  leakage.
- `engine/src/plugins/render/inspect/plan.rs` and
  `engine/src/plugins/render/inspect/graph_dump.rs` expose compiled plan,
  resource lifetime, compiler diagnostic, and preflight inspection summaries.
- Render public API docs, the render flow usage guide, and the renderer roadmap
  document the compiler/preflight contract and the product-policy ownership
  boundary.

## Validation

Focused validation passed:

```text
cargo test -p engine render_flow
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
```

Observed focused-test coverage:

- `cargo test -p engine render_flow`: 17 render flow unit tests, 1 submit
  cutoff guard test, and 3 render flow compiler maturity tests passed.
- `cargo test -p engine render_dynamic_targets`: 9 dynamic target and
  prepared-frame preflight tests passed.
- `cargo test -p engine render_runtime_inspect`: 11 runtime inspection tests
  passed.

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

This is not `runtime_proven`: the slice proves compiler/preflight contracts,
typed diagnostics, lifetime/capability inspection, and submit-boundary
preservation, but it does not claim new product-chain GPU behavior or pixel
proof output.

This is not `perfectionist_verified`: later production milestones still own
broad product-surface hardening, multi-surface presentation, render fragments
and hot reload, and final production readiness inspection.

## Known Gaps

- `PM-RENDER-PG-005` still owns broad product-surface platform hardening across
  viewport, material preview, field/debug, drawing, and future preview
  producers.
- `PM-RENDER-PG-006` still owns multi-surface presentation.
- `PM-RENDER-PG-007` still owns render fragments and hot reload.
- `PM-RENDER-PG-008` still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
- PM-004 did not claim `runtime_proven` or `perfectionist_verified` evidence.
