---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-27
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
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page is only a concise cross-project summary.

## Active

### RunenGPU G3 planning

RunenGPU S0, G1A, and the bounded G2 implementation are complete through issue `#172` and PR `#173`. The G2 entry becomes authoritative through the merge of PR `#173`; repository Git history and the closed issue/merged PR represent acceptance without requiring this branch to invent a merge SHA.

The one active serialized next action is to create one decision-complete G3 planning issue and specification for:

- work-time buffer access ranges and texture subresources;
- initialization-flow validation;
- hazards and dependency inference;
- immutable generic work;
- the internal work graph.

G3 implementation is not active or authorized. It requires its own accepted planning issue and specification before source changes. G4-G7 remain deferred to their existing owners, and G2 does not authorize an external package or extraction.

The [G2 implementation closeout](../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md) records the completed authority, exact reviewed implementation head, validation contract, equality audit, explicit import-lowering boundary, and deliberate later-phase seams.

The final independent review corrected two render-adapter semantics: target-alias binding keys are validated render-owned semantic authority carried through prepared/runtime lookup, while transitional render `TypeId` is explicit process-local declared-type compatibility evidence rather than normalized GPU authority. Diagnostic type names and display labels remain non-semantic, and compound render declarations expose no universal equality.

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

- G3 implementation only after one bounded planning issue and specification are decision-complete and accepted;
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
- G2 bounded implementation: issue `#172`, PR `#173`, and the [implementation closeout](../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md). This entry becomes authoritative through the merge of PR `#173`.
- RunenSDF standalone transfer, maintained authority, and Runenwerk duplicate-source retirement: Runenwerk PRs `#118` and `#157`, issue `#133`, and `dornglut/runen-sdf` PRs `#1`, `#2`, `#4`, `#5`, and `#6`.
