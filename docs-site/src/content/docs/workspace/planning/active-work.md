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
  - ../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md
---

# Active Work

GitHub issues and pull requests own live delivery state. This page is a concise
cross-project summary, not an execution ledger.

## Active

### Repository workflow cleanup

Issue: `#122`

The canonical `cargo validate` baseline and engineering workflow landed through
PR `#123`. Draft PR `#124` retires the obsolete Python orchestration platform,
machine execution state, structured production-track databases, and ambiguous
gate scripts while retaining human historical evidence.

### GPU/render architecture reconciliation

Draft PR `#119` records the intended `RunenRender -> RunenGPU` dependency
direction but predates current `main`. It requires rebase and reconciliation;
implementation remains unauthorized.

## Completed foundation

RunenSDF standalone transfer completed at revision
`d52badefc640d6dc6dcdd40268af3aea1bb8eefe` through Runenwerk PR `#118` and
`Crystonix/runen-sdf` PR `#1`.

## Queued decisions

- RunenSDF clean-cutover consumer audit and exact integration/removal decision.
- RunenGPU current-source inventory and first bounded execution contract.
- RunenECS R1 boundary repair.

No queued item is implementation authorization by itself.
