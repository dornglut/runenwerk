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
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
---

# Active Work

GitHub issues and pull requests own live delivery. This page is only a concise cross-project summary.

## Active

### RunenGPU G2 planning

RunenGPU G1A is implemented, merged, and closed against current `main`. The next bounded decision is G2: capabilities, resource descriptors, the typed GPU-data seam, and the first explicit decomposition of the transitional `RenderFlow` authority.

G2 planning must begin from merged G1A facts rather than pre-authored assumptions. Its specification must establish:

- the exact current capability and resource-descriptor inventory;
- the exact disposition of current `RenderFlow` resource, handle, capability, typed-data, ECS-projection, and backend-realization seams;
- the future-transferable ownership boundary between RunenGPU work contracts, RunenRender policy, and Runenwerk adapters;
- public and internal type placement;
- structured validation and error contracts;
- the `GpuParams`/`GpuUniform`/`GpuStorage` byte-layout and macro evidence that must be decided in G4 rather than copied from engine-specific paths;
- the prepared-value/upload boundary replacing direct ECS projection inside future framework APIs;
- exact render, non-render compute, and Runenwerk-adapter proof-consumer candidates;
- migration scope, dependency guards, tests, and stop conditions;
- explicit exclusions for later access hazards, work graphs, WGPU realization, execution, surfaces, and external extraction.

The same planning PR must correct stale active authority that still describes S0 or G1 as pending, correct the repository identity to `dornglut/runen-gpu`, and align the phase sequence so context/device admission belongs with G4 backend realization while G5 owns headless execution and transfers.

No G2 source implementation is authorized until a decision-complete specification and owning implementation issue are accepted.

## Queued

- G2 implementation after its specification is accepted;
- G3-G7 as individually specified slices that migrate and delete the authority each phase replaces;
- G8 as diagnostics, shutdown, residual anti-cheating audit, and internal conformance rather than a deferred first migration;
- external RunenGPU transfer only after G2-G8 and standalone conformance;
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
- RunenSDF standalone transfer, maintained authority, and Runenwerk duplicate-source retirement: Runenwerk PRs `#118` and `#157`, issue `#133`, and `dornglut/runen-sdf` PRs `#1`, `#2`, `#4`, `#5`, and `#6`.
