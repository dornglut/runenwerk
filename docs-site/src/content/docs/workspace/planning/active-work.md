---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../engineering-workflow.md
  - ./roadmap.md
  - ./completed-work.md
  - ./decision-register.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md
---

# Active Work

GitHub issues and pull requests own live delivery state. This page is a concise
cross-project summary, not an execution ledger.

## Active

### GPU/render architecture correction

Issue: `#125`

Closed PR `#119` is superseded. It correctly identified the need to separate GPU
execution from image formation, but it was based on stale repository state,
proposed speculative multi-package splits, and edited planning authorities removed
by workflow cleanup.

The successor work is based on current `main` and establishes:

```text
RunenRender -> RunenGPU
one public package per repository
WGPU internal to RunenGPU
no implementation phase before S0 inventory
```

Scope is documentation/architecture only. No Rust, Cargo, shader, WGPU, external
repository source, or runtime behavior is authorized.

## Completed foundation

### Repository workflow cleanup

Issue `#122` completed through PRs `#123` and `#124`.

The obsolete Python orchestration platform, machine execution state, structured
production-track databases, generated prompts, truth certificates, batch tools,
and quiet/full gate scripts are removed. `cargo validate` and exact-head CI are the
canonical baseline.

### RunenSDF standalone transfer

RunenSDF standalone transfer completed through Runenwerk PR `#118` and
`Crystonix/runen-sdf` PR `#1` at revision:

```text
d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

Current Runenwerk `main` does not yet record the later clean-cutover removal of
`domain/sdf` as complete.

## Queued decisions

- GPU/render S0 current-source and consumer inventory under issue `#125` after the
  architecture correction merges.
- RunenSDF clean-cutover consumer audit and exact integration/removal decision.
- RunenECS R1 boundary repair.

No queued decision is implementation authorization by itself.
