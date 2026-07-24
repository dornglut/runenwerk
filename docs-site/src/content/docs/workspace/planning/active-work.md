---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-24
related_docs:
  - ../engineering-workflow.md
  - ./roadmap.md
  - ./completed-work.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
---

# Active Work

GitHub issues and pull requests own live delivery. This page is only a concise cross-project summary.

## Active

### RunenGPU G1A

Issue `#131` owns the first internal RunenGPU implementation from the accepted post-RunenSDF current `main`.

The corrected target is owner-scoped rather than scalar-only:

```text
RenderResourceId
    -> GpuWorkResourceId { private owner scope, nonzero local value }

RenderResourceIdSequence
    -> owner-controlled GpuWorkResourceIdAllocator
```

The owner scope closes a confirmed collision seam: independent flows allocate the same local sequence values, while public uniform and storage handles can be passed into another flow. The implementation must prove that a foreign handle cannot resolve to an unrelated local resource.

G1A is intentionally isolated from WGPU, graph semantics, surfaces, shaders, renderer behavior, package creation, and external source movement. It is a future-transferable internal slice, not the RunenGPU extraction.

## Queued

- further internal RunenGPU phases only after G1A merges and closes against current `main`;
- external RunenGPU transfer only after G2-G8 and conformance;
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
- RunenSDF standalone transfer, maintained authority, and Runenwerk duplicate-source retirement: Runenwerk PRs `#118` and `#157`, issue `#133`, and `dornglut/runen-sdf` PRs `#1`, `#2`, `#4`, `#5`, and `#6`.
