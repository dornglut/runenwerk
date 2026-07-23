---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-23
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

### RunenSDF clean cutover

Issue `#133` owns retirement of the duplicate internal `domain/sdf` authority after the standalone source transfer and repository-identity correction.

Current evidence indicates that `domain/sdf` remains a workspace member but may have no live product consumer. The implementation must begin with a complete Cargo, Rust, test, adapter, documentation, and persisted-format census.

If the census confirms zero consumers, the accepted result is retirement-only:

```text
delete domain/sdf
remove workspace and lockfile authority
remove stale active duplicate-source authority
add a durable no-return guard
add no standalone dependency
```

If a real consumer is found, stop and record it before adding the exact standalone dependency. No compatibility package, alias, forwarding namespace, source include, submodule, branch dependency, or duplicate source may remain.

Canonical standalone authority:

```text
repository: dornglut/runen-sdf
maintained revision: ffa970f3eb7fd9ebaa1cfc67665e3e3128cd0676
source-transfer revision: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

## Next implementation

### RunenGPU G1A

Issue `#131` owns the first internal RunenGPU implementation after the SDF cutover merges.

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
- RunenSDF standalone transfer and maintained authority: Runenwerk PR `#118`, `dornglut/runen-sdf` PRs `#1`, `#2`, `#4`, and `#5`.
