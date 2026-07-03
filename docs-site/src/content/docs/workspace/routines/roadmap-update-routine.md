---
title: Roadmap Update Routine
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ../planning/README.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../authority-model.md
  - ../../guidelines/programming-principles.md
---

# Roadmap Update Routine

## Use when

Use this routine for planning record changes.

## Authority files to read

Read `planning/README.md`, `active-work.md`, `roadmap.md`, `deferred-work.md`, `completed-work.md`, `production-tracks.md`, `decision-register.md`, `authority-model.md`, `workflow-lifecycle.md`, `complete-investigation-gate.md`, `complete-design-gate.md`, and `programming-principles.md`.

## Working files to inspect

Inspect the planning files being changed and any owning roadmap, design, ADR, report, closeout evidence, complete investigation gate evidence, or complete design gate evidence.

## What to decide before editing

Decide whether the change is active, deferred, completed, strategic, historical, accepted direction, track candidate, active planning, or active implementation.

For work moving toward design, planning activation, or active implementation, decide whether complete investigation gate evidence is recorded.

For work moving toward active implementation, decide whether the complete implementation contract is recorded and whether `complete-design-gate.md` applies.

## State transitions produced

This routine may move work between accepted direction, track candidate, production track, active planning, active implementation, deferred, completed, or superseded states.

Record significant transitions in `decision-register.md`.

## Patch rules

Patch Markdown planning records first. Keep generated views and structured files as optional mirrors unless a machine contract requires them.

Do not mark implementation active unless exact owner, complete implementation contract, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, and stop conditions are known.

For architecture-sensitive, reusable, platform, public API, production-track, workflow, or domain-boundary work, do not mark implementation active unless the planning record points to complete investigation gate evidence and complete design gate evidence, including feature support, future-use-case pressure, hierarchy/composition where relevant, and ergonomics/usability evidence.

## Manual validation checklist

Confirm planning consistency, current focus, lifecycle state, concrete blockers, completion evidence, complete investigation gate evidence where applicable, complete design gate evidence where applicable, reactivation conditions, state-transition records, and stale mirrors.

## Stop conditions

Stop and redesign if the update would create multiple current focuses, mark work complete without evidence, activate implementation from an accepted design alone, activate implementation without required complete investigation or design gate evidence, reactivate deferred work without a reason, or move long design rationale into planning records.

## Evidence to report

Report planning files changed, state transitions, complete investigation gate status where applicable, complete design gate status where applicable, evidence, risks, stale mirrors, and next step.

## Optional local helpers

Run planning validation helpers only when a local checkout is available.
