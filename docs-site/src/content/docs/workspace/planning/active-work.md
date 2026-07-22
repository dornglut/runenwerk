---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
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

### Repository surface cleanup

Issue `#135` owns the final workflow and documentation simplification:

- remove the duplicate Task command layer;
- reduce root documents to concise entrypoints;
- consolidate canonical workspace documentation;
- correct stale planning state;
- simplify the path-scoped Astro build workflow;
- preserve `cargo validate`, permanent CI, and the Astro/Starlight site.

No runtime or framework behavior is in scope.

## Next implementation

### RunenGPU G1A

Issue `#131` remains open for the accepted migration:

```text
RenderResourceId         -> GpuWorkResourceId
RenderResourceIdSequence -> GpuWorkResourceIdAllocator
```

PR `#132` was closed without merge because it contained only temporary automation scaffolding. The Rust implementation has not started. After issue `#135` merges, G1A restarts from current `main` on one ordinary implementation branch with no additional planning or activation PR.

## Queued

- RunenSDF clean-cutover consumer audit and exact integration/removal decision.
- Further internal RunenGPU boundary slices after G1A is merged and reviewed.
- RunenECS boundary repair as a separately scheduled, non-conflicting workstream.

## Completed foundation

- workflow execution platform retirement: issues/PRs `#122`, `#123`, and `#124`;
- GPU/render architecture correction: issue `#125`, PR `#126`;
- GPU/render S0 inventory: issue `#127`, PR `#128`;
- G1A implementation specification: issue `#129`, PR `#130`;
- RunenSDF standalone transfer: Runenwerk PR `#118` and `Crystonix/runen-sdf` PR `#1`.
