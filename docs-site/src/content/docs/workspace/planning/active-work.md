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
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../specs/pt-runengpu-g3-access-work-graph.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page is only a concise cross-project summary.

## Active

### RunenGPU G3 decision phase

RunenGPU S0, G1A, and G2 are accepted. G2 merged through PR `#173` as `709aa6aced020ee99405e1e1c3dde7703c77a4d4`, and issue `#172` is closed.

Issue `#174` is the one active serialized decision slice. Its documentation-only branch is `docs/runengpu-g3-access-work-graph`. It owns:

- checked work-time buffer byte ranges, texture subresources, and query ranges;
- attachment load/store and region-aware initialization flow;
- typed compute, render, copy, clear, texture/query resolve, and logical present operations;
- normalized query-resolve destination buffer usage and exact destination byte coverage;
- read/write overlap, hazards, and access-derived dependency inference;
- typed import/export causality across independent fragments;
- immutable generic work fragments and nodes;
- deterministic `GpuPreparedWorkGraph` preparation and inspection;
- exact render/GPU-primitive/timestamp adapter, migration, and deletion inventories.

The accepted planning direction is:

- lexical node order orients data hazards within one fragment;
- fragment collection position is not semantic scheduling authority;
- cross-fragment causality requires shared typed resources plus typed import/export relationships;
- overlapping cross-fragment writers without one unique producer fail as ambiguous;
- explicit order remains typed, fragment-local, and limited to non-data constraints; redundant data edges fail;
- timestamp writes initialize query indices, typed query resolution consumes them, and a later buffer copy consumes the resolved byte range;
- attachment `Load` requires initialized coverage, `Clear` establishes coverage, `Store` preserves it, and `Discard` removes later readable coverage;
- attachment `Store` alone does not make an empty render operation meaningful;
- G3 validates graph-entry evidence but does not claim G5 execution persistence, query-resolution encoding, mapping, or synchronization.

The current planning artifacts are:

- [G3 focused design](../../design/active/runengpu-g3-access-work-graph-design.md);
- [G3 investigation](../../reports/investigations/runengpu-g3-access-work-graph-investigation.md);
- [G3 implementation specification](../specs/pt-runengpu-g3-access-work-graph.ron).

G3 Rust implementation is not active or authorized. Create one bounded implementation issue only after the G3 planning PR is independently reviewed and merged.

G4-G7 remain deferred to their accepted owners. No external package or extraction is authorized.

The target external repository remains `dornglut/runen-gpu`, but it is created only after internal G2-G8 conformance and extraction-readiness gates pass.

The proof portfolio remains separated:

- G5 deterministic conformance: exact inclusive/exclusive 4,097-element `u32` prefix scan;
- G5 stateful integration: headless fixed-seed Game of Life with full-grid CPU oracle, live-cell count, checksum, and selected-cell assertions;
- G6 graphics conformance: offscreen known-pattern draw;
- G6 GPU-driven composition: compute-generated indirect draw;
- G6 representative showcase: offscreen boids with structural and bounded invariants;
- G7 surface proof: reuse accepted G6 workloads;
- first RunenRender semantic proof: procedural sky/SDF terrain.

## Queued

- one bounded G3 implementation issue only after issue `#174` and its planning PR are accepted;
- G4 context/device admission, WGPU realization, shaders, pipelines, binding keys, backend layout, query-resolve offset-alignment admission, macro disposition, and removal of the temporary `RenderFlowId` resource-owner bridge;
- G5 execution, uploads, staging, query-resolution encoding, completion, asynchronous readback, cancellation, preserved execution state, and delayed retirement;
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
- G2 bounded implementation: issue `#172`, PR `#173`, merge `709aa6aced020ee99405e1e1c3dde7703c77a4d4`, and the [implementation closeout](../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md);
- RunenSDF standalone transfer, maintained authority, and Runenwerk duplicate-source retirement: Runenwerk PRs `#118` and `#157`, issue `#133`, and `dornglut/runen-sdf` PRs `#1`, `#2`, `#4`, `#5`, and `#6`.
