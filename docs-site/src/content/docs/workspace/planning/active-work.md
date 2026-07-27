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
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../specs/pt-runengpu-g3-access-work-graph.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page is a concise cross-project summary.

## Active

### Runen family operational hardening — issue `#176`

Accepted foundation:

```text
RunenGPU G1A  complete
RunenGPU G2   complete through #172 / PR #173
RunenGPU G3 planning accepted through #174 / PR #175
accepted G3 planning merge 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
```

Issue `#176` is the active documentation-only slice. It owns:

- evidence-based classification of external GPU/renderer weaknesses and complaints;
- distinction between Runen mitigation, inherited backend limitation, and new Runen risk;
- family-wide accepted-work, pressure, cache, compatibility, recovery, and reproducibility doctrine;
- G4-G8 operational requirements without changing accepted G3 semantics;
- RunenRender provider maturity, narrow capability, incremental scene, cache, and performance requirements;
- operational proof workloads for progress, saturation, cancellation, shutdown, cache compatibility, device loss, reconstruction, capture, and direct-WGPU comparison;
- ranked application-domain fit and explicit domain non-ownership;
- stale index and planning reconciliation.

The active branch is:

```text
docs/runen-family-operational-hardening
```

This slice changes documentation only. It does not authorize Rust, package,
dependency, lockfile, workflow, external repository, G3 API, or new G/R phase changes.

### Current operational findings

Current timing readback:

```text
map_async
-> device.poll(wait_indefinitely)
-> channel receive
-> decode/publish evidence
```

This is valid current local behavior but not a reusable progress contract. G5 must
bind native/web progress ownership, callback/reentrancy rules, bounded waits, quotas,
completion delivery, cancellation, and pending-work shutdown.

Pipeline cache, driver/compiler stalls, device loss, and platform differences are
inherited backend facts. RunenGPU must normalize compatibility and diagnostics without
claiming to remove them.

## Queued

### Bounded G3 implementation — issue `#177`

Issue `#177` owns the accepted G3 Rust implementation but is explicitly blocked on
accepted completion of `#176`.

Before source changes it must:

1. record the exact post-`#176` `main` as implementation base;
2. run `cargo validate` and `git diff --check`;
3. repeat the current declaration/consumer census;
4. verify no persisted or external contract changed;
5. confirm no G4/G5/G7 ownership is required.

The G3 implementation remains limited to checked access, graph-entry initialization,
render attachments and clear values, buffer zero, query resolution, hazards,
import/export causality, derived requirements, immutable work, deterministic
preparation, one temporary adapter, full migration, and deletion of replaced generic
renderer authority.

### Later RunenGPU phases

- G4: context/device admission, portability classes, cache compatibility, WGPU realization, shaders/pipelines/binding layout, generations, stale-generation validation, and removal of the temporary renderer-derived owner bridge;
- G5: preparation/submission, progress, pressure, uploads, query-resolution encoding, completion, asynchronous readback, cancellation, pending-work shutdown, and delayed retirement;
- G6: offscreen graphics, direct-WGPU comparisons, GPU-driven composition, and shared render/non-render proof;
- G7: surfaces, thread affinity, device generations/loss, and reconstruction facts;
- G8: operational conformance, reproducibility bundle, recovery evidence, diagnostics, shutdown, cache/performance audit, and no reach-through;
- GX: external `dornglut/runen-gpu` transfer only after internal G2-G8 conformance.

### RunenRender

RunenRender remains S0/design only until accepted external RunenGPU cutover.
Near-term proof pressure is procedural/analytic field content and overlays. Volume,
population, and regional summaries are research candidates. Fiber, liquid, broad
hardware-specialized providers, and universal provider unification remain deferred.

## Retained proof portfolio

```text
G5 correctness
    exact prefix scan/readback
    headless Game of Life
    deterministic compute-to-texture

G5 operations
    submission/readback/upload saturation
    native/web progress and callback proof
    cancellation and pending-work shutdown

G6 graphics and cost
    known-pattern offscreen draw
    compute-generated indirect draw
    direct-WGPU narrow comparisons
    offscreen boids

G7 lifecycle
    surface outcomes
    device generations
    reconstruction matrix

G8 conformance
    cache behavior
    reproducibility bundle
    recovery and residual audit

RunenRender
    procedural sky/SDF terrain
    incremental prepared scene
    synthetic volume
    cache/history invalidation
```

## Completed foundation

- workflow execution platform retirement: issues/PRs `#122`, `#123`, and `#124`;
- final repository-surface pruning: issue `#135`, PR `#136`;
- Rust 1.97 and documentation baseline recovery: issues `#150` and `#154`, PR `#155`;
- shared Rust validation adoption: issue `#137`, PR `#138`;
- root architecture foundation alignment: PR `#141`;
- GPU/render architecture correction: issue `#125`, PR `#126`;
- GPU/render S0 inventory: issue `#127`, PR `#128`;
- G1A implementation: issue `#131`, PR `#164`, merge `5bbdab36ae661d99432bfe5d215062c397aac975`;
- G2 decision and implementation: issues `#168` and `#172`, PRs `#171` and `#173`, merge `709aa6aced020ee99405e1e1c3dde7703c77a4d4`;
- G3 decision phase: issue `#174`, PR `#175`, merge `5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8`;
- RunenSDF standalone transfer and Runenwerk duplicate-source retirement: issue `#133`, Runenwerk PRs `#118` and `#157`, and accepted `dornglut/runen-sdf` work.
