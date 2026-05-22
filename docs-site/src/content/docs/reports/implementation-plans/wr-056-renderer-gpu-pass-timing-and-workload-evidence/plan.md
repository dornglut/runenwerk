---
title: WR-056 Renderer GPU Pass Timing And Workload Evidence Implementation Contract
description: Implementation contract for capability-gated renderer GPU pass timing and workload evidence.
status: completed
owner: engine
layer: engine-runtime / render diagnostics
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-056 Renderer GPU Pass Timing And Workload Evidence Implementation Contract

## Goal

Define the bounded implementation contract for `PM-RENDER-GPU-002` and
`WR-056`. The slice adds renderer-owned GPU pass timing evidence that separates
CPU encode/submit timing from GPU execution timing and reports explicit
unsupported diagnostics when backend timestamp queries are unavailable.

This contract authorizes implementation only after the stack or single-track
coordinator selects `execute_next_wr_implementation_contract` for `WR-056`.
Implementation remains limited to the write scopes below and must stop after
one bounded implementation slice before validation, closeout, and coordinator
rerun.

## Source Of Truth

- Production milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml::PM-RENDER-GPU-002`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml::WR-056`.
- Accepted doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md`.
- Accepted readiness contract:
  `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`.
- Doctrine closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-gpu-001-gpu-evidence-and-procedural-visuals-doctrine/closeout.md`.

`task production:plan -- --milestone PM-RENDER-GPU-002 --roadmap WR-056`
reported after roadmap promotion:

- milestone state: `active`;
- WR state: `current_candidate`;
- blocker: `B2`;
- dependency: `WR-055:completed`;
- next action: `write_implementation_contract`.

## Readiness

The accepted doctrine, `WR-055` dependency, architecture-governance kickoff,
and roadmap promotion now exist. `WR-056` is the current candidate for
`PM-RENDER-GPU-002`.

Architecture governance kickoff was run for the bounded scope:

```text
task ai:architecture-governance -- --task "Plan renderer GPU pass timing and workload evidence implementation for WR-056" --scope "engine/src/plugins/render engine/tests docs-site/src/content/docs/engine/reference/plugins/render docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md"
```

Governance conclusion for this contract:

- bounded context owner: `engine/src/plugins/render`;
- no ADR is required while timing remains renderer execution evidence and does
  not define product authority, product fallback, or cross-domain source truth;
- an ADR is required if timing evidence becomes a durable cross-domain ABI,
  changes backend/runtime ownership, or becomes authoritative for product
  policy.

## Promotion History

`task production:plan -- --milestone PM-RENDER-GPU-002 --roadmap WR-056`
reported after the row moved to `ready_next`:

- milestone state: `active`;
- WR state: `ready_next`;
- blocker: `B2`;
- dependency: `WR-055:completed`;
- promotion preflight: `promotable`.

The row was promoted with evidence naming this contract, the accepted doctrine,
the completed `PM-RENDER-GPU-001` closeout, and architecture-governance kickoff:

```text
task roadmap:promote -- --id WR-056 --state current_candidate --evidence "Accepted renderer GPU evidence doctrine, completed PM-RENDER-GPU-001 closeout, architecture-governance kickoff, and active WR-056 promotion/readiness contract at docs-site/src/content/docs/reports/implementation-plans/wr-056-renderer-gpu-pass-timing-and-workload-evidence/plan.md"
```

If future roadmap commands fail, repair only the exact metadata named by the
failed command, run `task roadmap:switch-current` if instructed, or stop and
report. Do not inspect adjacent WR rows.

## Ownership And Boundaries

Renderer-owned:

- backend capability detection for timestamp-query support;
- timestamp query allocation, resolve, readback state, stale/readback-pending
  diagnostics, and pass timing projection;
- CPU timing already exposed through `RendererFrameTimings`,
  `GfxFrameTimings`, `RenderDebugTimingsState`, readiness budgets, and public
  render inspection docs;
- backend-neutral DTOs for pass timing, timing source, capability status, frame,
  surface, flow, pass, pass kind, and diagnostics.

Not renderer-owned:

- product truth, product selection, freshness, authority, fallback legality,
  rebuild policy, residency policy, material truth, model truth, gameplay VFX
  semantics, or product scheduling priority.

Backend handles and mutable query pools stay private to backend/runtime modules.
Inspection exposes DTOs and diagnostics only.

## Critical Review Decisions

Source truth:

- WGPU backend capabilities and renderer command execution are the only source
  truth for GPU timestamp availability and pass timing evidence.
- `RendererFrameTimings`, `GfxFrameTimings`, `PassTimingSample`,
  `RenderDebugTimingsState`, readiness reports, docs, and tests are projections
  of renderer execution evidence. They must not become product authority.
- Product requests, selections, freshness, fallback, residency, material truth,
  model truth, gameplay effects, and visual-product existence stay in product
  producers and their owning domains.

Source-to-runtime chain:

1. `engine/src/plugins/render/backend/wgpu_ctx.rs` detects timestamp-query
   support and owns backend resources.
2. `engine/src/plugins/render/renderer/render_flow` and
   `engine/src/plugins/render/runtime/frame_submit.rs` collect CPU timing and,
   where supported, schedule GPU timestamp query resolve/readback around
   renderer-owned pass execution.
3. `engine/src/plugins/render/inspect/timings.rs` projects the execution data
   into backend-neutral DTOs with explicit capability, source, and diagnostic
   states.
4. `engine/src/plugins/render/inspect/readiness.rs` and
   `engine/src/plugins/render/inspect/budgets.rs` consume those DTOs as
   renderer execution evidence only.
5. `engine/tests/render_runtime_inspect.rs`,
   `engine/tests/render_gpu_timing.rs`, and render reference docs prove the DTO
   chain, unsupported behavior, and absence of backend handle leaks.

Typed contracts replace strings or ad hoc formatting for:

- GPU timing capability state;
- timing source (`cpu_encode_submit` versus `gpu_timestamp_query`);
- unsupported, unavailable-this-frame, and readback-pending diagnostics;
- per-pass timing identity by frame, surface, flow, pass, and pass kind.

Forbidden fallbacks:

- Do not report CPU encode/submit timing as GPU pass timing.
- Do not treat missing timestamp support as success without a typed diagnostic.
- Do not expose WGPU query sets, buffers, command encoders, or mutable backend
  handles through public inspection DTOs.
- Do not let readiness budgets decide product fallback or product rebuild
  policy.

Architecture guard tests must prove:

- unsupported timestamp capability produces typed diagnostics;
- readback-pending or unavailable data remains explicit;
- GPU timing DTOs keep CPU timing separate;
- runtime inspection and readiness budget surfaces consume DTOs without backend
  handles or product-policy decisions.

## Implementation Scope

Allowed write scopes after roadmap readiness:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-gpu-pass-timing-and-workload-evidence
docs-site/src/content/docs/reports/implementation-plans/wr-056-renderer-gpu-pass-timing-and-workload-evidence/plan.md
docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Expected owning modules:

- `engine/src/plugins/render/backend/wgpu_ctx.rs` for WGPU timestamp-query
  capability detection and backend resource ownership;
- `engine/src/plugins/render/runtime/frame_submit.rs` for submit-adjacent
  timing collection, resolve scheduling, and fail-closed command submission
  integration;
- `engine/src/plugins/render/inspect/timings.rs` for backend-neutral timing
  DTOs, capability states, unsupported diagnostics, and readback-pending
  summaries;
- `engine/src/plugins/render/inspect/readiness.rs` and
  `engine/src/plugins/render/inspect/budgets.rs` for readiness and budget
  consumption of GPU timing evidence;
- `engine/src/plugins/render/inspect/report.rs` and
  `engine/src/plugins/render/inspect/mod.rs` for public inspection exports;
- `engine/tests/render_runtime_inspect.rs` for runtime inspection and
  unsupported diagnostics coverage;
- a focused `engine/tests/render_gpu_timing.rs` test target if the
  implementation needs timing-specific coverage outside existing inspection
  tests.

Use nearby module boundaries if code inspection shows a tighter owner, but keep
timing diagnostics in render inspection and backend handles in backend/runtime.
Do not add catch-all helper files.

## Required Contracts

The implementation must introduce or refine typed contracts for:

- `RenderGpuTimingCapability` or equivalent capability status:
  `supported`, `unsupported`, `unavailable_this_frame`, and
  `readback_pending`;
- per-pass GPU timing DTOs keyed by surface, frame, flow, pass, pass kind, and
  timing source;
- unsupported diagnostics naming the missing backend capability;
- readback latency behavior that makes pending data explicit instead of
  silently dropping pass timing;
- readiness/budget integration that diagnoses renderer execution evidence only
  and does not decide product policy.

## First Bounded Implementation Slice

The first code slice after this contract should implement the backend-neutral
inspection contract before adding backend query resources:

1. Add typed GPU timing capability/source/diagnostic DTOs in
   `engine/src/plugins/render/inspect/timings.rs`.
2. Extend `RenderDebugTimingsState` and summary helpers so CPU pass samples,
   GPU pass samples, unsupported diagnostics, unavailable data, and
   readback-pending data can be reported separately.
3. Add deterministic tests in `engine/tests/render_runtime_inspect.rs` or
   `engine/tests/render_gpu_timing.rs` for DTO projection, unsupported
   diagnostics, readback-pending state, and CPU/GPU timing separation.
4. Update render inspection reference docs if public inspection names or normal
   usage text changes.

This first slice may use deterministic host-side samples. It must not claim
runtime GPU timestamp execution until backend query support, resolve/readback,
and runtime evidence are implemented and validated in a later bounded slice.

## Implementation Steps

1. Inspect current `RendererFrameTimings`, `GfxFrameTimings`,
   `RenderDebugTimingsState`, render readiness, budget, and submit paths.
2. Add backend-neutral GPU timing DTOs and unsupported diagnostic kinds under
   render inspection.
3. Detect timestamp-query support in the WGPU backend without exposing WGPU
   handles through public DTOs.
4. Allocate and resolve timestamp queries around renderer-owned flow/pass
   execution where supported.
5. Preserve CPU timing fields and add GPU pass timing as separate evidence.
6. Surface unsupported, unavailable, and readback-pending states through
   runtime inspection and readiness budgets.
7. Add focused tests for supported-shape DTO projection where deterministic
   host support is available and for unsupported/readback-pending diagnostics in
   all environments.
8. Update public render reference docs with normal usage, unsupported states,
   and the product-policy non-goal.
9. Create closeout evidence only after implementation validation passes.

## Non-Goals

Do not implement:

- render-flow pass-shape guards from `WR-057`;
- procedural mesh/SDF instance APIs from `WR-058`;
- boids rendering rewrite from `WR-059`;
- production benchmark and evidence hardening from `WR-060`;
- product fallback, freshness, authority, rebuild, residency, material, model,
  gameplay, or VFX policy;
- public backend handles or mutable timestamp query pools;
- a persisted timing ABI for non-render domains without an ADR.

## Acceptance Criteria

- Public inspection separates CPU preflight/encode/submit timing from GPU pass
  timing.
- Unsupported timestamp-query capability is visible as typed diagnostics.
- Readback-pending and unavailable-this-frame states are explicit.
- Readiness/budget reports can consume GPU timing evidence without deciding
  product policy.
- Existing CPU timing tests and inspection docs remain valid.
- No renderer-private WGPU handles leak into app, domain, docs, or examples.

## Validation

Contract-only validation:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-RENDER-GPU
task ai:goal -- --track PT-RENDER-PERFECTION --stack
```

Focused validation:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_gpu_timing
task docs:validate
```

Workflow validation after implementation and metadata changes:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-GPU
```

## Stop Conditions

- Stop if timestamp timing requires product-domain ownership or product-policy
  decisions.
- Stop if an accepted ADR becomes required for backend ownership, public ABI, or
  cross-domain evidence authority.
- Stop if unsupported timestamp-query states cannot be diagnosed explicitly.
- Stop if implementation would hide missing GPU timing behind CPU-only success.
- Stop if roadmap promotion, validation, or closeout gates fail.

## Bounded Slice Closeout Log

### 2026-05-22 - Backend-Neutral GPU Timing Inspection DTOs

Status: completed as a bounded implementation slice, not as full `WR-056`
completion.

Changed modules:

- `engine/src/plugins/render/inspect/timings.rs`:
  `RenderTimingSource`, `RenderGpuTimingCapability`,
  `RenderGpuTimingDiagnostic`, `RenderPassTimingEvidence`,
  `summarize_gpu_pass_timing_evidence`, and `RenderDebugTimingsState` GPU
  timing projection fields.
- `engine/src/plugins/render/inspect/budgets.rs`:
  `RenderReadinessBudgetKind::GpuPassTotalMillis`,
  `RenderReadinessBudgetKind::GpuTimingDiagnosticCount`, and
  `RenderReadinessBudgetMeasurements` GPU timing measurements.
- `engine/src/plugins/render/inspect/readiness.rs`:
  `RenderReadinessDiagnosticKind::GpuTimingDiagnostics`,
  `RenderReadinessReportRequest::timings`, and readiness source summaries for
  GPU pass samples and GPU timing diagnostics.
- `engine/tests/render_runtime_inspect.rs` and
  `engine/tests/render_gpu_timing.rs`: deterministic coverage for CPU/GPU
  timing source separation, supported GPU timing DTOs, unsupported diagnostics,
  readback-pending diagnostics, and readiness budget consumption.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
  and `docs-site/src/content/docs/engine/reference/plugins/render/usage-guide.md`:
  public inspection and usage documentation for the new DTO layer.

Validation:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_gpu_timing
task docs:validate
```

Known remaining `WR-056` work:

- Timestamp query allocation, pass wrapping, resolve, and readback scheduling in
  renderer/runtime submit paths.
- Runtime evidence on a timestamp-capable backend or explicit unsupported
  backend diagnostics wired from actual backend capabilities.
- Full `WR-056` closeout, roadmap completion metadata, and production milestone
  completion evidence.

### 2026-05-22 - Backend Timestamp Capability And Runtime Diagnostic Wiring

Status: completed as a bounded implementation slice, not as full `WR-056`
completion.

Changed modules:

- `engine/src/plugins/render/backend/device.rs`: requests
  `wgpu::Features::TIMESTAMP_QUERY` when the adapter supports it and records
  `RenderBackendTimingCapabilities`.
- `engine/src/plugins/render/backend/wgpu_ctx.rs`: stores backend timing
  capabilities beside the WGPU device and queue.
- `engine/src/plugins/render/renderer/mod.rs`,
  `engine/src/plugins/render/renderer/setup.rs`, and
  `engine/src/plugins/render/renderer/render_flow/execute.rs`: carry backend
  timing capability into render execution and store per-pass GPU timing evidence
  for frame inspection.
- `engine/src/plugins/render/runtime/frame_submit.rs`: observes renderer GPU
  timing evidence into `RenderDebugTimingsState`.
- `engine/tests/render_gpu_timing.rs` and
  `engine/tests/render_runtime_inspect.rs`: cover backend feature-to-capability
  mapping, unsupported diagnostics, unavailable-this-frame diagnostics, and
  readiness budget consumption.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents capability detection and unsupported diagnostics.

Validation:

```text
cargo test -p engine render_gpu_timing
cargo test -p engine render_runtime_inspect
```

Known remaining `WR-056` work:

- Timestamp query allocation and query-set ownership.
- Timestamp writes around compute/render pass execution.
- Resolve/readback staging and readback-pending to measured GPU milliseconds
  transition.
- Runtime evidence on a timestamp-capable backend or explicit local unsupported
  backend evidence in the final closeout.

### 2026-05-22 - Timestamp Query Resolve And Measured GPU Pass Evidence

Status: completed as a bounded implementation slice, pending final runtime
closeout evidence.

Changed modules:

- `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs`: owns
  renderer-private query sets, resolve/readback buffers, timestamp readback,
  measured GPU milliseconds, and unavailable diagnostics for readback failures.
- `engine/src/plugins/render/renderer/render_flow/mod.rs`: exposes the GPU
  timing runtime only inside the renderer module boundary.
- `engine/src/plugins/render/renderer/render_flow/execute.rs`: allocates
  per-frame timestamp query capacity, reserves timestamp pairs for supported
  pass types, resolves queries before submit, and replaces pending evidence with
  measured GPU pass timing after readback.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs` and
  `engine/src/plugins/render/renderer/setup.rs`: attach WGPU timestamp writes to
  compute, fullscreen, graphics, and builtin UI render passes.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents pass timestamp writes, resolve/readback, and unavailable
  diagnostics.

Validation:

```text
cargo test -p engine render_gpu_timing
cargo test -p engine render_runtime_inspect
```

Known remaining `WR-056` work:

- Local runtime evidence must be captured on this machine. If the backend lacks
  timestamp support, the closeout must record explicit unsupported diagnostics.
  If timestamp support is available, the closeout must record measured GPU pass
  timing from runtime inspection.
- Final `WR-056` closeout, roadmap completion metadata, and
  `PM-RENDER-GPU-002` production evidence remain open until runtime evidence is
  recorded.

## Closeout Requirements

Closeout path:

```text
docs-site/src/content/docs/reports/closeouts/wr-056-renderer-gpu-pass-timing-and-workload-evidence/closeout.md
```

The closeout must record:

- implementation files and modules changed;
- supported, unsupported, unavailable, and readback-pending evidence;
- validation command results;
- docs updates;
- roadmap metadata updates, including moving `WR-056` to completed only after
  implementation validation and closeout evidence exist;
- production metadata updates, including `PM-RENDER-GPU-002` completion only
  after `WR-056` is completed with matching evidence gates;
- known quality gaps, including any environment where GPU timestamp support
  could not be observed locally.

## Perfectionist Closeout Audit

Expected completion quality: `runtime_proven` only if runtime inspection shows
GPU pass timing on a timestamp-capable backend or explicit unsupported
diagnostics on the local backend, with tests proving both projection and
unsupported behavior.

Use `bounded_contract` instead if the implementation only proves DTO shape or
unsupported diagnostics without runtime execution evidence. Do not claim
`perfectionist_verified` for this row; the final renderer audit track owns that
claim.
