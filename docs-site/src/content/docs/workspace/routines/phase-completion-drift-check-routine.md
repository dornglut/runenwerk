---
title: Phase Completion Drift Check Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ../workflow-lifecycle.md
  - ../planning/completed-work.md
  - ../../reports/closeouts/README.md
---

# Phase Completion Drift Check Routine

## Use when

Use this routine after a completed phase or slice before starting the next one.

## Authority files to read

Read the accepted scope, changed files, tests, docs, planning records, reports, closeout evidence, `workflow-lifecycle.md`, and `programming-principles.md`.

## Working files to inspect

Inspect files changed by the completed work and docs/planning records that claim completion.

## What to decide before editing

Decide whether the phase is complete, incomplete, drifted, still risky, deferred, or superseded.

## State transitions produced

This routine may move work from review or active implementation to completed, deferred, superseded, or active planning.

## Patch rules

Do not start new implementation in this routine. Patch only status, docs, closeout reports, or planning records needed to make completion truthful.

Keep `completed-work.md` as a short index. Put detailed evidence in `reports/closeouts/` when needed.

## Manual validation checklist

Check promised work vs implemented work, code/docs drift, dependency drift, validation status, known gaps, closeout evidence, lifecycle state, and next safe action.

## Stop conditions

Stop and redesign if completion is claimed without evidence, validation status is unknown and not reported, known gaps are hidden, active planning for the next phase starts before closeout truth is clear, or closeout evidence would bloat planning indexes.

## Evidence to report

Report completed scope, files changed, evidence inspected, validation status, drift found, gaps, lifecycle state transition, and next action.

## Optional local helpers

Run focused tests or docs validation when available.
