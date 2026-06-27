---
title: Roadmap Update Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ../planning/README.md
  - ../workflow-lifecycle.md
  - ../authority-model.md
  - ../../guidelines/programming-principles.md
---

# Roadmap Update Routine

## Use when

Use this routine for planning record changes.

## Authority files to read

Read `planning/README.md`, `active-work.md`, `roadmap.md`, `deferred-work.md`, `completed-work.md`, `production-tracks.md`, `decision-register.md`, `authority-model.md`, `workflow-lifecycle.md`, and `programming-principles.md`.

## Working files to inspect

Inspect the planning files being changed and any owning roadmap, design, ADR, report, or closeout evidence.

## What to decide before editing

Decide whether the change is active, deferred, completed, strategic, historical, accepted direction, track candidate, active planning, or active implementation.

## State transitions produced

This routine may move work between accepted direction, track candidate, production track, active planning, active implementation, deferred, completed, or superseded states.

Record significant transitions in `decision-register.md`.

## Patch rules

Patch Markdown planning records first. Keep generated views and structured files as optional mirrors unless a narrow machine contract requires them.

Do not mark implementation active unless exact owner, scope, validation envelope, evidence expectation, and stop conditions are known.

## Manual validation checklist

Confirm planning consistency, current focus, lifecycle state, concrete blockers, completion evidence, reactivation conditions, state-transition records, and stale mirrors.

## Stop conditions

Stop and redesign if the update would create multiple current focuses, mark work complete without evidence, activate implementation from an accepted design alone, reactivate deferred work without a reason, or move long design rationale into planning records.

## Evidence to report

Report planning files changed, state transitions, evidence, risks, stale mirrors, and next step.

## Optional local helpers

Run planning validation helpers only when a local checkout is available.
