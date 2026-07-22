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
  - ./decision-register.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
  - ../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md
---

# Active Work

GitHub issues and pull requests own live delivery state. This page is a concise
cross-project summary, not an execution ledger.

## Active

### GPU/render S0 inventory

Issue: `#127`

Draft PR `#128` records the complete deterministic current-source inventory required
before RunenGPU implementation:

- 174 primary render/macro files classified;
- every file assigned move, stay, redesign, or delete disposition;
- 23 identity-like values classified by semantic owner and stability;
- 111 direct source consumer files and 963 matches inventoried;
- shader, macro, surface, device, lifecycle, and validation boundaries recorded;
- environment-dependent GPU proof explicitly deferred;
- one bounded first candidate identified.

The candidate is:

```text
G1A
RenderResourceId -> GpuWorkResourceId
```

S0 does not authorize implementation. The next step after S0 merges is one exact
G1A implementation specification based on current `main`.

## Completed foundation

### Repository workflow cleanup

Issue `#122` completed through PRs `#123` and `#124`. The obsolete workflow
orchestration and generated state are removed. `cargo validate` and exact-head CI
are authoritative.

### GPU/render architecture correction

Issue `#125` completed through PR `#126`. Accepted direction:

```text
RunenRender -> RunenGPU
```

RunenGPU and RunenRender each begin with one public package. WGPU belongs to
RunenGPU; RunenRender owns image formation.

### RunenSDF standalone transfer

RunenSDF standalone transfer completed through Runenwerk PR `#118` and
`Crystonix/runen-sdf` PR `#1` at revision:

```text
d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

Current Runenwerk `main` does not yet record the later clean-cutover removal of
`domain/sdf` as complete.

## Queued decisions

- exact G1A implementation specification after S0 merges;
- RunenSDF clean-cutover consumer audit and exact integration/removal decision;
- RunenECS R1 boundary repair.

No queued decision is implementation authorization by itself.
