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
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
---

# Active Work

GitHub issues and pull requests own live delivery. This page is only a concise cross-project summary.

## Active

### RunenGPU G2 planning

RunenGPU G1A is implemented, merged, and closed against current `main`. The next bounded decision is G2: capabilities, resource descriptors, the typed GPU-data seam, public API ergonomics, and the first explicit decomposition of the transitional `RenderFlow` authority.

G2 planning must begin from merged G1A facts rather than pre-authored assumptions. Its specification must establish:

- the exact current capability and resource-descriptor inventory;
- the exact disposition of current `RenderFlow` resource, handle, capability, typed-data, ECS-projection, and backend-realization seams;
- the future-transferable ownership boundary between RunenGPU work contracts, RunenRender policy, and Runenwerk adapters;
- independent resource dimensions for kind, lifetime, ownership, transfer, and reconstruction;
- public and internal type placement;
- a one-call ordinary submission path that validates automatically;
- a separate inspectable prepare/submit path for diagnostics and tooling;
- typed resource and binding references, with strings limited to human labels;
- lexical/closure-scoped builders rather than nested `finish()` ladders;
- inferred dependency ordering from declared access, with explicit ordering only for non-data dependencies;
- RAII public handles with safe delayed backend retirement;
- human-readable structured errors that include operation, label, cause, provenance, and corrective action;
- the `GpuParams`/`GpuUniform`/`GpuStorage` byte-layout and macro evidence that must be decided in G4 rather than copied from engine-specific paths;
- the prepared-value/upload boundary replacing direct ECS projection inside future framework APIs;
- exact render, non-render compute, and Runenwerk-adapter proof-consumer candidates;
- migration scope, dependency guards, tests, and stop conditions;
- explicit exclusions for later access hazards, work graphs, WGPU realization, execution, surfaces, and external extraction.

The common public path must not require callers to understand `GpuWorkGraph`, execution epochs, admission, realization, or retirement terminology. Those remain internal or advanced concepts unless a caller explicitly requests inspection or lower-level control.

The same planning PR must correct stale active authority that still describes S0 or G1 as pending, correct the repository identity to `dornglut/runen-gpu`, and align the phase sequence so context/device admission belongs with G4 backend realization while G5 owns headless execution and transfers.

The [industry comparison](../../reports/investigations/runengpu-industry-comparison.md) and [public API ergonomics review](../../reports/investigations/runengpu-public-api-ergonomics-review.md) are supporting evidence. Together they constrain the target to be broader and safer than a direct WGPU wrapper, more general than a render-only frame graph, simpler than a mature AAA render dependency graph, and understandable without exposing framework internals in ordinary code.

No G2 source implementation is authorized until a decision-complete specification and owning implementation issue are accepted.

## Queued

- G2 implementation after its specification is accepted;
- G3-G7 as individually specified slices that preserve the simple/advanced API split while migrating and deleting the authority each phase replaces;
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
