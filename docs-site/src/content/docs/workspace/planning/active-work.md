---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-26
related_docs:
  - ../engineering-workflow.md
  - ./roadmap.md
  - ./completed-work.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page is only a concise cross-project summary.

## Active

### RunenGPU G2 implementation

RunenGPU S0 and G1A are complete. Issue `#168` and PR `#171` bind the G2 investigation and decision-complete specification. Issue `#172` is the single bounded implementation slice and remains blocked until the planning PR merges.

The implementation must:

- create future-transferable capability, logical-resource, kind-typed-handle, prepared-data, provenance, and structured-error authority under `engine::plugins::gpu`;
- model resource kind, lifetime, ownership, transfer/observation, reconstruction, and memory intent independently;
- distinguish buffer initialization from texture initialization, including checked texture format, extent, `bytes_per_row`, and `rows_per_image`;
- keep texture-view validity bounded by the parent texture lease and checked subresources;
- keep ECS/domain projection, render target/history/surface meaning, shader-file policy, fixed-time scheduling, UI, capture, artifacts, and product recovery outside RunenGPU;
- migrate every inventoried declaration and consumer of the authority G2 replaces;
- delete the old capability profile, combined lifetime/import, generic descriptor, and render-owned generic handle authority without aliases or forwarding paths;
- preserve the common automatic-validation path and the inspectable advanced path as one future authority;
- add focused unit, rustdoc compile-pass, rustdoc compile-fail, source-guard, and dependency-guard evidence;
- stop rather than widening into G3 hazards/work graphs, G4 backend realization, G5 execution/readback, G6 graphics, G7 surfaces, or external extraction.

The target external repository remains `dornglut/runen-gpu`, but no external package is created during G2.

The proof portfolio is already bound and remains separated:

- G5 deterministic conformance: exact inclusive/exclusive 4,097-element `u32` prefix scan;
- G5 stateful integration: headless fixed-seed Game of Life with full-grid CPU oracle, live-cell count, checksum, and selected-cell assertions;
- G6 graphics conformance: offscreen known-pattern draw;
- G6 GPU-driven composition: compute-generated indirect draw;
- G6 representative showcase: offscreen boids with structural and bounded invariants;
- G7 surface proof: reuse accepted G6 workloads;
- first RunenRender semantic proof: procedural sky/SDF terrain.

## Queued

- G3 access, initialization flow, hazards, immutable generic work, inferred dependencies, and internal graph after G2 closeout;
- G4 context/device admission, WGPU realization, shaders, pipelines, binding keys, backend layout, macro disposition, and removal of the temporary `RenderFlowId` bridge;
- G5 execution, uploads, staging, completion, asynchronous readback, cancellation, and delayed retirement;
- G6 offscreen graphics and shared render/non-render proof;
- G7 surfaces, generations, thread affinity, and device outcomes;
- G8 final diagnostics, shutdown, residual anti-cheating audit, and internal conformance;
- external RunenGPU clean cutover only after G2-G8 and standalone conformance;
- internal then external RunenRender work on the accepted RunenGPU boundary;
- RunenECS boundary repair as a separately scheduled, non-conflicting workstream.

## Completed foundation

- workflow execution platform retirement: issues/PRs `#122`, `#123`, and `#124`;
- final repository-surface pruning: issue `#135`, PR `#136`;
- Rust 1.97 and documentation baseline recovery: issues `#150` and `#154`, PR `#155`;
- shared Rust validation adoption: issue `#137`, PR `#138`;
- root architecture foundation alignment: PR `#141`;
- GPU/render architecture correction: issue `#125`, PR `#126`;
- GPU/render S0 inventory: issue `#127`, PR `#128`;
- original G1A implementation specification: issue `#129`, PR `#130`;
- corrected G1A owner-scoped identity and fallible-authoring authority;
- G1A implementation: issue `#131`, PR `#164`, merge `5bbdab36ae661d99432bfe5d215062c397aac975`, and [closeout report](../../reports/closeouts/pt-runengpu-g1a-closeout.md);
- G2 public API/industry/proof-workload supporting decisions: PRs `#169` and `#170`;
- G2 decision phase: issue `#168`, PR `#171`, the [current-main investigation](../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md), and the [implementation specification](../specs/pt-runengpu-g2-capabilities-resource-descriptors.ron);
- RunenSDF standalone transfer, maintained authority, and Runenwerk duplicate-source retirement: Runenwerk PRs `#118` and `#157`, issue `#133`, and `dornglut/runen-sdf` PRs `#1`, `#2`, `#4`, `#5`, and `#6`.
