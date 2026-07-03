---
title: Phase Completion Drift Check Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../evidence-quality-taxonomy.md
  - ../complete-merge-readiness-gate.md
  - ../planning/README.md
  - ../planning/active-work.md
  - ../planning/roadmap.md
  - ../planning/completed-work.md
  - ../planning/production-tracks.md
  - ../planning/decision-register.md
  - ../../reports/closeouts/README.md
---

# Phase Completion Drift Check Routine

## Use when

Use this routine after a completed phase or slice before starting the next one.

Use it whenever code, docs, or a validation report indicates a phase is done but planning records still show the phase as active, review, pending, or blocked.

## Authority files to read

Read the accepted scope, complete investigation gate evidence where applicable, complete design gate evidence where applicable, evidence quality taxonomy, merge readiness gate, changed files, tests, docs, planning records, reports, closeout evidence, `workflow-lifecycle.md`, and `programming-principles.md`.

For production-track phases, read:

```text
workspace/planning/active-work.md
workspace/planning/roadmap.md
workspace/planning/production-tracks.md
workspace/planning/completed-work.md
workspace/planning/decision-register.md
workspace/evidence-quality-taxonomy.md
workspace/complete-merge-readiness-gate.md
reports/closeouts/README.md
```

## Working files to inspect

Inspect files changed by the completed work and docs/planning records that claim completion.

Also inspect the owning design or ADR when completion changes a design from planning authority to completed reference.

## What to decide before editing

Decide whether the phase is complete, incomplete, drifted, still risky, deferred, or superseded.

Decide whether merge readiness has been satisfied, whether evidence classes support the completion claim, and whether the next phase may become active planning, must remain inactive, requires a complete investigation gate first, or requires a complete design gate/design intake first.

## State transitions produced

This routine may move work from review or active implementation to completed, deferred, superseded, or active planning.

It must not move the next phase to active implementation.

## Patch rules

Do not start new implementation in this routine. Patch only status, docs, closeout reports, or planning records needed to make completion truthful.

Keep `completed-work.md` as a short index. Put detailed evidence in `reports/closeouts/` when needed.

When a phase is completed, update the completion truth set:

```text
delivered contract
evidence classes used
validation status
merge readiness status when applicable
known gaps
follow-up
lifecycle transition
active-work update
roadmap update
production-track update when applicable
completed-work entry
decision-register entry for lifecycle changes
closeout report when evidence is too large for completed-work.md
owning design status update when the design changes from planning/design to completed reference
complete investigation gate status for the next phase when applicable
complete design gate status for the next phase when applicable
```

The next phase may be opened as active planning in the same patch only after the completed phase is recorded truthfully and the next phase is explicitly not implementation-authorized.

## Manual validation checklist

Check promised work vs implemented work, code/docs drift, dependency drift, evidence classes, validation status, merge readiness, known gaps, closeout evidence, lifecycle state, active-work state, roadmap state, production-track state, completed-work entry, decision-register transition, complete investigation gate status for the next phase where applicable, complete design gate status for the next phase where applicable, and next safe action.

## Stop conditions

Stop and redesign if completion is claimed without evidence, evidence class is too weak for the decision, validation status is unknown and not reported, merge readiness is unknown when a merge decision is being made, known gaps are hidden, active planning for the next phase starts before closeout truth is clear, active implementation for the next phase is opened by this routine, required complete investigation or design gate evidence is missing, or closeout evidence would bloat planning indexes.

## Evidence to report

Report completed scope, files changed, evidence classes used, validation status, merge readiness status, drift found, gaps, lifecycle state transition, planning files updated, complete investigation gate status where applicable, complete design gate status where applicable, closeout report status, and next action.

## Optional local helpers

Run focused tests or docs validation when available.
